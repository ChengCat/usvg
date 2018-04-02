// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
*usvg* (micro SVG) is an [SVG] simplification tool.

[SVG]: https://en.wikipedia.org/wiki/Scalable_Vector_Graphics
*/

#![forbid(unsafe_code)]
#![warn(missing_docs)]


pub extern crate svgdom;
extern crate base64;
#[macro_use] extern crate log;


pub mod tree;
mod convert;
mod geom;
mod options;
mod preproc;
mod traits;

/// Shorthand names for modules.
mod short {
    pub use svgdom::{
        LengthUnit as Unit,
        ElementId as EId,
        AttributeId as AId,
        AttributeValue as AValue,
    };
}


pub use options::*;
pub use geom::*;

use preproc::{
    DEFAULT_FONT_FAMILY,
    DEFAULT_FONT_SIZE,
};


/// Creates `Tree` from SVG data.
pub fn parse_tree_from_data(
    text: &str,
    opt: &Options,
) -> tree::Tree {
    let doc = parse_dom(text);
    parse_tree_from_dom(doc, opt)
}

/// Creates `Tree` from `svgdom::Document`.
pub fn parse_tree_from_dom(
    mut doc: svgdom::Document,
    opt: &Options,
) -> tree::Tree {
    preproc::prepare_doc(&mut doc, opt);
    convert::convert_doc(&doc, opt)
}

/// Parses `svgdom::Document` object from the string data.
fn parse_dom(text: &str) -> svgdom::Document {
    let opt = svgdom::ParseOptions {
        parse_comments: false,
        parse_declarations: false,
        parse_unknown_elements: false,
        parse_unknown_attributes: false,
        parse_px_unit: false,
        skip_invalid_attributes: true,
        skip_invalid_css: true,
        skip_paint_fallback: true,
        .. svgdom::ParseOptions::default()
    };

    svgdom::Document::from_str_with_opt(text, &opt).unwrap_or_else(|e| {
        warn!("Failed to parse an SVG data cause {}.", e);
        svgdom::Document::new()
    })
}
