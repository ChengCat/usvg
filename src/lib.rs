// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
`usvg` (micro SVG) is an [SVG] simplification tool.

## Purpose

Imagine, that you have to extract some data from the [SVG] file, but your
library/framework/language doesn't have a good SVG library.
And all you need is paths data.

You can try to export it by yourself (how hard can it be, right).
All you need is an XML library (I'll hope that your language has one).
But soon you realize that paths data has a pretty complex format and a lot
of edge-cases. And we didn't mention attributes propagation, transforms,
visibility flags, attribute values validation, XML quirks, etc.
It will take a lot of time and code to implement this stuff correctly.

So, instead of creating a library that can be used from any language (impossible),
*usvg* takes a different approach. It converts an input SVG to an extremely
simple representation, which is still a valid SVG.
And now, all you need is to convert your SVG to a simplified one via *usvg*
and an XML library with some small amount of code.

## Key features of the simplified SVG

- No basic shapes (rect, circle, etc). Only paths
- Simple paths:
  - Only MoveTo, LineTo, CurveTo and ClosePath will be produced
  - All path segments are in absolute coordinates
  - No implicit segment commands
  - All values are separated by space
- All (supported) attributes are resolved. No implicit one
- No `use`. Everything is resolved
- No invisible elements
- No invalid elements (like `rect` with negative/zero size)
- No units (mm, em, etc.)
- No comments
- No DTD
- No CSS (partial support)
- No `script` (simply ignoring it)

Full spec can be found [here](https://github.com/RazrFalcon/usvg/blob/master/docs/usvg_spec.adoc).

## Limitations

- Currently, it's not lossless. Some SVG features isn't supported yet and will be ignored.
- CSS support is minimal.
- Scripting and animation isn't supported and not planned.
- `a` elements will be removed.
- Unsupported elements:
  - filter-based elements
  - font-based elements
  - `marker`
  - `symbol`
  - `view`
  - `foreignObject`

[SVG]: https://en.wikipedia.org/wiki/Scalable_Vector_Graphics
*/

#![doc(html_root_url = "https://docs.rs/usvg/0.1.1")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]


pub extern crate svgdom;
extern crate base64;
extern crate libflate;
extern crate lyon_geom;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;


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


use std::path;

pub use options::*;
pub use geom::*;

use preproc::{
    DEFAULT_FONT_FAMILY,
    DEFAULT_FONT_SIZE,
};


/// Parsers `Tree` from SVG data.
pub fn parse_tree_from_data(
    text: &str,
    opt: &Options,
) -> tree::Tree {
    let doc = parse_dom(text);
    parse_tree_from_dom(doc, opt)
}

/// Parsers `Tree` from `svgdom::Document`.
pub fn parse_tree_from_dom(
    mut doc: svgdom::Document,
    opt: &Options,
) -> tree::Tree {
    preproc::prepare_doc(&mut doc, opt);
    convert::convert_doc(&doc, opt)
}

/// List of errors that can be produced by `parse_tree_from_file`.
#[derive(Fail, Debug)]
#[allow(missing_docs)]
pub enum FileReadError {
    #[fail(display = "{}", _0)]
    Io(::std::io::Error),

    #[fail(display = "{}", _0)]
    Utf8(::std::string::FromUtf8Error),
}

impl From<::std::io::Error> for FileReadError {
    fn from(value: ::std::io::Error) -> FileReadError {
        FileReadError::Io(value)
    }
}

impl From<::std::string::FromUtf8Error> for FileReadError {
    fn from(value: ::std::string::FromUtf8Error) -> FileReadError {
        FileReadError::Utf8(value)
    }
}

/// Parsers `Tree` from file.
pub fn parse_tree_from_file<P: AsRef<path::Path>>(
    path: P,
    opt: &Options,
) -> Result<tree::Tree, FileReadError> {
    let text = load_svg_file(path.as_ref())?;
    Ok(parse_tree_from_data(&text, opt))
}

/// Loads SVG, SVGZ file content.
pub fn load_svg_file(path: &path::Path) -> Result<String, FileReadError> {
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    let mut file = fs::File::open(path)?;
    let length = file.metadata()?.len() as usize;

    let ext = if let Some(ext) = Path::new(path).extension() {
        ext.to_str().map(|s| s.to_lowercase()).unwrap_or_default()
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
            unreachable!()
        }
    }
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
        skip_elements_crosslink: true,
        .. svgdom::ParseOptions::default()
    };

    svgdom::Document::from_str_with_opt(text, &opt).unwrap_or_else(|e| {
        warn!("Failed to parse an SVG data cause {}.", e);
        svgdom::Document::new()
    })
}
