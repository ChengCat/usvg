// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom;

// self
use {
    Options,
};


mod clip_element;
mod conv_units;
mod fix_gradient_stops;
mod fix_recursive_pattern;
mod fix_xlinks;
mod group_defs;
mod prepare_clip_path;
mod prepare_mask;
mod prepare_nested_svg;
mod prepare_text_decoration;
mod prepare_text_nodes;
mod regroup;
mod resolve_attrs_via_xlink;
mod resolve_children_via_xlink;
mod resolve_curr_color;
mod resolve_font_size;
mod resolve_font_weight;
mod resolve_inherit;
mod resolve_style_attrs;
mod resolve_svg_size;
mod resolve_tref;
mod resolve_use;
mod resolve_visibility;
mod rm_desc_elems;
mod rm_invalid_font_size;
mod rm_invalid_gradients;
mod rm_invalid_ts;
mod rm_invisible_elems;
mod rm_non_svg_data;
mod rm_unused_defs;
mod ungroup_a;
mod ungroup_groups;
mod ungroup_switch;


use self::conv_units::convert_units;
use self::fix_gradient_stops::fix_gradient_stops;
use self::fix_recursive_pattern::fix_recursive_pattern;
use self::fix_xlinks::*;
use self::group_defs::group_defs;
use self::resolve_children_via_xlink::*;
use self::prepare_clip_path::*;
use self::prepare_mask::resolve_mask_attributes;
use self::prepare_nested_svg::*;
use self::prepare_text_decoration::prepare_text_decoration;
use self::prepare_text_nodes::prepare_text_nodes;
use self::regroup::regroup_elements;
use self::resolve_curr_color::resolve_current_color;
use self::resolve_font_size::resolve_font_size;
use self::resolve_font_weight::resolve_font_weight;
use self::resolve_inherit::resolve_inherit;
use self::resolve_style_attrs::resolve_style_attributes;
use self::resolve_svg_size::resolve_svg_size;
use self::resolve_tref::resolve_tref;
use self::resolve_use::*;
use self::resolve_visibility::resolve_visibility;
use self::resolve_attrs_via_xlink::*;
use self::rm_desc_elems::remove_desc_elements;
use self::rm_invalid_font_size::remove_invalid_font_size;
use self::rm_invalid_gradients::remove_invalid_gradients;
use self::rm_invalid_ts::remove_invalid_transform;
use self::rm_invisible_elems::remove_invisible_elements;
use self::rm_non_svg_data::remove_non_svg_data;
use self::rm_unused_defs::remove_unused_defs;
use self::ungroup_a::ungroup_a;
use self::ungroup_groups::ungroup_groups;
use self::ungroup_switch::ungroup_switch;


// TODO: to options
// TODO: maybe use a system font
// Default font is user-agent dependent so we can use whatever we like.
pub const DEFAULT_FONT_FAMILY: &str = "Times New Roman";
pub const DEFAULT_FONT_SIZE: f64 = 12.0;

mod prelude {
    pub use svgdom::{
        AttributeType,
        Document,
        ElementType,
        FilterSvg,
        FilterSvgAttrs,
        FilterSvgAttrsMut,
        FuzzyEq,
        FuzzyZero,
        Node,
    };
    pub use super::DEFAULT_FONT_FAMILY;
    pub use super::DEFAULT_FONT_SIZE;
    pub use geom::*;
    pub use short::*;
    pub use traits::*;
    pub use Options;
}


/// Prepares an input `Document`.
///
/// # Errors
///
/// - If `Document` doesn't have an SVG node - clears the `doc`.
/// - If `Document` size can't be determined - clears the `doc`.
///
/// Basically, any error, even a critical one, should be recoverable.
/// In worst case scenario clear the `doc`.
///
/// Must not panic!
pub fn prepare_doc(doc: &mut svgdom::Document, opt: &Options) {
    let mut svg = if let Some(svg) = doc.svg_element() {
        svg
    } else {
        // Technically unreachable, because svgdom will return a parser error
        // if input SVG doesn't have an `svg` node.
        warn!("Invalid SVG structure. The Document will be cleared.");
        *doc = svgdom::Document::new();
        return;
    };

    let svg = &mut svg;

    // Detect image size. If it failed there is no point in continuing.
    if !resolve_svg_size(svg) {
        warn!("File doesn't have 'width', 'height' and 'viewBox' attributes. \
               Automatic image size determination is not supported. \
               The Document will be cleared.");
        *doc = svgdom::Document::new();
        return;
    }

    // TODO: remove duplicated defs

    remove_non_svg_data(doc);
    remove_desc_elements(doc);

    resolve_inherit(doc);
    resolve_current_color(doc);

    group_defs(doc, svg);

    resolve_mask_attributes(doc);
    resolve_use_attributes(doc);
    resolve_svg_attributes(doc);

    resolve_font_size(doc);
    resolve_font_weight(doc);

    convert_units(svg, opt);

    fix_xlinks(doc);

    resolve_linear_gradient_attributes(doc);
    resolve_radial_gradient_attributes(doc);
    resolve_gradient_stops(doc);
    fix_gradient_stops(doc);
    remove_invalid_gradients(doc);

    resolve_pattern_attributes(doc);
    resolve_pattern_children(doc);
    fix_recursive_pattern(doc);

    resolve_clip_path_attributes(doc);

    remove_unused_defs(doc, svg);

    prepare_nested_svg(doc, svg);

    // `use` should be resolved before style attributes,
    // because `use` can propagate own style.
    resolve_use(doc);

    ungroup_a(doc);

    prepare_text_decoration(doc);
    resolve_visibility(doc, svg);
    resolve_style_attributes(doc);

    resolve_tref(doc);

    remove_xlinks(doc);

    ungroup_switch(doc);

    remove_invalid_transform(doc);
    remove_invisible_elements(doc);

    prepare_clip_path_children(doc);

    ungroup_groups(doc, svg, opt);
    regroup_elements(doc, svg);

    prepare_text_nodes(doc);
    remove_invalid_font_size(doc);
}
