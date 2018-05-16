// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    self,
    PaintFallback,
};

// self
use tree;
use tree::prelude::*;
use super::prelude::*;


pub fn convert(
    rtree: &tree::Tree,
    attrs: &svgdom::Attributes,
) -> Option<tree::Fill> {
    let paint = resolve_paint(rtree, attrs, AId::Fill)?;

    let fill_opacity = attrs.get_number(AId::FillOpacity).unwrap_or(1.0);

    let fill_rule = attrs.get_str(AId::FillRule).unwrap_or("nonzero");
    let fill_rule = match fill_rule {
        "evenodd" => tree::FillRule::EvenOdd,
        _ => tree::FillRule::NonZero,
    };

    let fill = tree::Fill {
        paint,
        opacity: fill_opacity.into(),
        rule: fill_rule,
    };

    Some(fill)
}

pub fn resolve_paint(
    rtree: &tree::Tree,
    attrs: &svgdom::Attributes,
    aid: AId,
) -> Option<tree::Paint> {
    match attrs.get_type(aid) {
        Some(&AValue::Color(c)) => {
            Some(tree::Paint::Color(c))
        }
        Some(&AValue::Paint(ref link, fallback)) => {
            // a-fill-016.svg
            // a-fill-017.svg
            // a-fill-018.svg

            if link.is_gradient() || link.is_tag_name(EId::Pattern) {
                if let Some(node) = rtree.defs_by_id(&link.id()) {
                    Some(tree::Paint::Link(node.id().to_string()))
                } else if let Some(PaintFallback::Color(c)) = fallback {
                    Some(tree::Paint::Color(c))
                } else {
                    None
                }
            } else {
                // a-fill-023.svg
                warn!("'{}' cannot be used to {} the shape.", link.tag_name(), aid);
                None
            }
        }
        Some(&AValue::None) => {
            // a-fill-020.svg
            None
        }
        Some(av) => {
            warn!("An invalid {} value: {}. Skipped.", aid, av);
            None
        }
        None => None,
    }
}
