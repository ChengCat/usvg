// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use svgdom::{
    Length,
    Transform,
};

use super::prelude::*;


pub fn prepare_use(doc: &Document) {
    for mut node in doc.root().descendants().filter(|n| n.is_tag_name(EId::Use)) {
        node.set_attribute_if_none((AId::X, 0.0));
        node.set_attribute_if_none((AId::Y, 0.0));
        node.set_attribute_if_none((AId::Width,  Length::new(100.0, Unit::Percent)));
        node.set_attribute_if_none((AId::Height, Length::new(100.0, Unit::Percent)));
    }
}

pub fn resolve_use(doc: &mut Document) {
    let mut rm_nodes = Vec::new();

    // 'use' elements can be linked in any order,
    // so we have to process the tree until all 'use' are solved.
    let mut is_any_resolved = true;
    while is_any_resolved {
        is_any_resolved = false;
        rm_nodes.clear();

        for mut node in doc.root().descendants().filter(|n| n.is_tag_name(EId::Use)) {
            let av = node.attributes().get_value(("xlink", AId::Href)).cloned();
            if let Some(AValue::Link(mut link)) = av {
                // Ignore 'use' elements linked to other 'use' elements.
                if link.is_tag_name(EId::Use) {
                    continue;
                }

                // TODO: this
                // We don't support 'use' elements linked to 'svg' element.
                if link.is_tag_name(EId::Svg) {
                    warn!("'use' element linked to an 'svg' element is not supported. Skipped.");
                    rm_nodes.push(node.clone());
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
                    rm_nodes.push(node.clone());
                    continue;
                }

                _resolve_use(doc, &mut node, &mut link);

                is_any_resolved = true;
            }

            // 'use' elements without 'xlink:href' attribute will be removed
            // by 'remove_invisible_elements()'.
        }

        // Remove unresolved 'use' elements, since there is not need
        // to keep them around and they will be skipped anyway.
        for node in &mut rm_nodes {
            doc.remove_node(node.clone());
        }
    }
}

fn _resolve_use(
    doc: &mut Document,
    use_node: &mut Node,
    linked_node: &mut Node,
) {
    // Unlink 'use'.
    use_node.remove_attribute(("xlink", AId::Href));

    use_node.set_tag_name(EId::G);

    // Remember that this group was 'use' before.
    use_node.set_attribute(("from-use", 1));

    // We require original transformation to setup 'clipPath'.
    let orig_ts = use_node.attributes().get_transform(AId::Transform).unwrap_or_default();
    // Remove original transform. It will be resolved later.
    use_node.remove_attribute(AId::Transform);

    {
        // If the `use` element has a non-zero `x` or `y` attributes
        // then we should add their values to
        // the transform (existing or default).
        let x = use_node.attributes().get_number(AId::X).unwrap_or(0.0);
        let y = use_node.attributes().get_number(AId::Y).unwrap_or(0.0);
        if !(x.is_fuzzy_zero() && y.is_fuzzy_zero()) {
            use_node.append_transform(Transform::new(1.0, 0.0, 0.0, 1.0, x, y));
        }
    }

    // TODO: validate linked nodes

    if linked_node.is_tag_name(EId::Symbol) {
        resolve_symbol(doc, use_node, linked_node, orig_ts)
    } else {
        let new_node = doc.copy_node_deep(linked_node.clone());
        use_node.append(new_node);
        use_node.prepend_transform(orig_ts);
    }

    for mut node in use_node.descendants().skip(1) {
        if node.has_attribute("resolved-font-size") {
            let parent = node.parent().unwrap_or(use_node.clone());
            let fs = parent.find_attribute(AId::FontSize).unwrap_or(DEFAULT_FONT_SIZE);
            node.set_attribute((AId::FontSize, fs));
        }
    }
}

fn resolve_symbol(
    doc: &mut Document,
    use_node: &mut Node,
    linked_node: &mut Node,
    orig_ts: Transform,
) {
    // Required for the 'clip_element' method.
    linked_node.copy_attribute_to(AId::Overflow, use_node);

    if linked_node.has_attribute(AId::ViewBox) {
        use_node.copy_attribute_to(AId::Width, linked_node);
        use_node.copy_attribute_to(AId::Height, linked_node);

        if let Some(ts) = linked_node.get_viewbox_transform() {
            use_node.append_transform(ts);
        }
    }

    let new_node = doc.copy_node_deep(linked_node.clone());
    for child in new_node.children() {
        use_node.append(child);
    }

    let new_g_node = super::clip_element::clip_element(doc, use_node);
    if let Some(mut g_node) = new_g_node {
        // If 'clipPath' was created we have to set the original transform
        // to the group that contains 'clip-path' attribute.
        g_node.set_attribute((AId::Transform, orig_ts));
    } else {
        // Set the original transform back to the 'use' itself.
        use_node.prepend_transform(orig_ts);
    }
}
