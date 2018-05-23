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
    AValue,
    EId,
};


// TODO: xml:space

pub fn conv_doc(rtree: &Tree) -> svgdom::Document {
    let mut new_doc = svgdom::Document::new();

    let mut svg = new_doc.create_element(EId::Svg);
    new_doc.root().append(svg.clone());

    let svg_node = rtree.svg_node();

    svg.set_attribute((AId::Xmlns, "http://www.w3.org/2000/svg"));
    svg.set_attribute((AId::Width,  svg_node.size.width));
    svg.set_attribute((AId::Height, svg_node.size.height));
    conv_viewbox(&svg_node.view_box, &mut svg);
    svg.set_attribute((("xmlns", AId::Xlink), "http://www.w3.org/1999/xlink"));
    svg.set_attribute((("xmlns", "usvg"), "https://github.com/RazrFalcon/usvg"));
    svg.set_attribute((("usvg", "version"), env!("CARGO_PKG_VERSION")));

    let mut defs = new_doc.create_element(EId::Defs);
    svg.append(defs.clone());

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
        match *n.borrow() {
            NodeKind::LinearGradient(ref lg) => {
                let mut grad_elem = new_doc.create_element(EId::LinearGradient);
                defs.append(grad_elem.clone());

                grad_elem.set_id(lg.id.clone());

                grad_elem.set_attribute((AId::X1, lg.x1));
                grad_elem.set_attribute((AId::Y1, lg.y1));
                grad_elem.set_attribute((AId::X2, lg.x2));
                grad_elem.set_attribute((AId::Y2, lg.y2));

                conv_base_grad(&n, &lg.d, new_doc, &mut grad_elem);
            }
            NodeKind::RadialGradient(ref rg) => {
                let mut grad_elem = new_doc.create_element(EId::RadialGradient);
                defs.append(grad_elem.clone());

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
                defs.append(clip_elem.clone());

                clip_elem.set_id(clip.id.clone());
                conv_units(AId::ClipPathUnits, clip.units, &mut clip_elem);
                conv_transform(AId::Transform, &clip.transform, &mut clip_elem);
                later_nodes.push((n.clone(), clip_elem.clone()));
            }
            NodeKind::Mask(ref mask) => {
                let mut mask_elem = new_doc.create_element(EId::Mask);
                defs.append(mask_elem.clone());

                mask_elem.set_id(mask.id.clone());
                conv_units(AId::MaskUnits, mask.units, &mut mask_elem);
                conv_units(AId::MaskContentUnits, mask.content_units, &mut mask_elem);
                conv_rect(mask.rect, &mut mask_elem);
                later_nodes.push((n.clone(), mask_elem.clone()));
            }
            NodeKind::Pattern(ref pattern) => {
                let mut pattern_elem = new_doc.create_element(EId::Pattern);
                defs.append(pattern_elem.clone());

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
        match *n.borrow() {
            NodeKind::Path(ref p) => {
                let mut path_elem = new_doc.create_element(EId::Path);
                parent.append(path_elem.clone());

                conv_transform(AId::Transform, &p.transform, &mut path_elem);
                path_elem.set_id(p.id.clone());

                use svgdom::Path as SvgDomPath;
                use svgdom::PathSegment as SvgDomPathSegment;

                let mut path = SvgDomPath::with_capacity(p.segments.len());
                for seg in &p.segments {
                    match *seg {
                        PathSegment::MoveTo { x, y } => {
                            path.push(SvgDomPathSegment::MoveTo { abs: true, x, y });
                        }
                        PathSegment::LineTo { x, y } => {
                            path.push(SvgDomPathSegment::LineTo { abs: true, x, y });
                        }
                        PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                            path.push(SvgDomPathSegment::CurveTo { abs: true, x1, y1, x2, y2, x, y });
                        }
                        PathSegment::ClosePath => {
                            path.push(SvgDomPathSegment::ClosePath { abs: true });
                        }
                    }
                }

                path_elem.set_attribute((AId::D, path));

                conv_fill(rtree, &p.fill, defs, parent, &mut path_elem);
                conv_stroke(rtree, &p.stroke, defs, &mut path_elem);
            }
            NodeKind::Text(ref text) => {
                let mut text_elem = new_doc.create_element(EId::Text);
                parent.append(text_elem.clone());

                conv_transform(AId::Transform, &text.transform, &mut text_elem);
                text_elem.set_id(text.id.clone());


                if let Some(ref rotate) = text.rotate {
                    text_elem.set_attribute((AId::Rotate, rotate.clone()));
                }

                // conv_text_decoration(&text.decoration, &mut text_elem);

                for chunk_node in n.children() {
                    if let NodeKind::TextChunk(ref chunk) = *chunk_node.borrow() {
                        let mut chunk_tspan_elem = new_doc.create_element(EId::Tspan);
                        text_elem.append(chunk_tspan_elem.clone());

                        if let Some(ref x) = chunk.x {
                            chunk_tspan_elem.set_attribute((AId::X, x.clone()));
                        }

                        if let Some(ref y) = chunk.y {
                            chunk_tspan_elem.set_attribute((AId::Y, y.clone()));
                        }

                        if let Some(ref dx) = chunk.dx {
                            chunk_tspan_elem.set_attribute((AId::Dx, dx.clone()));
                        }

                        if let Some(ref dy) = chunk.dy {
                            chunk_tspan_elem.set_attribute((AId::Dy, dy.clone()));
                        }

                        if chunk.anchor != TextAnchor::Start {
                            chunk_tspan_elem.set_attribute((AId::TextAnchor,
                                match chunk.anchor {
                                    TextAnchor::Start => "start",
                                    TextAnchor::Middle => "middle",
                                    TextAnchor::End => "end",
                                }
                            ));
                        }

                        for tspan_node in chunk_node.children() {
                            if let NodeKind::TSpan(ref tspan) = *tspan_node.borrow() {
                                let mut tspan_elem = new_doc.create_element(EId::Tspan);
                                chunk_tspan_elem.append(tspan_elem.clone());

                                let text_node = new_doc.create_node(
                                    svgdom::NodeType::Text,
                                    tspan.text.clone(),
                                );
                                tspan_elem.append(text_node.clone());

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
                parent.append(img_elem.clone());

                conv_transform(AId::Transform, &img.transform, &mut img_elem);
                img_elem.set_id(img.id.clone());
                conv_viewbox2(&img.view_box, &mut img_elem);

                let href = match img.data {
                    ImageData::Path(ref path) => path.to_str().unwrap().to_owned(),
                    ImageData::Raw(ref data) => {
                        let mut d = String::with_capacity(data.len() + 20);

                        d.push_str("data:image/");
                        match img.format {
                            ImageFormat::PNG => d.push_str("png"),
                            ImageFormat::JPEG => d.push_str("jpg"),
                            ImageFormat::SVG => d.push_str("svg+xml"),
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
                parent.append(g_elem.clone());

                conv_transform(AId::Transform, &g.transform, &mut g_elem);
                g_elem.set_id(g.id.clone());

                if let Some(ref id) = g.clip_path {
                    if let Some(node) = rtree.defs_by_id(id) {
                        let defs_id = node.id();
                        let link = defs.children().find(|n| *n.id() == *defs_id).unwrap();
                        g_elem.set_attribute((AId::ClipPath, link));
                    }
                }

                if let Some(ref id) = g.mask {
                    if let Some(node) = rtree.defs_by_id(id) {
                        let defs_id = node.id();
                        let link = defs.children().find(|n| *n.id() == *defs_id).unwrap();
                        g_elem.set_attribute((AId::Mask, link));
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
    let r = view_box.rect;
    let vb = svgdom::ViewBox::new(r.x, r.y, r.width, r.height);
    node.set_attribute((AId::ViewBox, vb));

    node.set_attribute((AId::PreserveAspectRatio, view_box.aspect));
}

fn conv_rect(
    r: Rect,
    node: &mut svgdom::Node,
) {
    node.set_attribute((AId::X, r.x));
    node.set_attribute((AId::Y, r.y));
    node.set_attribute((AId::Width, r.width));
    node.set_attribute((AId::Height, r.height));
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

            let rule = if fill.rule == FillRule::NonZero { "nonzero" } else { "evenodd" };
            let rule_aid = if parent.is_tag_name(EId::ClipPath) {
                AId::ClipRule
            } else {
                AId::FillRule
            };
            node.set_attribute((rule_aid, rule));
        }
        None => {
            node.set_attribute((AId::Fill, AValue::None));
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
                    LineCap::Butt => "butt",
                    LineCap::Round => "round",
                    LineCap::Square => "square",
                }
            ));

            node.set_attribute((AId::StrokeLinejoin,
                match stroke.linejoin {
                    LineJoin::Miter => "miter",
                    LineJoin::Round => "round",
                    LineJoin::Bevel => "bevel",
                }
            ));

            if let Some(ref array) = stroke.dasharray {
                node.set_attribute((AId::StrokeDasharray, array.clone()));
            } else {
                node.set_attribute((AId::StrokeDasharray, AValue::None));
            }
        }
        None => {
            node.set_attribute((AId::Stroke, AValue::None));
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
            SpreadMethod::Pad => "pad",
            SpreadMethod::Reflect => "reflect",
            SpreadMethod::Repeat => "repeat",
        }
    ));

    conv_transform(AId::GradientTransform, &g.transform, node);

    for n in g_node.children() {
        if let NodeKind::Stop(s) = *n.borrow() {
            let mut stop = doc.create_element(EId::Stop);
            node.append(stop.clone());

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
            Units::UserSpaceOnUse => "userSpaceOnUse",
            Units::ObjectBoundingBox => "objectBoundingBox",
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
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
            FontStyle::Oblique => "oblique",
        }
    ));

    node.set_attribute((AId::FontVariant,
        match font.variant {
            FontVariant::Normal => "normal",
            FontVariant::SmallCaps => "small-caps",
        }
    ));

    node.set_attribute((AId::FontWeight,
        match font.weight {
            FontWeight::W100 => "100",
            FontWeight::W200 => "200",
            FontWeight::W300 => "300",
            FontWeight::W400 => "400",
            FontWeight::W500 => "500",
            FontWeight::W600 => "600",
            FontWeight::W700 => "700",
            FontWeight::W800 => "800",
            FontWeight::W900 => "900",
        }
    ));

    node.set_attribute((AId::FontStretch,
        match font.stretch {
            FontStretch::Normal => "normal",
            FontStretch::Wider => "wider",
            FontStretch::Narrower => "narrower",
            FontStretch::UltraCondensed => "ultra-condensed",
            FontStretch::ExtraCondensed => "extra-condensed",
            FontStretch::Condensed => "condensed",
            FontStretch::SemiCondensed => "semi-condensed",
            FontStretch::SemiExpanded => "semi-expanded",
            FontStretch::Expanded => "expanded",
            FontStretch::ExtraExpanded => "extra-expanded",
            FontStretch::UltraExpanded => "ultra-expanded",
        }
    ));
}
