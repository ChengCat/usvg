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
    AId,
    AValue,
    EId,
};
use traits::{
    GetValue,
};


pub fn resolve_use(doc: &mut Document) {
    let mut nodes = Vec::new();

    // 'use' elements can be linked in any order,
    // so we have to process the tree until all 'use' are solved.
    let mut is_any_resolved = true;
    while is_any_resolved {
        is_any_resolved = false;
        nodes.clear();

        for mut node in doc.root().descendants().filter(|n| n.is_tag_name(EId::Use)) {
            let av = node.attributes().get_value(("xlink", AId::Href)).cloned();
            if let Some(AValue::Link(link)) = av {
                // Ignore 'use' elements linked to other 'use' elements.
                if link.is_tag_name(EId::Use) {
                    continue;
                }

                // We don't support 'use' elements linked to 'svg' element.
                if link.is_tag_name(EId::Svg) {
                    nodes.push(node.clone());
                    continue;
                }

                if link.is_tag_name(EId::Symbol) {
                    nodes.push(node.clone());
                    continue;
                }

                // Check that none of the linked node's children reference current `use` node
                // via other `use` node.
                //
                // Example:
                // <g id="g1">
                //     <use xlink:href="#use1" id="use2"/>
                // </g>
                // <use xlink:href="#g1" id="use1"/>
                //
                // `use2` should be removed.
                let mut is_recursive = false;
                for link_child in link.descendants().skip(1).filter(|n| n.is_tag_name(EId::Use)) {
                    let av = link_child.attributes().get_value(("xlink", AId::Href)).cloned();
                    if let Some(AValue::Link(link2)) = av {
                        if link2 == node {
                            is_recursive = true;
                            break;
                        }
                    }
                }

                if is_recursive {
                    warn!("Recursive 'use' detected. '{}' will be deleted.", node.id());
                    nodes.push(node.clone());
                    continue;
                }

                _resolve_use(doc, node.clone(), &link);

                is_any_resolved = true;
            }

            // 'use' elements without 'xlink:href' attribute will be removed
            // by 'remove_invisible_elements()'.
        }

        // Remove unresolved 'use' elements, since there is not need
        // to keep them around and they will be skipped anyway.
        for node in &mut nodes {
            doc.remove_node(node.clone());
        }
    }
}

fn _resolve_use(doc: &mut Document, mut use_node: Node, linked_node: &Node) {
    // Unlink 'use'.
    use_node.remove_attribute(("xlink", AId::Href));

    {
        // 'use' element support 'x', 'y' and 'transform' attributes
        // and we should process them.
        // So we apply translate transform to the linked element transform.

        let mut attrs = use_node.attributes_mut();

        // 'x' or 'y' should be set.
        if attrs.contains(AId::X) || attrs.contains(AId::Y) {
            let x = attrs.get_number(AId::X).unwrap_or(0.0);
            let y = attrs.get_number(AId::Y).unwrap_or(0.0);

            let mut ts = attrs.get_transform(AId::Transform)
                              .unwrap_or_default();

            ts.translate(x, y);

            attrs.insert_from(AId::Transform, ts);
            attrs.remove(AId::X);
            attrs.remove(AId::Y);
        }
    }

    // Create a deep copy of the linked node.
    let mut new_node = doc.copy_node_deep(linked_node.clone());
    use_node.insert_after(new_node.clone());

    // Copy attributes from 'use'.
    for (aid, attr) in use_node.attributes().iter_svg() {
        // Do not replace existing attributes.
        if !new_node.has_visible_attribute(aid) {
            new_node.set_attribute(attr.clone());
        }
    }

    // Copy old ID.
    new_node.set_id(use_node.id().clone());

    // Relink linked nodes to the new node.
    let linked_nodes = use_node.linked_nodes().clone();
    for mut n in linked_nodes {
        n.set_attribute((("xlink", AId::Href), new_node.clone()));
    }

    // Remove resolved 'use'.
    doc.remove_node(use_node);
}
