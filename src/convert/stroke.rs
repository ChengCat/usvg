// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    self,
    ElementType,
    FuzzyEq,
};

// self
use tree::prelude::*;
use short::{
    AId,
    AValue,
    EId,
};
use traits::{
    GetValue,
};


pub fn convert(
    rtree: &tree::Tree,
    attrs: &svgdom::Attributes,
) -> Option<tree::Stroke> {
    let dashoffset  = attrs.get_number(AId::StrokeDashoffset).unwrap_or(0.0);
    // a-stroke-miterlimit-001.svg
    let miterlimit  = attrs.get_number(AId::StrokeMiterlimit).unwrap_or(4.0);
    let opacity     = attrs.get_number(AId::StrokeOpacity).unwrap_or(1.0).into();
    let width       = attrs.get_number(AId::StrokeWidth).unwrap_or(1.0);

    if !(width > 0.0) {
        return None;
    }

    // Must be bigger than 1.
    //
    // a-stroke-miterlimit-003.svg
    let miterlimit = if miterlimit < 1.0 { 1.0 } else { miterlimit };

    let paint = if let Some(stroke) = attrs.get_type(AId::Stroke) {
        match *stroke {
            AValue::Color(c) => {
                tree::Paint::Color(c)
            }
            AValue::FuncLink(ref link) => {
                let mut p = None;
                if link.is_gradient() || link.is_tag_name(EId::Pattern) {
                    if let Some(node) = rtree.defs_by_id(&link.id()) {
                        p = Some(tree::Paint::Link(node.id().to_string()));
                    }
                }

                match p {
                    Some(p) => p,
                    None => {
                        warn!("'{:?}' cannot be used for stroking.",
                              link.tag_name());
                        return None;
                    }
                }
            }
            AValue::PredefValue(svgdom::ValueId::None) => {
                return None;
            }
            _ => {
                warn!("An invalid stroke value: {}. Skipped.", stroke);
                return None;
            }
        }
    } else {
        return None;
    };

    let linecap = attrs.get_predef(AId::StrokeLinecap).unwrap_or(svgdom::ValueId::Butt);
    let linecap = match linecap {
        svgdom::ValueId::Butt => tree::LineCap::Butt,
        svgdom::ValueId::Round => tree::LineCap::Round,
        svgdom::ValueId::Square => tree::LineCap::Square,
        _ => tree::LineCap::Butt,
    };

    let linejoin = attrs.get_predef(AId::StrokeLinejoin).unwrap_or(svgdom::ValueId::Miter);
    let linejoin = match linejoin {
        svgdom::ValueId::Miter => tree::LineJoin::Miter,
        svgdom::ValueId::Round => tree::LineJoin::Round,
        svgdom::ValueId::Bevel => tree::LineJoin::Bevel,
        _ => tree::LineJoin::Miter,
    };

    let dasharray = conv_dasharray(attrs.get_value(AId::StrokeDasharray));

    let stroke = tree::Stroke {
        paint,
        dasharray,
        dashoffset,
        miterlimit,
        opacity,
        width,
        linecap,
        linejoin,
    };

    Some(stroke)
}

// Prepare the 'stroke-dasharray' according to:
// https://www.w3.org/TR/SVG/painting.html#StrokeDasharrayProperty
fn conv_dasharray(av: Option<&AValue>) -> Option<svgdom::NumberList> {
    if let Some(&AValue::NumberList(ref list)) = av {
        // `A negative value is an error`
        //
        // a-stroke-dasharray-007.svg
        if list.iter().any(|n| n.is_sign_negative()) {
            return None;
        }

        // `If the sum of the values is zero, then the stroke is rendered
        // as if a value of none were specified.`
        //
        // a-stroke-dasharray-008.svg
        {
            // no Iter::sum(), because of f64

            let mut sum = 0.0f64;
            for n in list {
                sum += *n;
            }

            if sum.fuzzy_eq(&0.0) {
                return None;
            }
        }

        // `If an odd number of values is provided, then the list of values
        // is repeated to yield an even number of values.`
        //
        // a-stroke-dasharray-003.svg
        if list.len() % 2 != 0 {
            let mut tmp_list = list.clone();
            tmp_list.extend_from_slice(list);

            return Some(tmp_list.clone());
        }

        return Some(list.clone());
    }

    None
}
