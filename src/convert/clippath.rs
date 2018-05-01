// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    self,
    FilterSvg,
};

// self
use tree;
use short::{
    AId,
    EId,
};
use traits::{
    GetValue,
};
use super::{
    path,
    text,
    shapes,
};


pub fn convert(
    node: &svgdom::Node,
    rtree: &mut tree::Tree,
) -> tree::Node {
    let attrs = node.attributes();

    rtree.append_to_defs(
        tree::NodeKind::ClipPath(tree::ClipPath {
            id: node.id().clone(),
            units: super::convert_element_units(&attrs, AId::ClipPathUnits),
            transform: attrs.get_transform(AId::Transform).unwrap_or_default(),
        })
    )
}

pub fn convert_children(
    node: &svgdom::Node,
    parent: &tree::Node,
    rtree: &mut tree::Tree,
) {
    for (id, node) in node.children().svg() {
        match id {
              EId::Rect
            | EId::Polyline
            | EId::Polygon
            | EId::Circle
            | EId::Ellipse => {
                if let Some(d) = shapes::convert(&node) {
                    path::convert(&node, d, parent.clone(), rtree);
                }
            }
            EId::Path => {
                let attrs = node.attributes();
                if let Some(d) = attrs.get_path(AId::D) {
                    path::convert(&node, d.clone(), parent.clone(), rtree);
                }
            }
            EId::Text => {
                text::convert(&node, parent.clone(), rtree);
            }
            EId::Line => {
                // `line` doesn't impact rendering because stroke is always disabled
                // for `clipPath` children.
                // So we can ignore it completely.
            }
            _ => {
                warn!("Skipping the '{}' clipPath invalid child element '{}'.",
                      node.id(), id);
                continue;
            }
        }
    }
}
