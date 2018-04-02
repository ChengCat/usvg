// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// external
use base64;
use svgdom;

// self
use super::*;
use geom::*;
use short::{
    AId,
    EId,
};


// TODO: xml:space

pub fn conv_doc(rtree: &Tree) -> svgdom::Document {
    let mut new_doc = svgdom::Document::new();

    let mut svg = new_doc.create_element(EId::Svg);
    new_doc.append(&svg);

    let svg_node = rtree.svg_node();

    svg.set_attribute((AId::Xmlns, "http://www.w3.org/2000/svg"));
    svg.set_attribute((AId::Width,  svg_node.size.width));
    svg.set_attribute((AId::Height, svg_node.size.height));
    conv_viewbox(&svg_node.view_box, &mut svg);
    svg.set_attribute((("xmlns", AId::Xlink), "http://www.w3.org/1999/xlink"));
    svg.set_attribute((("xmlns", "usvg"), "https://github.com/RazrFalcon/usvg"));
    svg.set_attribute((("usvg", "version"), env!("CARGO_PKG_VERSION")));

    let mut defs = new_doc.create_element(EId::Defs);
    svg.append(&defs);

    conv_defs(rtree, &mut new_doc, &mut defs);
    conv_elements(rtree, &rtree.root(), &defs, &mut new_doc, &mut svg);

    new_doc
}

fn conv_defs(
    rtree: &Tree,
    new_doc: &mut svgdom::Document,
    defs: &mut svgdom::Node,
) {
    let mut later_nodes = Vec::new();

    for n in rtree.defs().children() {
        match **n.borrow() {
            NodeKind::LinearGradient(ref lg) => {
                let mut grad_elem = new_doc.create_element(EId::LinearGradient);
                defs.append(&grad_elem);

                grad_elem.set_id(lg.id.clone());

                grad_elem.set_attribute((AId::X1, lg.x1));
                grad_elem.set_attribute((AId::Y1, lg.y1));
                grad_elem.set_attribute((AId::X2, lg.x2));
                grad_elem.set_attribute((AId::Y2, lg.y2));

                conv_base_grad(&n, &lg.d, new_doc, &mut grad_elem);
            }
            NodeKind::RadialGradient(ref rg) => {
                let mut grad_elem = new_doc.create_element(EId::RadialGradient);
                defs.append(&grad_elem);

                grad_elem.set_id(rg.id.clone());

                grad_elem.set_attribute((AId::Cx, rg.cx));
                grad_elem.set_attribute((AId::Cy, rg.cy));
                grad_elem.set_attribute((AId::R, rg.r));
                grad_elem.set_attribute((AId::Fx, rg.fx));
                grad_elem.set_attribute((AId::Fy, rg.fy));

                conv_base_grad(&n, &rg.d, new_doc, &mut grad_elem);
            }
            NodeKind::ClipPath(ref clip) => {
                let mut clip_elem = new_doc.create_element(EId::ClipPath);
                defs.append(&clip_elem);

                clip_elem.set_id(clip.id.clone());
                conv_units(AId::ClipPathUnits, clip.units, &mut clip_elem);
                conv_transform(AId::Transform, &clip.transform, &mut clip_elem);
                later_nodes.push((n.clone(), clip_elem.clone()));
            }
            NodeKind::Pattern(ref pattern) => {
                let mut pattern_elem = new_doc.create_element(EId::Pattern);
                defs.append(&pattern_elem);

                pattern_elem.set_id(pattern.id.clone());

                conv_rect(pattern.rect, &mut pattern_elem);

                if let Some(vbox) = pattern.view_box {
                    conv_viewbox(&vbox, &mut pattern_elem);
                }

                conv_units(AId::PatternUnits, pattern.units, &mut pattern_elem);
                conv_units(AId::PatternContentUnits, pattern.content_units, &mut pattern_elem);
                conv_transform(AId::PatternTransform, &pattern.transform, &mut pattern_elem);
                later_nodes.push((n.clone(), pattern_elem.clone()));
            }
            _ => {}
        }
    }

    for (rnode, mut elem) in later_nodes {
        conv_elements(rtree, &rnode, defs, new_doc, &mut elem);
    }
}

