// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom;

// self
use tree::prelude::*;
use short::{
    AId,
};


pub fn convert(
    node: &svgdom::Node,
    rtree: &mut tree::Tree,
) -> Option<tree::Node> {
    let ref attrs = node.attributes();

    let rect = super::convert_rect(attrs);
    if !(rect.size.width > 0.0 && rect.size.height > 0.0) {
        warn!("Mask '{}' has an invalid size. Skipped.", node.id());
        return None;
    }

    Some(rtree.append_to_defs(tree::NodeKind::Mask(tree::Mask {
        id: node.id().clone(),
        units: super::convert_element_units(attrs, AId::MaskUnits),
        content_units: super::convert_element_units(attrs, AId::MaskContentUnits),
        rect,
    })))
}