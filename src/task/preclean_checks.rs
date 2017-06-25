/****************************************************************************
**
** svgcleaner could help you to clean up your SVG files
** from unnecessary data.
** Copyright (C) 2012-2017 Evgeniy Reizner
**
** This program is free software; you can redistribute it and/or modify
** it under the terms of the GNU General Public License as published by
** the Free Software Foundation; either version 2 of the License, or
** (at your option) any later version.
**
** This program is distributed in the hope that it will be useful,
** but WITHOUT ANY WARRANTY; without even the implied warranty of
** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
** GNU General Public License for more details.
**
** You should have received a copy of the GNU General Public License along
** with this program; if not, write to the Free Software Foundation, Inc.,
** 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
**
****************************************************************************/

use svgdom::{
    AttributeType,
    AttributeValue,
    Document,
};

use task::short::{EId, AId};
use error::Error;

pub fn preclean_checks(doc: &Document) -> Result<(), Error> {
    check_for_unsupported_elements(doc)?;
    check_for_script_attributes(doc)?;
    check_for_conditional_attributes(doc)?;
    check_for_external_xlink(doc)?;

    Ok(())
}

fn check_for_unsupported_elements(doc: &Document) -> Result<(), Error> {
    for node in doc.descendants().svg() {
        match node.tag_id().unwrap() {
            EId::Script => {
                return Err(Error::ScriptingIsNotSupported);
            }
              EId::Animate
            | EId::AnimateColor
            | EId::AnimateMotion
            | EId::AnimateTransform
            | EId::Set => {
                return Err(Error::AnimationIsNotSupported);
            }
            _ => {}
        }
    }

    Ok(())
}

fn check_for_script_attributes(doc: &Document) -> Result<(), Error> {
    for node in doc.descendants().svg() {
        for attr in node.attributes().iter() {
            if     attr.is_graphical_event()
                || attr.is_document_event()
                || attr.is_animation_event()
            {
                return Err(Error::ScriptingIsNotSupported);
            }
        }
    }

    Ok(())
}

fn check_for_conditional_attributes(doc: &Document) -> Result<(), Error> {
    // TODO: what to do with 'requiredExtensions'?

    macro_rules! check_attr {
        ($aid:expr, $node:expr) => (
            let attrs = $node.attributes();
            if let Some(&AttributeValue::String(ref s)) = attrs.get_value($aid) {
                if !s.is_empty() {
                    // NOTE: We are only care about non-empty attributes.
                    return Err(Error::ConditionalProcessingIsNotSupported);
                }
            }
        )
    }

    for node in doc.descendants().svg() {
        check_attr!(AId::RequiredFeatures, node);
        check_attr!(AId::SystemLanguage, node);
    }

    Ok(())
}

fn check_for_external_xlink(doc: &Document) -> Result<(), Error> {
    for node in doc.descendants().svg() {
        if !node.has_attribute(AId::XlinkHref) {
            continue;
        }

        match node.tag_id().unwrap() {
              EId::A
            | EId::Image
            | EId::FontFaceUri
            | EId::FeImage => continue,
            _ => {}
        }

        let attrs = node.attributes();
        if let AttributeValue::String(ref s) = *attrs.get_value(AId::XlinkHref).unwrap() {
            return Err(Error::ExternalHrefIsNotSupported(s.clone()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svgdom::Document;
    use error::Error;

    macro_rules! test {
        ($name:ident, $in_text:expr, $err:expr) => (
            #[test]
            fn $name() {
                let doc = Document::from_str($in_text).unwrap();
                assert_eq!(preclean_checks(&doc).err().unwrap(), $err);
            }
        )
    }

    macro_rules! test_ok {
        ($name:ident, $in_text:expr) => (
            #[test]
            fn $name() {
                let doc = Document::from_str($in_text).unwrap();
                assert_eq!(preclean_checks(&doc).is_ok(), true);
            }
        )
    }

    test!(test_scripting_1, "<svg><script/></svg>",
          Error::ScriptingIsNotSupported);

    test!(test_scripting_2, "<svg onload=''/>",
          Error::ScriptingIsNotSupported);

    test!(test_animation_1, "<svg><set/></svg>",
          Error::AnimationIsNotSupported);

    test!(test_conditions_1, "<svg><switch requiredFeatures='text'/></svg>",
          Error::ConditionalProcessingIsNotSupported);

    test!(test_conditions_2, "<svg><switch systemLanguage='en'/></svg>",
          Error::ConditionalProcessingIsNotSupported);

    test_ok!(test_conditions_3, "<svg><switch requiredFeatures=''/></svg>");
    test_ok!(test_conditions_4, "<svg><switch systemLanguage=''/></svg>");
}
