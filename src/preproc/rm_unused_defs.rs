// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


pub fn remove_unused_defs(doc: &mut Document, svg: &mut Node) {
    remove_unused_defs_impl(doc, svg);
}

fn remove_unused_defs_impl(doc: &mut Document, parent: &mut Node) {
    let mut mv_nodes = Vec::new();
    let mut rm_nodes = Vec::new();

    for mut node in parent.children() {
        if node.is_referenced() && !node.is_used() {
            ungroup_children(&node, &mut mv_nodes, &mut rm_nodes);
        } else if node.has_children() {
            remove_unused_defs_impl(doc, &mut node);
        }
    }

    for node in mv_nodes {
        parent.append(node);
    }

    for node in rm_nodes {
        doc.remove_node(node);
    }
}

fn ungroup_children(node: &Node, mv_nodes: &mut Vec<Node>, rm_nodes: &mut Vec<Node>) {
    if node.has_children() {
        // Element can be unused, but elements in it can be,
        // so we need to move them to parent element before removing.
        for c in node.children() {
            if c.is_used() {
                mv_nodes.push(c.clone());
            }
        }
    }

    rm_nodes.push(node.clone());
}
