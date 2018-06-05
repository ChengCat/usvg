// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use svgdom::{
    self,
    ElementType,
    FilterSvg,
};

// self
use tree;
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
    Options,
};


mod clippath;
mod fill;
mod gradient;
mod image;
mod mask;
mod path;
mod pattern;
mod shapes;
mod stroke;
mod text;

mod prelude {
    pub use svgdom::{
        AttributeType,
        ElementType,
        FilterSvg,
        FilterSvgAttrs,
        FilterSvgAttrsMut,
        FuzzyEq,
        FuzzyZero,
    };
    pub use geom::*;
    pub use short::*;
    pub use traits::*;
    pub use Options;
}


/// Converts an input `Document` to the `Tree`.
///
/// # Errors
///
/// - If `Document` doesn't have an SVG node - returns an empty tree.
/// - If `Document` doesn't have a valid size - it will be set to 100x100.
/// - If `Document` doesn't have a valid viewbox - it will be set to '0 0 W H'.
///
/// Basically, any error, even a critical one, should be recoverable.
/// In worst case scenario return an empty tree, but not an error.
///
/// Must not panic!
pub fn convert_doc(
    svg_doc: &svgdom::Document,
    opt: &Options,
) -> tree::Tree {
    let svg = if let Some(svg) = svg_doc.svg_element() {
        svg
    } else {
        // Can be reached if 'preproc' module has a bug,
        // otherwise document will always have an svg node.
        //
        // Or if someone passed an invalid document directly though API.

        warn!("Invalid SVG structure. An empty tree will be produced.");

        let svg_kind = tree::Svg {
            size: Size::new(100.0, 100.0),
            view_box: tree::ViewBox {
                rect: (0.0, 0.0, 100.0, 100.0).into(),
                aspect: tree::AspectRatio::default(),
            },
        };

        return tree::Tree::create(svg_kind);
    };

    let size = get_img_size(&svg);

    let view_box = {
        let attrs = svg.attributes();
        tree::ViewBox {
            rect: get_view_box(&svg, size),
            aspect: convert_aspect(&attrs),
        }
    };

    let svg_kind = tree::Svg {
        size,
        view_box,
    };

    let mut tree = tree::Tree::create(svg_kind);

    convert_ref_nodes(svg_doc, opt, &mut tree);
    convert_nodes(&svg, tree.root(), opt, &mut tree);

    tree
}

fn convert_ref_nodes(
    svg_doc: &svgdom::Document,
    opt: &Options,
    tree: &mut tree::Tree,
) {
    // e-defs-001.svg

    let defs_elem = try_opt!(svg_doc.defs_element(), ());

    let mut later_nodes = Vec::new();

    for (id, node) in defs_elem.children().svg() {
        // 'defs' can contain any elements, but here we interested only
        // in referenced one.
        if !node.is_referenced() {
            continue;
        }

        match id {
            EId::LinearGradient => {
                gradient::convert_linear(&node, tree);
            }
            EId::RadialGradient => {
                gradient::convert_radial(&node, tree);
            }
            EId::ClipPath => {
                let new_node = clippath::convert(&node, tree);
                later_nodes.push((node, new_node));
            }
            EId::Mask => {
                if let Some(new_node) = mask::convert(&node, tree) {
                    later_nodes.push((node, new_node));
                }
            }
            EId::Pattern => {
                if let Some(new_node) = pattern::convert(&node, tree) {
                    later_nodes.push((node, new_node));
                }
            }
            EId::Symbol => {
                // Already resolved. Skip it.
            }
            _ => {
                // TODO: shapes should be ignored
                // e-defs-005.svg
                warn!("Unsupported element '{}'.", id);
            }
        }
    }

    for (node, mut new_node) in later_nodes {
        if node.is_tag_name(EId::ClipPath) {
            clippath::convert_children(&node, &new_node, tree);

            // e-clipPath-022.svg
            // e-clipPath-016.svg
            // e-clipPath-023.svg
            // e-clipPath-026.svg
            // e-clipPath-024.svg
            // e-clipPath-025.svg
            if !new_node.has_children() {
                warn!("ClipPath '{}' has no children. Skipped.", node.id());
                new_node.detach();
            }
        } else if node.is_tag_name(EId::Mask) {
            convert_nodes(&node, new_node.clone(), opt, tree);

            if !new_node.has_children() {
                warn!("Mask '{}' has no children. Skipped.", node.id());
                new_node.detach();
            }
        } else if node.is_tag_name(EId::Pattern) {
            convert_nodes(&node, new_node.clone(), opt, tree);

            if !new_node.has_children() {
                warn!("Pattern '{}' has no children. Skipped.", node.id());
                new_node.detach();
            }
        }
    }
}

