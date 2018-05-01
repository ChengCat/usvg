// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    AspectRatio,
    Document,
    FilterSvgAttrs,
    FuzzyEq,
    FuzzyZero,
    Length,
    Node,
};

// self
use short::{
    AId,
    AValue,
    EId,
    Unit,
};
use utils;
use geom::*;
use traits::*;


pub fn prepare_use(doc: &mut Document) {
    for mut node in doc.root().descendants().filter(|n| n.is_tag_name(EId::Use)) {
        node.set_attribute_if_none((AId::X, 0.0));
        node.set_attribute_if_none((AId::Y, 0.0));
        node.set_attribute_if_none((AId::Width,  Length::new(100.0, Unit::Percent)));
        node.set_attribute_if_none((AId::Height, Length::new(100.0, Unit::Percent)));
    }
}

pub fn resolve_use(doc: &mut Document) {
    let mut nodes = Vec::new();
    let mut clip_path_idx = 1;

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

                // TODO: this
                // We don't support 'use' elements linked to 'svg' element.
                if link.is_tag_name(EId::Svg) {
                    warn!("'use' element linked to an 'svg' element is not supported. Skipped.");
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

                _resolve_use(doc, &mut node, &link, &mut clip_path_idx);

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

fn _resolve_use(
    doc: &mut Document,
    use_node: &mut Node,
    linked_node: &Node,
    clip_path_idx: &mut usize,
) {
    // Unlink 'use'.
    use_node.remove_attribute(("xlink", AId::Href));

    {
        // If the `use` element has a non-zero `x` or `y` attributes
        // then we should add their values to
        // the transform (existing or default).

        let mut attrs = use_node.attributes_mut();
        let x = attrs.get_number(AId::X).unwrap_or(0.0);
        let y = attrs.get_number(AId::Y).unwrap_or(0.0);

        if !(x.is_fuzzy_zero() && y.is_fuzzy_zero()) {
            let mut ts = attrs.get_transform(AId::Transform)
                              .unwrap_or_default();
            ts.translate(x, y);
            attrs.insert_from(AId::Transform, ts);
        }
    }

    // TODO: validate linked nodes

    let mut new_node = if linked_node.is_tag_name(EId::Symbol) {
        resolve_symbol(doc, use_node, linked_node, clip_path_idx)
    } else {
        // Create a deep copy of the linked node.
        let new_node = doc.copy_node_deep(linked_node.clone());
        use_node.insert_after(new_node.clone());
        new_node
    };

    // This attributes are no longer needed.
    use_node.remove_attribute(AId::X);
    use_node.remove_attribute(AId::Y);
    use_node.remove_attribute(AId::Width);
    use_node.remove_attribute(AId::Height);

    // Copy attributes from 'use'.
    for (aid, attr) in use_node.attributes().iter().svg() {
        if super::ungroup_groups::prepare_attribute(&mut new_node, aid, attr) {
            continue;
        }

        let is_resolved_font_size =    aid == AId::FontSize
                                    && new_node.has_attribute("resolved-font-size");

        // Do not replace existing attributes.
        if !new_node.has_attribute(aid) || is_resolved_font_size {
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
    doc.remove_node(use_node.clone());
}

fn resolve_symbol(
    doc: &mut Document,
    use_node: &mut Node,
    linked_node: &Node,
    clip_path_idx: &mut usize,
) -> Node {
    let mut g_node = doc.create_element(EId::G);

    if linked_node.has_attribute(AId::ViewBox) {
        prepare_viewbox(&use_node, linked_node, &mut g_node);
    }

    let new_node = doc.copy_node_deep(linked_node.clone());
    for child in new_node.children() {
        g_node.append(child);
    }

    if let Some(clip_rect) = get_clip_rect(doc, use_node) {
        // This `symbol` should be clipped.
        if let Some(mut defs_node) = doc.defs_element() {
            // We can't set `clip-path` on the new group itself,
            // because it will be affected by the transform.
            // So we have to create a new one.
            let mut g_node2 = doc.create_element(EId::G);
            g_node2.append(g_node.clone());

            let mut clip_node = doc.create_element(EId::ClipPath);
            clip_node.set_id(gen_clip_path_id(doc, clip_path_idx));
            defs_node.append(clip_node.clone());

            let mut rect_node = doc.create_element(EId::Rect);

            rect_node.set_attribute((AId::X, clip_rect.origin.x));
            rect_node.set_attribute((AId::Y, clip_rect.origin.y));
            rect_node.set_attribute((AId::Width, clip_rect.size.width));
            rect_node.set_attribute((AId::Height, clip_rect.size.height));
            clip_node.append(rect_node);

            g_node2.set_attribute((AId::ClipPath, clip_node.clone()));

            use_node.insert_after(g_node2);
        } else {
            // Technically unreachable because we should always
            // have the `defs` element.
            use_node.insert_after(g_node.clone());
        }
    } else {
        use_node.insert_after(g_node.clone());
    }

    g_node
}

fn get_clip_rect(doc: &Document, use_node: &Node) -> Option<Rect> {
    let (x, y, w, h) = {
        let use_attrs = use_node.attributes();
        let x = use_attrs.get_number(AId::X)?;
        let y = use_attrs.get_number(AId::Y)?;
        let w = use_attrs.get_number(AId::Width)?;
        let h = use_attrs.get_number(AId::Height)?;
        (x, y, w, h)
    };

    let svg = doc.svg_element()?;
    let svg_w = svg.attributes().get_number(AId::Width)?;
    let svg_h = svg.attributes().get_number(AId::Height)?;

    // Clip rect is not needed when it has the same size as a whole image.
    if w.fuzzy_eq(&svg_w) && h.fuzzy_eq(&svg_h) {
        return None;
    }

    Some(rect(x, y, w, h))
}

/// Creates a free id for `clipPath`.
fn gen_clip_path_id(doc: &Document, clip_path_idx: &mut usize) -> String {
    let mut clip_path_id = format!("clipPath{}", clip_path_idx);
    while doc.root().descendants().any(|n| *n.id() == clip_path_id) {
        *clip_path_idx += 1;
        clip_path_id = format!("clipPath{}", clip_path_idx);
    }

    clip_path_id
}

fn prepare_viewbox(use_node: &Node, linked_node: &Node, g_node: &mut Node) {
    let size = {
        let use_attrs = use_node.attributes();
        let w = try_opt!(use_attrs.get_number(AId::Width), ());
        let h = try_opt!(use_attrs.get_number(AId::Height), ());
        Size::new(w, h)
    };

    let vb = try_opt!(linked_node.get_viewbox(), ());
    let aspect = match linked_node.attributes().get_value(AId::PreserveAspectRatio) {
        Some(&AValue::AspectRatio(aspect)) => aspect,
        _ => AspectRatio::default(),
    };

    let ts = utils::view_box_to_transform(vb, aspect, size);
    g_node.set_attribute((AId::Transform, ts));
}
