// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use svgdom::{
    Length,
    NumberList,
    ViewBox,
};

use super::prelude::*;


/// Converts `Length` to `Number`.
///
/// Also converts `LengthList` to `NumberList`.
///
/// Details: https://www.w3.org/TR/SVG/coords.html#Units
pub fn convert_units(svg: &mut Node, opt: &Options) {
    // a-stroke-dasharray-004.svg
    // a-stroke-dasharray-005.svg
    // a-stroke-dasharray-006.svg
    // a-stroke-dashoffset-003.svg
    // a-stroke-dashoffset-004.svg
    // a-stroke-dashoffset-005.svg
    // a-stroke-width-005.svg

    // We should convert `font-size` before all other attributes,
    // because it's value used for `em`/`ex` conversion.
    convert_font_size(svg, opt.dpi);

    let view_box = resolve_view_box(svg, opt.dpi);

    let vb_len = (
        view_box.width * view_box.width + view_box.height * view_box.height
    ).sqrt() / 2.0_f64.sqrt();

    let convert_len = |len: Length, aid: AId, font_size: f64| {
        if len.unit == Unit::Percent {
            match aid {
                AId::X | AId::Cx | AId::Width  => convert_percent(len, view_box.width),
                AId::Y | AId::Cy | AId::Height => convert_percent(len, view_box.height),
                _ => convert_percent(len, vb_len),
            }
        } else {
            convert(len, font_size, opt.dpi)
        }
    };

    let mut is_bbox_gradient;
    for (_, mut node) in svg.descendants().svg() {
        is_bbox_gradient = false;

        if node.is_paint_server() {
            // `objectBoundingBox` is a default value.
            is_bbox_gradient = true;

            if node.attributes().get_str(AId::GradientUnits) == Some("userSpaceOnUse") {
                is_bbox_gradient = false;
            }
        }

        let font_size = node.find_attribute(AId::FontSize)
                            .expect("'font-size' must be resolved before units conversion.");

        let mut attrs = node.attributes_mut();

        // Convert Length to Number.
        for (aid, ref mut attr) in attrs.iter_mut().svg() {
            if let AValue::Length(len) = attr.value {
                let n = if len.num.is_fuzzy_zero() {
                    0.0
                } else if is_bbox_gradient && len.unit == Unit::Percent {
                    // In paint servers with `objectBoundingBox` 100% is equal to 1.0.
                    len.num / 100.0
                } else if aid == AId::Offset && len.unit == Unit::Percent {
                    // The `offset` % value does not depend on viewBox.
                    len.num / 100.0
                } else {
                    // In other elements % units are depend on viewBox.
                    convert_len(len, aid, font_size)
                };

                attr.value = AValue::Number(n);
            }
        }

        // Convert LengthList to NumberList.
        for (aid, ref mut attr) in attrs.iter_mut().svg() {
            let mut list = None;
            if let AValue::LengthList(ref len_list) = attr.value {
                list = Some(NumberList(len_list.iter()
                    .map(|len| convert_len(*len, aid, font_size))
                    .collect()));
            }

            if let Some(list) = list {
                attr.value = AValue::NumberList(list);
            }
        }
    }
}

fn convert_font_size(svg: &Node, dpi: f64) {
    for (_, mut node) in svg.descendants().svg() {
        // Get parent `font-size` in case of em/ex.
        //
        // Check only a parent, because `font-size` was already resolved for
        // all element by `resolve_font_size`.
        let parent_size = match node.parent() {
            Some(p) => {
                if p.is_root() {
                    DEFAULT_FONT_SIZE
                } else {
                    // Check that parent already resolved and replaced with Number.
                    debug_assert_eq!(p.attributes().get_value(AId::FontSize)
                                      .map(|v| v.is_number()), Some(true));

                    p.attributes().get_number_or(AId::FontSize, DEFAULT_FONT_SIZE)
                }
            }
            None => DEFAULT_FONT_SIZE,
        };

        let mut attrs = node.attributes_mut();

        if let Some(attr) = attrs.get_mut(AId::FontSize) {
            if let AValue::Length(len) = attr.value {
                let n = convert(len, parent_size, dpi);
                attr.value = AValue::Number(n);
            } else {
                warn!("'font-size' should have a Length type.");
                attr.value = AValue::Number(DEFAULT_FONT_SIZE);
            }
        }
    }
}

fn resolve_view_box(svg: &mut Node, dpi: f64) -> Rect {
    match svg.get_viewbox() {
        Some(vb) => vb,
        None => {
            debug_assert!(svg.has_attribute(AId::FontSize));

            let font_size = svg.attributes().get_number(AId::FontSize).unwrap();

            // Must be already resolved by resolve_svg_size.
            let width = svg.attributes().get_length(AId::Width).unwrap();
            let height = svg.attributes().get_length(AId::Height).unwrap();

            let width = convert(width, font_size, dpi);
            let height = convert(height, font_size, dpi);

            let vb = ViewBox::new(0.0, 0.0, width, height);
            svg.set_attribute((AId::ViewBox, vb));

            (0.0, 0.0, width, height).into()
        }
    }
}

fn convert(len: Length, font_size: f64, dpi: f64) -> f64 {
    let n = len.num;

    match len.unit {
        Unit::None | Unit::Px => n,
        Unit::Em => n * font_size,
        Unit::Ex => n * font_size / 2.0, // The same coefficient as in resolve_font_size.
        Unit::In => n * dpi,
        Unit::Cm => n * dpi / 2.54,
        Unit::Mm => n * dpi / 25.4,
        Unit::Pt => n * dpi / 72.0,
        Unit::Pc => n * dpi / 6.0,
        Unit::Percent => unreachable!("must be already converted"),
    }
}

fn convert_percent(len: Length, base: f64) -> f64 {
    base * len.num / 100.0
}
