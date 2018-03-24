// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path;

// external
use base64;
use svgdom;

// self
use tree::prelude::*;
use short::{
    AId,
    AValue,
};
use traits::{
    GetValue,
};
use {
    Options,
};


pub(super) fn convert(
    node: &svgdom::Node,
    opt: &Options,
    parent: tree::NodeId,
    rtree: &mut tree::Tree,
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

    if let Some(data) = get_href_data(href, opt.path.as_ref()) {
        rtree.append_child(parent, tree::NodeKind::Image(tree::Image {
            id: node.id().clone(),
            transform,
            view_box,
            data,
        }));
    }
}

fn get_href_data(
    href: &str,
    path: Option<&path::PathBuf>,
) -> Option<tree::ImageData> {
    if href.starts_with("data:image/") {
        if let Some(idx) = href.find(',') {
            let start_idx = 11; // data:image/
            let kind = match &href[start_idx..idx] {
                "jpg;base64" | "jpeg;base64" => {
                    tree::ImageDataKind::JPEG
                }
                "png;base64" => {
                    tree::ImageDataKind::PNG
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
                return Some(tree::ImageData::Raw(data.to_owned(), kind));
            }
        }

        warn!("Invalid xlink:href content.");
    } else {
        let path = match path {
            Some(path) => path.parent().unwrap().join(href),
            None => path::PathBuf::from(href),
        };

        if path.exists() {
            if is_valid_image_format(&path) {
                return Some(tree::ImageData::Path(path.to_owned()));
            } else {
                warn!("'{}' is not a PNG or a JPEG image.", href);
            }
        } else {
            warn!("Linked file does not exist: '{}'.", href);
        }
    }

    None
}

/// Checks that file has a PNG or a JPEG magic bytes.
fn is_valid_image_format(path: &path::Path) -> bool {
    use std::fs;
    use std::io::Read;

    macro_rules! try_bool {
        ($e:expr) => {
            match $e {
                Ok(v) => v,
                Err(_) => return false,
            }
        };
    }

    let mut file = try_bool!(fs::File::open(path));

    let mut d = Vec::new();
    d.resize(8, 0);
    try_bool!(file.read_exact(&mut d));

    d.starts_with(b"\x89PNG\r\n\x1a\n") || d.starts_with(&[0xff, 0xd8, 0xff])
}
