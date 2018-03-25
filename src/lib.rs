// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
*usvg* (micro SVG) is an [SVG] simplification tool.

[SVG]: https://en.wikipedia.org/wiki/Scalable_Vector_Graphics
*/

#![forbid(unsafe_code)]
#![warn(missing_docs)]


extern crate base64;
extern crate libflate;
extern crate svgdom;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;


macro_rules! guard_warn {
    ($cond:expr, $ret:expr, $msg:expr) => {
        if !$cond {
            warn!($msg);
            return $ret;
        }
    };
    ($cond:expr, $ret:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            warn!($fmt, $($arg)*);
            return $ret;
        }
    };
}

macro_rules! guard_assert {
    ($cond:expr, $ret:expr, $msg:expr) => {
        debug_assert!($cond, $msg);

        if !$cond {
            warn!($msg);
            return $ret;
        }
    };
    ($cond:expr, $ret:expr, $fmt:expr, $($arg:tt)*) => {
        debug_assert!($cond, $fmt, $($arg)*);

        if !$cond {
            warn!($fmt, $($arg)*);
            return $ret;
        }
    };
}


pub mod tree;
mod convert;
mod error;
mod geom;
mod options;
mod preproc;
mod traits;


use std::path::{
    Path,
};

pub use error::{
    Error,
    ErrorKind,
    Result,
};
pub use options::*;
pub use geom::*;

/// Shorthand names for modules.
mod short {
    pub use svgdom::{
        LengthUnit as Unit,
        ElementId as EId,
        AttributeId as AId,
        AttributeValue as AValue,
    };
}

use preproc::{
    DEFAULT_FONT_FAMILY,
    DEFAULT_FONT_SIZE,
};


/// Creates `Tree` from SVG data.
pub fn parse_tree_from_data(
    text: &str,
    opt: &Options,
) -> Result<tree::Tree> {
    let doc = parse_dom(text)?;
    parse_tree_from_dom(doc, opt)
}

/// Creates `Tree` from file.
///
/// `.svg` and `.svgz` files are supported.
pub fn parse_tree_from_file<P: AsRef<Path>>(
    path: P,
    opt: &Options,
) -> Result<tree::Tree> {
    let text = load_file(path.as_ref())?;
    parse_tree_from_data(&text, opt)
}

/// Creates `Tree` from `svgdom::Document`.
pub fn parse_tree_from_dom(
    mut doc: svgdom::Document,
    opt: &Options,
) -> Result<tree::Tree> {
    preproc::prepare_doc(&mut doc, opt)?;
    let rtree = convert::convert_doc(&doc, opt)?;

    Ok(rtree)
}

/// Load an SVG file.
///
/// - `svg` files will be loaded as is.
/// - `svgz` files will be decompressed.
fn load_file(path: &Path) -> Result<String> {
    use std::fs;
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let length = file.metadata()?.len() as usize;

    let ext = if let Some(ext) = Path::new(path).extension() {
        ext.to_str().map(|s| s.to_lowercase()).unwrap_or(String::new())
    } else {
        String::new()
    };

    match ext.as_str() {
        "svgz" => {
            let mut decoder = libflate::gzip::Decoder::new(&file)?;
            let mut decoded = Vec::new();
            decoder.read_to_end(&mut decoded)?;

            Ok(String::from_utf8(decoded)?)
        }
        "svg" => {
            let mut s = String::with_capacity(length + 1);
            file.read_to_string(&mut s)?;
            Ok(s)
        }
        _ => {
            Err(ErrorKind::InvalidFileExtension.into())
        }
    }
}

/// Parses `svgdom::Document` object from the string data.
fn parse_dom(text: &str) -> Result<svgdom::Document> {
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

    let doc = svgdom::Document::from_str_with_opt(&text, &opt)?;
    Ok(doc)
}

/// Converts a provided `svgdom::Document` to `tree::Tree`.
pub fn convert_dom_to_rtree(
    doc: &svgdom::Document,
    opt: &Options,
) -> Result<tree::Tree> {
    convert::convert_doc(doc, opt)
}
