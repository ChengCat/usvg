// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    Attribute,
    Document,
    FilterSvg,
    FilterSvgAttrs,
    Node,
    Transform,
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

use {
    Options,
};


pub fn ungroup_groups(doc: &mut Document, svg: &Node, opt: &Options) {
    let mut groups = Vec::with_capacity(16);

    loop {
        _ungroup_groups(svg, opt, &mut groups);

        if groups.is_empty() {
            break;
        }

        while let Some(mut g) = groups.pop() {
            ungroup_group(&mut g);
            doc.remove_node(g);
        }
    }
}

fn _ungroup_groups(parent: &Node, opt: &Options, groups: &mut Vec<Node>) {
    for node in parent.children() {
        if node.has_children() {
            _ungroup_groups(&node, opt, groups);
        }

        if node.is_tag_name(EId::G) {
            if !node.has_children() {
                groups.push(node.clone());
                continue;
            }

            if opt.keep_named_groups && node.has_id() {
                continue;
            }

            // Do not ungroup groups inside `clipPath`.
            // They will be removed during conversion.
            if node.ancestors().skip(1).any(|n| n.is_tag_name(EId::ClipPath)) {
                // Groups that was created from 'use' can be ungroupped.
                if !node.has_attribute("from-use") {
                    continue;
                }
            }

            // Groups with a `clip-path` attribute can't be ungroupped.
            if let Some(&AValue::FuncLink(_)) = node.attributes().get_type(AId::ClipPath) {
                continue;
            }

            // Groups with a `mask` attribute can't be ungroupped.
            if let Some(&AValue::FuncLink(_)) = node.attributes().get_type(AId::Mask) {
                continue;
            }

            // We can ungroup group with opacity only when it has only one child.
            if node.has_attribute(AId::Opacity) {
                if node.children().count() != 1 {
                    continue;
                }
            }

            groups.push(node.clone());
        }
    }
}

fn ungroup_group(g: &mut Node) {
    for (aid, attr) in g.attributes().iter().svg() {
        for (_, mut child) in g.children().svg() {
            if prepare_attribute(&mut child, aid, attr) {
                continue;
            }

            child.set_attribute_if_none((aid, attr.value.clone()));
        }
    }

    let is_single_child = g.children().count() == 1;

    while g.has_children() {
        let mut child = g.last_child().unwrap();
        child.detach();
        g.insert_after(child.clone());

        // Transfer the group ID to the child.
        if is_single_child && !child.has_id() {
            child.set_id(g.id().clone());
        }
    }
}

pub fn prepare_attribute(node: &mut Node, aid: AId, attr: &Attribute) -> bool {
    if aid == AId::Opacity {
        if node.has_attribute(aid) {
            // We can't just replace 'opacity' attribute,
            // we should multiply it.
            let op1 = if let AValue::Number(n) = attr.value { n } else { 1.0 };
            let op2 = node.attributes().get_number(aid).unwrap_or(1.0);
            node.set_attribute((aid, op1 * op2));
            return true;
        }
    }

    if aid == AId::Transform {
        if node.has_attribute(aid) {
            // We should multiply transform matrices.
            let mut t1 = if let AValue::Transform(n) = attr.value {
                n
            } else {
                Transform::default()
            };
            let t2 = node.attributes().get_transform(aid).unwrap_or_default();

            t1.append(&t2);
            node.set_attribute((aid, t1));
            return true;
        }
    }

    if aid == AId::Display {
        // Display attribute has a priority during rendering, so we must
        // copy it even if a child has it already.
        node.set_attribute((aid, attr.value.clone()));
        return true;
    }

    false
}