pub(super) fn convert_nodes(
    parent: &svgdom::Node,
    mut parent_node: tree::Node,
    opt: &Options,
    tree: &mut tree::Tree,
) {
    for (id, node) in parent.children().svg() {
        if node.is_referenced() {
            continue;
        }

        match id {
              EId::Title
            | EId::Desc
            | EId::Metadata
            | EId::Defs
            | EId::View => {
                // skip, because pointless
            }
            EId::G => {
                debug_assert!(node.has_children(),
                              "the 'g' element must contain nodes");

                // TODO: maybe move to the separate module

                let attrs = node.attributes();

                // After preprocessing, `clip-path` can be set only on groups.
                //
                // e-clipPath-027.svg
                // e-clipPath-028.svg
                let clip_path = match resolve_iri(&node, EId::ClipPath, AId::ClipPath, tree) {
                    IriResolveResult::Id(id) => Some(id),
                    IriResolveResult::Skip => continue,
                    IriResolveResult::None => None,
                };

                // After preprocessing, `mask` can be set only on groups.
                let mask = match resolve_iri(&node, EId::Mask, AId::Mask, tree) {
                    IriResolveResult::Id(id) => Some(id),
                    IriResolveResult::Skip => continue,
                    IriResolveResult::None => None,
                };

                let ts = attrs.get_transform(AId::Transform).unwrap_or_default();
                let opacity = attrs.get_number(AId::Opacity).map(|v| v.into());

                let g_node = parent_node.append_kind(tree::NodeKind::Group(tree::Group {
                    id: node.id().clone(),
                    transform: ts,
                    opacity,
                    clip_path,
                    mask,
                }));

                convert_nodes(&node, g_node, opt, tree);

                // TODO: check that opacity != 1.0
            }
              EId::Line
            | EId::Rect
            | EId::Polyline
            | EId::Polygon
            | EId::Circle
            | EId::Ellipse => {
                if let Some(d) = shapes::convert(&node) {
                    path::convert(&node, d, parent_node.clone(), tree);
                }
            }
              EId::Use
            | EId::Switch
            | EId::Svg => {
                warn!("'{}' must be already resolved.", id);
            }
            EId::Path => {
                let attrs = node.attributes();
                if let Some(d) = attrs.get_path(AId::D) {
                    path::convert(&node, d.clone(), parent_node.clone(), tree);
                }
            }
            EId::Text => {
                text::convert(&node, parent_node.clone(), tree);
            }
            EId::Image => {
                image::convert(&node, opt, parent_node.clone());
            }
            _ => {
                warn!("Unsupported element '{}'.", id);
            }
        }
    }
}

enum IriResolveResult {
    Id(String),
    Skip,
    None,
}

fn resolve_iri(node: &svgdom::Node, eid: EId, aid: AId, tree: &tree::Tree) -> IriResolveResult {
    let attrs = node.attributes();
    if let Some(&AValue::FuncLink(ref link)) = attrs.get_type(aid) {
        if link.is_tag_name(eid) {
            if let Some(node) = tree.defs_by_id(&link.id()) {
                return IriResolveResult::Id(node.id().to_string());
            } else {
                // If an IRI is invalid than all elements that uses it should be removed/skipped.
                return IriResolveResult::Skip;
            }
        }
    }

    IriResolveResult::None
}

fn get_img_size(svg: &svgdom::Node) -> Size {
    let attrs = svg.attributes();

    let w = attrs.get_number(AId::Width);
    let h = attrs.get_number(AId::Height);

    if let (Some(w), Some(h)) = (w, h) {
        Size::new(w.round(), h.round())
    } else {
        // Can be reached if 'preproc' module has a bug,
        // otherwise document will always have a valid size.
        //
        // Or if someone passed an invalid document directly though API.
        warn!("Invalid SVG size. Reset to 100x100.");
        Size::new(100.0, 100.0)
    }
}

fn get_view_box(svg: &svgdom::Node, size: Size) -> Rect {
    match svg.get_viewbox() {
        Some(vb) => vb,
        None => {
            warn!("Invalid SVG viewBox. Reset to '0 0 {} {}'.", size.width, size.height);
            size.to_rect(0.0, 0.0)
        }
    }
}

fn convert_element_units(attrs: &svgdom::Attributes, aid: AId) -> tree::Units {
    match attrs.get_str(aid) {
        Some("userSpaceOnUse") => tree::Units::UserSpaceOnUse,
        Some("objectBoundingBox") => tree::Units::ObjectBoundingBox,
        _ => {
            warn!("{} must be already resolved.", aid);
            tree::Units::UserSpaceOnUse
        }
    }
}

fn convert_rect(attrs: &svgdom::Attributes) -> Rect {
    (
        attrs.get_number_or(AId::X, 0.0),
        attrs.get_number_or(AId::Y, 0.0),
        attrs.get_number_or(AId::Width, 0.0),
        attrs.get_number_or(AId::Height, 0.0),
    ).into()
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
