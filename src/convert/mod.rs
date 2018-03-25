// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    self,
    ElementType,
};

// self
use tree::prelude::*;
use short::{
    AId,
    AValue,
    EId,
};
use traits::{
    GetDefsNode,
    GetValue,
    GetViewBox,
};
use geom::*;
use {
    ErrorKind,
    Options,
    Result,
};


mod clippath;
mod fill;
mod gradient;
mod image;
mod path;
mod pattern;
mod shapes;
mod stroke;
mod text;


pub fn convert_doc(
    svg_doc: &svgdom::Document,
    opt: &Options,
) -> Result<tree::Tree> {
    let svg = if let Some(svg) = svg_doc.svg_element() {
        svg
    } else {
        // Can be reached if 'preproc' module has a bug,
        // otherwise document will always have an svg node.
        //
        // Or if someone passed an invalid document directly though API.
        return Err(ErrorKind::MissingSvgNode.into());
    };

    let view_box = {
        let ref attrs = svg.attributes();
        tree::ViewBox {
            rect: get_view_box(&svg)?,
            aspect: convert_aspect(attrs),
        }
    };

    let svg_kind = tree::Svg {
        size: get_img_size(&svg)?,
        view_box,
    };

    let mut rtree = tree::Tree::create(svg_kind);

    convert_ref_nodes(svg_doc, opt, &mut rtree);
    convert_nodes(&svg, rtree.root().id(), opt, &mut rtree);

    Ok(rtree)
}

fn convert_ref_nodes(
    svg_doc: &svgdom::Document,
    opt: &Options,
    rtree: &mut tree::Tree,
) {
    let defs_elem = match svg_doc.defs_element() {
        Some(e) => e.clone(),
        None => return,
    };

    let mut later_nodes = Vec::new();

    for (id, node) in defs_elem.children().svg() {
        // 'defs' can contain any elements, but here we interested only
        // in referenced one.
        if !node.is_referenced() {
            continue;
        }

        match id {
            EId::LinearGradient => {
                gradient::convert_linear(&node, rtree);
            }
            EId::RadialGradient => {
                gradient::convert_radial(&node, rtree);
            }
            EId::ClipPath => {
                let new_node = clippath::convert(&node, rtree);
                later_nodes.push((node, new_node));
            }
            EId::Pattern => {
                let new_node = pattern::convert(&node, rtree);
                later_nodes.push((node, new_node));
            }
            _ => {
                warn!("Unsupported element '{}'.", id);
            }
        }
    }

    for (node, new_node) in later_nodes {
        if node.is_tag_name(EId::ClipPath) {
            clippath::convert_children(&node, new_node, rtree);

            debug_assert!(rtree.get(new_node).has_children(),
                          "clipPath must have at least 1 child");
        } else if node.is_tag_name(EId::Pattern) {
            convert_nodes(&node, new_node, opt, rtree);

            debug_assert!(rtree.get(new_node).has_children(),
                          "pattern must have at least 1 child");
        }
    }
}

