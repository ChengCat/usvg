// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    Document,
    FuzzyEq,
    Node,
};

// self
use short::{
    AId,
    EId,
};
use traits::{
    GetValue,
};


// TODO: images should not be grouped
pub fn regroup_elements(doc: &mut Document, parent: &Node) {
    let g_attrs = [AId::ClipPath, AId::Opacity];

    let mut ids = Vec::new();
    let mut curr_node = parent.first_child();
    while let Some(mut node) = curr_node {
        curr_node = node.next_sibling();
        ids.clear();

        if node.has_children() {
            regroup_elements(doc, &node);
        }

        if node.is_tag_name(EId::G) || node.is_tag_name(EId::Defs) {
            continue;
        }

        let opacity = node.attributes().get_number(AId::Opacity).unwrap_or(1.0);
        if opacity.fuzzy_eq(&1.0) && !node.has_attribute(AId::ClipPath) {
            continue;
        }

        // Do not group elements inside the clipPath.
        if node.parents_with_self().any(|n| n.is_tag_name(EId::ClipPath)) {
            continue;
        }

        let mut g_node = doc.create_element(EId::G);

        {
            let attrs = node.attributes();
            for aid in &g_attrs {
                if let Some(attr) = attrs.get(*aid) {
                    g_node.set_attribute(attr.clone());
                    ids.push(*aid);
                }
            }

            if let Some(ts) = attrs.get(AId::Transform) {
                g_node.set_attribute(ts.clone());
                ids.push(AId::Transform);
            }
        }

        for id in &ids {
            node.remove_attribute(*id);
        }

        node.insert_before(&g_node);
        node.detach();
        g_node.append(&node);
    }
}
