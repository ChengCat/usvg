// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


// TODO: Note that if the 'visibility' property is set to hidden on a 'tspan', 'tref' or 'altGlyph'
//       element, then the text is invisible but still takes up space in text layout calculations.

pub fn resolve_visibility(doc: &mut Document, svg: &Node) {
    // a-visibility-001.svg
    // a-visibility-002.svg

    let mut nodes = Vec::with_capacity(16);

    _resolve(svg, &mut nodes);

    // `_resolve` can add duplicated nodes so we should remove them,
    // otherwise `remove_node` will panic.
    nodes.dedup();

    for node in nodes {
        doc.remove_node(node);
    }
}

fn _resolve(parent: &Node, nodes: &mut Vec<Node>) {
    for mut node in parent.children() {
        if node.has_children() {
            _resolve(&node, nodes);
        }

        if node.is_tag_name(EId::G) {
            // From the SVG spec:
            //
            // Setting 'visibility' to hidden on a 'g' will make its children invisible as
            // long as the children do not specify their own 'visibility' properties as visible.
            // Note that 'visibility' is not an inheritable property.
            if !is_hidden(&node) {
                continue;
            }

            node.remove_attribute(AId::Visibility);

            for child in node.children() {
                if child.attributes().get_str(AId::Visibility) != Some("visible") {
                    nodes.push(child.clone());
                }
            }
        } else {
            if is_hidden(&node) {
                nodes.push(node.clone());
            }
        }
    }
}

/// Checks that element has 'visibility' set to 'hidden' or 'collapse'.
fn is_hidden(node: &Node) -> bool {
    match node.attributes().get_str(AId::Visibility) {
        Some("visible") | None => false,
        _ => true,
    }
}