pub(super) fn convert_nodes(
    parent: &svgdom::Node,
    parent_node: tree::NodeId,
    opt: &Options,
    rtree: &mut tree::Tree,
) {
    for (id, node) in parent.children().svg() {
        if node.is_referenced() {
            continue;
        }

        match id {
              EId::Title
            | EId::Desc
            | EId::Metadata
            | EId::Defs => {
                // skip, because pointless
            }
            EId::G => {
                debug_assert!(node.has_children(),
                              "the 'g' element must contain nodes");

                // TODO: maybe move to the separate module

                let attrs = node.attributes();

                // After preprocessing, `clip-path` can be set only on groups.
                let clip_path = if let Some(av) = attrs.get_type(AId::ClipPath) {
                    let mut v = None;
                    if let &AValue::FuncLink(ref link) = av {
                        if link.is_tag_name(EId::ClipPath) {
                            if let Some(idx) = rtree.defs_by_svg_id(&link.id()) {
                                v = Some(idx);
                            }
                        }
                    }

                    debug_assert!(v.is_some(), "clipPath must be already resolved");

                    v
                } else {
                    None
                };

                let ts = attrs.get_transform(AId::Transform).unwrap_or_default();
                let opacity = attrs.get_number(AId::Opacity);

                let g_node = rtree.append_child(parent_node, tree::NodeKind::Group(tree::Group {
                    id: node.id().clone(),
                    transform: ts,
                    opacity,
                    clip_path,
                }));

                convert_nodes(&node, g_node, opt, rtree);

                // TODO: check that opacity != 1.0
            }
              EId::Line
            | EId::Rect
            | EId::Polyline
            | EId::Polygon
            | EId::Circle
            | EId::Ellipse => {
                if let Some(d) = shapes::convert(&node) {
                    path::convert(&node, d, parent_node, rtree);
                }
            }
              EId::Use
            | EId::Switch => {
                warn!("'{}' must be resolved.", id);
            }
            EId::Svg => {
                warn!("Nested 'svg' unsupported.");
            }
            EId::Path => {
                let attrs = node.attributes();
                if let Some(d) = attrs.get_path(AId::D) {
                    path::convert(&node, d.clone(), parent_node, rtree);
                }
            }
            EId::Text => {
                text::convert(&node, parent_node, rtree);
            }
            EId::Image => {
                image::convert(&node, opt, parent_node, rtree);
            }
            _ => {
                warn!("Unsupported element '{}'.", id);
            }
        }
    }
}

fn get_img_size(svg: &svgdom::Node) -> Result<Size> {
    let attrs = svg.attributes();

    let w = attrs.get_number(AId::Width);
    let h = attrs.get_number(AId::Height);

    let (w, h) = if let (Some(w), Some(h)) = (w, h) {
        (w, h)
    } else {
        // Can be reached if 'preproc' module has a bug,
        // otherwise document will always have a valid size.
        //
        // Or if someone passed an invalid document directly though API.
        return Err(ErrorKind::InvalidSize.into());
    };

    let size = Size::new(w.round(), h.round());
    Ok(size)
}

fn get_view_box(svg: &svgdom::Node) -> Result<Rect> {
    match svg.get_viewbox() {
        Some(vb) => Ok(vb),
        None => Err(ErrorKind::MissingViewBox.into()),
    }
}

fn convert_element_units(attrs: &svgdom::Attributes, aid: AId) -> tree::Units {
    let av = attrs.get_predef(aid);
    match av {
        Some(svgdom::ValueId::UserSpaceOnUse) => tree::Units::UserSpaceOnUse,
        Some(svgdom::ValueId::ObjectBoundingBox) => tree::Units::ObjectBoundingBox,
        _ => {
            warn!("{} must be already resolved.", aid);
            tree::Units::UserSpaceOnUse
        }
    }
}

fn convert_rect(attrs: &svgdom::Attributes) -> Rect {
    let rect = Rect::new(
        Point::new(
            attrs.get_number(AId::X).unwrap_or(0.0),
            attrs.get_number(AId::Y).unwrap_or(0.0),
        ),
        Size::new(
            attrs.get_number(AId::Width).unwrap_or(0.0),
            attrs.get_number(AId::Height).unwrap_or(0.0),
        ),
    );

    debug_assert!(!rect.size.width.is_fuzzy_zero());
    debug_assert!(!rect.size.height.is_fuzzy_zero());

    rect
}

fn convert_aspect(attrs: &svgdom::Attributes) -> tree::AspectRatio {
    let ratio: Option<&tree::AspectRatio> = attrs.get_type(AId::PreserveAspectRatio);
    match ratio {
        Some(v) => *v,
        None => {
            tree::AspectRatio {
                defer: false,
                align: tree::Align::XMidYMid,
                slice: false,
            }
        }
    }
}
