// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path;

// external
use base64;
use svgdom;

// self
use tree;
use tree::prelude::*;
use super::prelude::*;


pub fn convert(
    node: &svgdom::Node,
    opt: &Options,
    mut parent: tree::Node,
) {
    let ref attrs = node.attributes();

    let transform = attrs.get_transform(AId::Transform).unwrap_or_default();

    let view_box = tree::ViewBox {
        rect: super::convert_rect(attrs),
        aspect: super::convert_aspect(attrs),
    };

    let href = match attrs.get_value(("xlink", AId::Href)) {
        Some(&AValue::String(ref s)) => s,
        _ => {
            warn!("The 'image' element lacks '{}' attribute. Skipped.", "xlink:href");
            return;
        }
    };

    if let Some((data, format)) = get_href_data(href, opt.path.as_ref()) {
        parent.append_kind(tree::NodeKind::Image(tree::Image {
            id: node.id().clone(),
            transform,
            view_box,
            data,
            format,
        }));
    }
}

fn get_href_data(
    href: &str,
    path: Option<&path::PathBuf>,
) -> Option<(tree::ImageData, tree::ImageFormat)> {
    if href.starts_with("data:image/") {
        if let Some(idx) = href.find(',') {
            let start_idx = 11; // data:image/
            let format = match &href[start_idx..idx] {
                "jpg;base64" | "jpeg;base64" => {
                    tree::ImageFormat::JPEG
                }
                "png;base64" => {
                    tree::ImageFormat::PNG
                }
                "svg+xml;base64" => {
                    tree::ImageFormat::SVG
                }
                _ => {
                    return None;
                }
            };

            let base_data = &href[(idx + 1)..];

            let conf = base64::Config::new(
                base64::CharacterSet::Standard,
                true,
                true,
                base64::LineWrap::NoWrap,
            );

            if let Ok(data) = base64::decode_config(base_data, conf) {
                return Some((tree::ImageData::Raw(data.to_owned()), format));
            }
        }

        warn!("Invalid xlink:href content.");
    } else {
        let path = match path {
            Some(path) => path.parent().unwrap().join(href),
            None => path::PathBuf::from(href),
        };

        if path.exists() {
            if let Some(format) = get_image_format(&path) {
                return Some((tree::ImageData::Path(path::PathBuf::from(href)), format));
            } else {
                warn!("'{}' is not a PNG, JPEG or SVG(Z) image.", href);
            }
        } else {
            warn!("Linked file does not exist: '{}'.", href);
        }
    }

    None
}

fn file_extension(path: &path::Path) -> Option<&str> {
    if let Some(ext) = path.extension() {
        ext.to_str()
    } else {
        None
    }
}

/// Checks that file has a PNG or a JPEG magic bytes.
/// Or SVG(Z) extension.
fn get_image_format(path: &path::Path) -> Option<tree::ImageFormat> {
    use std::fs;
    use std::io::Read;

    let ext = file_extension(path)?.to_lowercase();
    if ext == "svg" || ext == "svgz" {
        return Some(tree::ImageFormat::SVG);
    }

    let mut file = fs::File::open(path).ok()?;

    let mut d = Vec::new();
    d.resize(8, 0);
    file.read_exact(&mut d).ok()?;

    if d.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(tree::ImageFormat::PNG)
    } else if d.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(tree::ImageFormat::JPEG)
    } else {
        None
    }
}