fn conv_elements(
    rtree: &Tree,
    root: &Node,
    defs: &svgdom::Node,
    new_doc: &mut svgdom::Document,
    parent: &mut svgdom::Node,
) {
    let base64_conf = base64::Config::new(
        base64::CharacterSet::Standard,
        true,
        true,
        base64::LineWrap::Wrap(64, base64::LineEnding::LF),
    );

    for n in root.children() {
        match *n.kind() {
            NodeKind::Path(ref p) => {
                let mut path_elem = new_doc.create_element(EId::Path);
                parent.append(&path_elem);

                conv_transform(AId::Transform, &p.transform, &mut path_elem);
                path_elem.set_id(p.id.clone());

                use svgdom::path::Path as SvgDomPath;
                use svgdom::path::Segment;

                let mut path = SvgDomPath::with_capacity(p.segments.len());
                for seg in &p.segments {
                    match *seg {
                        PathSegment::MoveTo { x, y } => {
                            path.push(Segment::new_move_to(x, y));
                        }
                        PathSegment::LineTo { x, y } => {
                            path.push(Segment::new_line_to(x, y));
                        }
                        PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                            path.push(Segment::new_curve_to(x1, y1, x2, y2, x, y));
                        }
                        PathSegment::ClosePath => {
                            path.push(Segment::new_close_path());
                        }
                    }
                }

                path_elem.set_attribute((AId::D, path));

                conv_fill(rtree, &p.fill, defs, parent, &mut path_elem);
                conv_stroke(rtree, &p.stroke, defs, &mut path_elem);
            }
            NodeKind::Text(ref text) => {
                let mut text_elem = new_doc.create_element(EId::Text);
                parent.append(&text_elem);

                conv_transform(AId::Transform, &text.transform, &mut text_elem);
                text_elem.set_id(text.id.clone());

                // conv_text_decoration(&text.decoration, &mut text_elem);

                for chunk_node in n.children() {
                    if let NodeKind::TextChunk(ref chunk) = *chunk_node.kind() {
                        let mut chunk_tspan_elem = new_doc.create_element(EId::Tspan);
                        text_elem.append(&chunk_tspan_elem);

                        chunk_tspan_elem.set_attribute((AId::X, chunk.x));
                        chunk_tspan_elem.set_attribute((AId::Y, chunk.y));

                        if chunk.anchor != TextAnchor::Start {
                            chunk_tspan_elem.set_attribute((AId::TextAnchor,
                                match chunk.anchor {
                                    TextAnchor::Start => svgdom::ValueId::Start,
                                    TextAnchor::Middle => svgdom::ValueId::Middle,
                                    TextAnchor::End => svgdom::ValueId::End,
                                }
                            ));
                        }

                        for tspan_node in chunk_node.children() {
                            if let NodeKind::TSpan(ref tspan) = *tspan_node.kind() {
                                let mut tspan_elem = new_doc.create_element(EId::Tspan);
                                chunk_tspan_elem.append(&tspan_elem);

                                let text_node = new_doc.create_node(
                                    svgdom::NodeType::Text,
                                    &tspan.text,
                                );
                                tspan_elem.append(&text_node);

                                conv_fill(rtree, &tspan.fill, defs, parent, &mut tspan_elem);
                                conv_stroke(rtree, &tspan.stroke, defs, &mut tspan_elem);
                                conv_font(&tspan.font, &mut tspan_elem);

                                // TODO: text-decoration
                            }
                        }
                    }
                }
            }
            NodeKind::Image(ref img) => {
                let mut img_elem = new_doc.create_element(EId::Image);
                parent.append(&img_elem);

                conv_transform(AId::Transform, &img.transform, &mut img_elem);
                img_elem.set_id(img.id.clone());
                conv_viewbox2(&img.view_box, &mut img_elem);

                let href = match img.data {
                    ImageData::Path(ref path) => path.to_str().unwrap().to_owned(),
                    ImageData::Raw(ref data, kind) => {
                        let mut d = String::with_capacity(data.len() + 20);

                        d.push_str("data:image/");
                        match kind {
                            ImageDataKind::PNG => d.push_str("png"),
                            ImageDataKind::JPEG => d.push_str("jpg"),
                        }
                        d.push_str(";base64,\n");
                        d.push_str(&base64::encode_config(data, base64_conf));

                        d
                    }
                };

                img_elem.set_attribute((("xlink", AId::Href), href));
            }
            NodeKind::Group(ref g) => {
                let mut g_elem = new_doc.create_element(EId::G);
                parent.append(&g_elem);

                conv_transform(AId::Transform, &g.transform, &mut g_elem);
                g_elem.set_id(g.id.clone());

                if let Some(ref id) = g.clip_path {
                    if let Some(node) = rtree.defs_by_id(id) {
                        let defs_id = node.id();
                        let link = defs.children().find(|n| *n.id() == *defs_id).unwrap();
                        g_elem.set_attribute((AId::ClipPath, link));
                    }
                }

                if let Some(opacity) = g.opacity {
                    g_elem.set_attribute((AId::Opacity, opacity.value()));
                }

                if !g_elem.has_id() && g_elem.attributes().len() == 0 {
                    warn!("Group must have at least one attribute otherwise it's pointless.");
                }

                conv_elements(rtree, &n, defs, new_doc, &mut g_elem);
            }
            _ => {}
        }
    }
}

