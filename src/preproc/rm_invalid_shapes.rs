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
    EId,
};
use traits::{
    GetValue,
};


// TODO: what to do with zero length lines, paths, polylines, polygons?

pub fn remove_invalid_shapes(doc: &mut Document) {
    let mut rm_nodes = Vec::new();

    for (id, node) in doc.descendants().svg() {
        let rm = match id {
            EId::Rect => rm_rect(&node),
            EId::Polyline | EId::Polygon => rm_poly(id, &node),
            EId::Circle => rm_circle(&node),
            EId::Ellipse => rm_ellipse(&node),
            EId::Path => rm_path(&node),
            _ => false,
        };

        if rm {
            rm_nodes.push(node.clone());
        }
    }

    rm_nodes.iter_mut().for_each(|n| n.remove());
}

fn rm_rect(node: &Node) -> bool {
    let attrs = node.attributes();

    // 'width' and 'height' attributes must be positive and non-zero.
    //
    // e-rect-007.svg
    // e-rect-008.svg
    // e-rect-009.svg
    // e-rect-010.svg
    // e-rect-011.svg
    // e-rect-012.svg
    let width  = attrs.get_number(AId::Width).unwrap_or(0.0);
    let height = attrs.get_number(AId::Height).unwrap_or(0.0);
    guard_warn!(width > 0.0, true, "Rect '{}' has an invalid 'width' value. Removed.", node.id());
    guard_warn!(height > 0.0, true, "Rect '{}' has an invalid 'height' value. Removed.", node.id());

    false
}

fn rm_poly(eid: EId, node: &Node) -> bool {
    let attrs = node.attributes();
    let points = if let Some(p) = attrs.get_points(AId::Points) {
        p
    } else {
        warn!("{} '{}' has an invalid 'points' value. Removed.", eid, node.id());
        return true;
    };

    guard_warn!(points.len() >= 2, true, "{} '{}' has less than 2 points. Removed.",
                eid, node.id());

    false
}

fn rm_circle(node: &Node) -> bool {
    let attrs = node.attributes();
    let r = attrs.get_number(AId::R).unwrap_or(0.0);
    guard_warn!(r > 0.0, true, "Circle '{}' has an invalid 'r' value. Removed.", node.id());

    false
}

fn rm_ellipse(node: &Node) -> bool {
    let attrs = node.attributes();
    let rx = attrs.get_number(AId::Rx).unwrap_or(0.0);
    let ry = attrs.get_number(AId::Ry).unwrap_or(0.0);

    guard_warn!(rx > 0.0, true, "Ellipse '{}' has an invalid 'rx' value. Removed.", node.id());
    guard_warn!(ry > 0.0, true, "Ellipse '{}' has an invalid 'ry' value. Removed.", node.id());

    false
}

fn rm_path(node: &Node) -> bool {
    let attrs = node.attributes();

    if let Some(d) = attrs.get_path(AId::D) {
        guard_warn!(d.len() >= 2, true, "Path '{}' has less than 2 segments. Removed.", node.id());
    } else {
        return true;
    }

    false
}
