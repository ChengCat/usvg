// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    Document,
    Node,
};

// self
use short::{
    EId,
};


/// Remove `clipPath`'s without children.
///
/// Must be used after invalid and invisible shapes removal.
pub fn remove_invalid_clip_path(doc: &mut Document) {
    let mut rm_nodes = Vec::new();

    for node in doc.descendants().filter(|n| n.is_tag_name(EId::ClipPath)) {
        let mut valid_children = 0;
        for (id, _) in node.children().svg() {
            match id {
                  EId::Rect
                | EId::Polyline
                | EId::Polygon
                | EId::Circle
                | EId::Ellipse
                | EId::Path
                | EId::Text => valid_children += 1,
                _ => {}
            }
        }

        if valid_children == 0 {
            rm_nodes.push(node.clone());
        }
    }

    // If a `clipPath` is invalid than all elements that uses it should be
    // removed too.
    for node in &rm_nodes {
        for mut linked in node.linked_nodes().collect::<Vec<Node>>() {
            linked.remove();
        }
    }

    rm_nodes.iter_mut().for_each(|n| n.remove());
}
