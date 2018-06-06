// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path;

// external
use svgdom;
use libflate;

use {
    Error,
};

/// Loads SVG, SVGZ file content.
pub fn load_svg_file(path: &path::Path) -> Result<String, Error> {
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    let mut file = fs::File::open(path).map_err(|_| Error::FileOpenFailed)?;
    let length = file.metadata().map_err(|_| Error::FileOpenFailed)?.len() as usize + 1;

    let ext = if let Some(ext) = Path::new(path).extension() {
        ext.to_str().map(|s| s.to_lowercase()).unwrap_or_default()
    } else {
        String::new()
    };

    match ext.as_str() {
        "svgz" => {
            deflate(&file, length)
        }
        "svg" => {
            let mut s = String::with_capacity(length);
            file.read_to_string(&mut s).map_err(|_| Error::NotAnUtf8Str)?;
            Ok(s)
        }
        _ => {
            Err(Error::InvalidFileSuffix)
        }
    }
}

pub fn deflate<R: ::std::io::Read>(inner: R, len: usize) -> Result<String, Error> {
    use std::io::Read;

    let mut decoder = libflate::gzip::Decoder::new(inner).map_err(|_| Error::MalformedGZip)?;
    let mut decoded = String::with_capacity(len * 2);
    decoder.read_to_string(&mut decoded).map_err(|_| Error::NotAnUtf8Str)?;
    Ok(decoded)
}

/// Parses `svgdom::Document` object from the string data.
pub fn parse_dom(text: &str) -> Result<svgdom::Document, Error> {
    let opt = svgdom::ParseOptions {
        skip_invalid_attributes: true,
        skip_invalid_css: true,
        skip_unresolved_classes: true,
    };

    svgdom::Document::from_str_with_opt(text, &opt)
        .map_err(|e| Error::ParsingFailed(e))
}

