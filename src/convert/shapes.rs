// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    self,
    path,
    FuzzyEq
};

// self
use short::{
    AId,
    EId,
};
use traits::{
    GetValue,
};


pub fn convert(node: &svgdom::Node) -> Option<path::Path> {
    match node.tag_id().unwrap() {
        EId::Rect =>     convert_rect(node),
        EId::Line =>     convert_line(node),
        EId::Polyline => convert_polyline(node),
        EId::Polygon =>  convert_polygon(node),
        EId::Circle =>   convert_circle(node),
        EId::Ellipse =>  convert_ellipse(node),
        _ => unreachable!(),
    }
}

// e-rect-001.svg
fn convert_rect(node: &svgdom::Node) -> Option<path::Path> {
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
    if !(width > 0.0) {
        warn!("Rect '{}' has an invalid 'width' value. Skipped.", node.id());
        return None;
    }
    if !(height > 0.0) {
        warn!("Rect '{}' has an invalid 'height' value. Skipped.", node.id());
        return None;
    }


    // e-rect-002.svg
    // e-rect-003.svg
    let x = attrs.get_number(AId::X).unwrap_or(0.0);
    let y = attrs.get_number(AId::Y).unwrap_or(0.0);


    // Resolve rx, ry.
    //
    // e-rect-005.svg
    // e-rect-006.svg
    let mut rx_opt = attrs.get_number(AId::Rx);
    let mut ry_opt = attrs.get_number(AId::Ry);

    // Remove negative values first.
    //
    // e-rect-013.svg
    // e-rect-014.svg
    // e-rect-015.svg
    if let Some(v) = rx_opt {
        if v.is_sign_negative() {
            rx_opt = None;
        }
    }
    if let Some(v) = ry_opt {
        if v.is_sign_negative() {
            ry_opt = None;
        }
    }

    // Resolve.
    let (mut rx, mut ry) = match (rx_opt, ry_opt) {
        (None,     None)     => (0.0, 0.0),
        (Some(rx), None)     => (rx, rx),
        (None,     Some(ry)) => (ry, ry),
        (Some(rx), Some(ry)) => (rx, ry),
    };

    // Clamp rx/ry to the half of the width/height.
    //
    // Should be done only after resolving.
    //
    // e-rect-018.svg
    // e-rect-019.svg
    // e-rect-020.svg
    if rx > width  / 2.0 { rx = width  / 2.0; }
    if ry > height / 2.0 { ry = height / 2.0; }


    // Conversion according to https://www.w3.org/TR/SVG/shapes.html#RectElement
    let path = if rx.fuzzy_eq(&0.0) {
        path::Builder::with_capacity(5)
            .move_to(x, y)
            .hline_to(x + width)
            .vline_to(y + height)
            .hline_to(x)
            .close_path()
            .finalize()
    } else {
        // e-rect-004.svg
        path::Builder::with_capacity(9)
            .move_to(x + rx, y)
            .line_to(x + width - rx, y)
            .arc_to(rx, ry, 0.0, false, true, x + width, y + ry)
            .line_to(x + width, y + height - ry)
            .arc_to(rx, ry, 0.0, false, true, x + width - rx, y + height)
            .line_to(x + rx, y + height)
            .arc_to(rx, ry, 0.0, false, true, x, y + height - ry)
            .line_to(x, y + ry)
            .arc_to(rx, ry, 0.0, false, true, x + rx, y)
            .finalize()
    };

    Some(path)
}

fn convert_line(node: &svgdom::Node) -> Option<path::Path> {
    let attrs = node.attributes();

    let x1 = attrs.get_number(AId::X1).unwrap_or(0.0);
    let y1 = attrs.get_number(AId::Y1).unwrap_or(0.0);
    let x2 = attrs.get_number(AId::X2).unwrap_or(0.0);
    let y2 = attrs.get_number(AId::Y2).unwrap_or(0.0);

    let path = path::Builder::new()
        .move_to(x1, y1)
        .line_to(x2, y2)
        .finalize();

    Some(path)
}

fn convert_polyline(node: &svgdom::Node) -> Option<path::Path> {
    points_to_path(node, "Polyline")
}

fn convert_polygon(node: &svgdom::Node) -> Option<path::Path> {
    if let Some(mut path) = points_to_path(node, "Polygon") {
        path.push(path::Segment::new_close_path());
        Some(path)
    } else {
        None
    }
}

fn points_to_path(node: &svgdom::Node, eid: &str) -> Option<path::Path> {
    let attrs = node.attributes();
    let points = if let Some(p) = attrs.get_points(AId::Points) {
        p
    } else {
        warn!("{} '{}' has an invalid 'points' value. Skipped.", eid, node.id());
        return None;
    };

    // 'polyline' and 'polygon' elements must contain at least 2 points.
    if points.len() < 2 {
        warn!("{} '{}' has less than 2 points. Skipped.", eid, node.id());
        return None;
    }

    let mut path = path::Path::new();
    for (i, &(x, y)) in points.iter().enumerate() {
        let seg = if i == 0 {
            path::Segment::new_move_to(x, y)
        } else {
            path::Segment::new_line_to(x, y)
        };
        path.push(seg);
    }

    Some(path)
}

fn convert_circle(node: &svgdom::Node) -> Option<path::Path> {
    let attrs = node.attributes();

    let cx = attrs.get_number(AId::Cx).unwrap_or(0.0);
    let cy = attrs.get_number(AId::Cy).unwrap_or(0.0);
    let r  = attrs.get_number(AId::R).unwrap_or(0.0);

    if !(r > 0.0) {
        warn!("Circle '{}' has an invalid 'r' value. Skipped.", node.id());
        return None;
    }

    Some(ellipse_to_path(cx, cy, r, r))
}

fn convert_ellipse(node: &svgdom::Node) -> Option<path::Path> {
    let attrs = node.attributes();

    let cx = attrs.get_number(AId::Cx).unwrap_or(0.0);
    let cy = attrs.get_number(AId::Cy).unwrap_or(0.0);
    let rx = attrs.get_number(AId::Rx).unwrap_or(0.0);
    let ry = attrs.get_number(AId::Ry).unwrap_or(0.0);

    if !(rx > 0.0) {
        warn!("Ellipse '{}' has an invalid 'rx' value. Skipped.", node.id());
        return None;
    }

    if !(ry > 0.0) {
        warn!("Ellipse '{}' has an invalid 'ry' value. Skipped.", node.id());
        return None;
    }

    Some(ellipse_to_path(cx, cy, rx, ry))
}

fn ellipse_to_path(cx: f64, cy: f64, rx: f64, ry: f64) -> path::Path {
    path::Builder::with_capacity(6)
        .move_to(cx + rx, cy)
        .arc_to(rx, ry, 0.0, false, true, cx,      cy + ry)
        .arc_to(rx, ry, 0.0, false, true, cx - rx, cy)
        .arc_to(rx, ry, 0.0, false, true, cx,      cy - ry)
        .arc_to(rx, ry, 0.0, false, true, cx + rx, cy)
        .close_path()
        .finalize()
}