fn conv_viewbox(
    view_box: &ViewBox,
    node: &mut svgdom::Node,
) {
    let vb = svgdom::ViewBox::new(
        view_box.rect.origin.x, view_box.rect.origin.y,
        view_box.rect.size.width, view_box.rect.size.height,
    );
    node.set_attribute((AId::ViewBox, vb));

    node.set_attribute((AId::PreserveAspectRatio, view_box.aspect));
}

fn conv_rect(
    r: Rect,
    node: &mut svgdom::Node,
) {
    node.set_attribute((AId::X, r.origin.x));
    node.set_attribute((AId::Y, r.origin.y));
    node.set_attribute((AId::Width, r.size.width));
    node.set_attribute((AId::Height, r.size.height));
}

fn conv_viewbox2(
    vb: &ViewBox,
    node: &mut svgdom::Node,
) {
    conv_rect(vb.rect, node);
    node.set_attribute((AId::PreserveAspectRatio, vb.aspect));
}

fn conv_fill(
    rtree: &Tree,
    fill: &Option<Fill>,
    defs: &svgdom::Node,
    parent: &svgdom::Node,
    node: &mut svgdom::Node,
) {
    match *fill {
        Some(ref fill) => {
            match fill.paint {
                Paint::Color(c) => node.set_attribute((AId::Fill, c)),
                Paint::Link(ref id) => {
                    if let Some(defs_node) = rtree.defs_by_id(id) {
                        let defs_id = defs_node.id();
                        let link = defs.children().find(|n| *n.id() == *defs_id).unwrap();
                        node.set_attribute((AId::Fill, link));
                    }
                }
            }

            node.set_attribute((AId::FillOpacity, fill.opacity.value()));

            if parent.is_tag_name(EId::ClipPath) {
                node.set_attribute((AId::ClipRule, svgdom::ValueId::Evenodd));
            } else {
                node.set_attribute((AId::FillRule, svgdom::ValueId::Evenodd));
            }
        }
        None => {
            node.set_attribute((AId::Fill, svgdom::ValueId::None));
        }
    }
}

fn conv_stroke(
    rtree: &Tree,
    stroke: &Option<Stroke>,
    defs: &svgdom::Node,
    node: &mut svgdom::Node,
) {
    match *stroke {
        Some(ref stroke) => {
            match stroke.paint {
                Paint::Color(c) => node.set_attribute((AId::Stroke, c)),
                Paint::Link(ref id) => {
                    if let Some(defs_node) = rtree.defs_by_id(id) {
                        let defs_id = defs_node.id();
                        let link = defs.children().find(|n| *n.id() == *defs_id).unwrap();
                        node.set_attribute((AId::Stroke, link));
                    }
                }
            }

            node.set_attribute((AId::StrokeOpacity, stroke.opacity.value()));
            node.set_attribute((AId::StrokeDashoffset, stroke.dashoffset));
            node.set_attribute((AId::StrokeMiterlimit, stroke.miterlimit));
            node.set_attribute((AId::StrokeWidth, stroke.width));

            node.set_attribute((AId::StrokeLinecap,
                match stroke.linecap {
                    LineCap::Butt => svgdom::ValueId::Butt,
                    LineCap::Round => svgdom::ValueId::Round,
                    LineCap::Square => svgdom::ValueId::Square,
                }
            ));

            node.set_attribute((AId::StrokeLinejoin,
                match stroke.linejoin {
                    LineJoin::Miter => svgdom::ValueId::Miter,
                    LineJoin::Round => svgdom::ValueId::Round,
                    LineJoin::Bevel => svgdom::ValueId::Bevel,
                }
            ));

            if let Some(ref array) = stroke.dasharray {
                node.set_attribute((AId::StrokeDasharray, array.clone()));
            } else {
                node.set_attribute((AId::StrokeDasharray, svgdom::ValueId::None));
            }
        }
        None => {
            node.set_attribute((AId::Stroke, svgdom::ValueId::None));
        }
    }
}

fn conv_base_grad(
    g_node: &Node,
    g: &BaseGradient,
    doc: &mut svgdom::Document,
    node: &mut svgdom::Node,
) {
    conv_units(AId::GradientUnits, g.units, node);

    node.set_attribute((AId::SpreadMethod,
        match g.spread_method {
            SpreadMethod::Pad => svgdom::ValueId::Pad,
            SpreadMethod::Reflect => svgdom::ValueId::Reflect,
            SpreadMethod::Repeat => svgdom::ValueId::Repeat,
        }
    ));

    conv_transform(AId::GradientTransform, &g.transform, node);

    for n in g_node.children() {
        if let NodeKind::Stop(s) = *n.kind() {
            let mut stop = doc.create_element(EId::Stop);
            node.append(&stop);

            stop.set_attribute((AId::Offset, s.offset.value()));
            stop.set_attribute((AId::StopColor, s.color));
            stop.set_attribute((AId::StopOpacity, s.opacity.value()));
        }
    }
}

fn conv_units(
    aid: AId,
    units: Units,
    node: &mut svgdom::Node,
) {
    node.set_attribute((aid,
        match units {
            Units::UserSpaceOnUse => svgdom::ValueId::UserSpaceOnUse,
            Units::ObjectBoundingBox => svgdom::ValueId::ObjectBoundingBox,
        }
    ));
}

fn conv_transform(
    aid: AId,
    ts: &svgdom::Transform,
    node: &mut svgdom::Node,
) {
    if !ts.is_default() {
        node.set_attribute((aid, *ts));
    }
}

fn conv_font(
    font: &Font,
    node: &mut svgdom::Node,
) {
    node.set_attribute((AId::FontFamily, font.family.clone()));
    node.set_attribute((AId::FontSize, font.size));

    node.set_attribute((AId::FontStyle,
        match font.style {
            FontStyle::Normal => svgdom::ValueId::Normal,
            FontStyle::Italic => svgdom::ValueId::Italic,
            FontStyle::Oblique => svgdom::ValueId::Oblique,
        }
    ));

    node.set_attribute((AId::FontVariant,
        match font.variant {
            FontVariant::Normal => svgdom::ValueId::Normal,
            FontVariant::SmallCaps => svgdom::ValueId::SmallCaps,
        }
    ));

    node.set_attribute((AId::FontWeight,
        match font.weight {
            FontWeight::W100 => svgdom::ValueId::N100,
            FontWeight::W200 => svgdom::ValueId::N200,
            FontWeight::W300 => svgdom::ValueId::N300,
            FontWeight::W400 => svgdom::ValueId::N400,
            FontWeight::W500 => svgdom::ValueId::N500,
            FontWeight::W600 => svgdom::ValueId::N600,
            FontWeight::W700 => svgdom::ValueId::N700,
            FontWeight::W800 => svgdom::ValueId::N800,
            FontWeight::W900 => svgdom::ValueId::N900,
        }
    ));

    node.set_attribute((AId::FontStretch,
        match font.stretch {
            FontStretch::Normal => svgdom::ValueId::Normal,
            FontStretch::Wider => svgdom::ValueId::Wider,
            FontStretch::Narrower => svgdom::ValueId::Narrower,
            FontStretch::UltraCondensed => svgdom::ValueId::UltraCondensed,
            FontStretch::ExtraCondensed => svgdom::ValueId::ExtraCondensed,
            FontStretch::Condensed => svgdom::ValueId::Condensed,
            FontStretch::SemiCondensed => svgdom::ValueId::SemiCondensed,
            FontStretch::SemiExpanded => svgdom::ValueId::SemiExpanded,
            FontStretch::Expanded => svgdom::ValueId::Expanded,
            FontStretch::ExtraExpanded => svgdom::ValueId::ExtraExpanded,
            FontStretch::UltraExpanded => svgdom::ValueId::UltraExpanded,
        }
    ));
}
