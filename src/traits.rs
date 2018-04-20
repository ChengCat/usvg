// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::f64;
use std::fmt::Display;

// external
use svgdom::{
    AspectRatio,
    Attributes,
    Color,
    Document,
    Length,
    LengthList,
    Node,
    NumberList,
    Path,
    Points,
    Transform,
    ViewBox,
};

// self
use short::{
    AId,
    AValue,
    EId,
};
use geom::*;


pub trait GetViewBox {
    fn get_viewbox(&self) -> Option<Rect>;
}

impl GetViewBox for Node {
    fn get_viewbox(&self) -> Option<Rect> {
        self.attributes()
            .get_type::<ViewBox>(AId::ViewBox)
            .map(|vb| Rect::new(Point::new(vb.x, vb.y), Size::new(vb.w, vb.h)))
    }
}


pub trait GetDefsNode {
    fn defs_element(&self) -> Option<Node>;
}

impl GetDefsNode for Document {
    fn defs_element(&self) -> Option<Node> {
        let svg = match self.svg_element() {
            Some(svg) => svg.clone(),
            None => return None,
        };

        match svg.first_child() {
            Some(child) => {
                if child.is_tag_name(EId::Defs) {
                    Some(child.clone())
                } else {
                    warn!("The first child of the 'svg' element should be 'defs'. Found '{:?}' instead.",
                          child.tag_name());
                    None
                }
            }
            None => {
                None
            }
        }
    }
}


pub trait FromValue {
    fn get(v: &AValue) -> Option<&Self>;
}

macro_rules! impl_from_value {
    ($rtype:ty, $etype:ident) => (
        impl FromValue for $rtype {
            fn get(v: &AValue) -> Option<&Self> {
                if let AValue::$etype(ref vv) = *v { Some(vv) } else { None }
            }
        }
    )
}

impl_from_value!(Color, Color);
impl_from_value!(f64, Number);
impl_from_value!(Length, Length);
impl_from_value!(LengthList, LengthList);
impl_from_value!(NumberList, NumberList);
impl_from_value!(Path, Path);
impl_from_value!(Transform, Transform);
impl_from_value!(ViewBox, ViewBox);
impl_from_value!(Points, Points);
impl_from_value!(AspectRatio, AspectRatio);

impl FromValue for str {
    fn get(v: &AValue) -> Option<&Self> {
        match v {
            &AValue::String(ref s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl FromValue for AValue {
    fn get(v: &AValue) -> Option<&Self> {
        Some(v)
    }
}


pub trait GetValue {
    fn get_type<T: FromValue + ?Sized>(&self, id: AId) -> Option<&T>;

    fn get_number(&self, id: AId) -> Option<f64> {
        self.get_type(id).cloned()
    }

    fn get_length(&self, id: AId) -> Option<Length> {
        self.get_type(id).cloned()
    }

    fn get_transform(&self, id: AId) -> Option<Transform> {
        self.get_type(id).cloned()
    }

    fn get_number_list(&self, id: AId) -> Option<&NumberList> {
        self.get_type(id)
    }

    fn get_color(&self, id: AId) -> Option<Color> {
        self.get_type(id).cloned()
    }

    fn get_path(&self, id: AId) -> Option<&Path> {
        self.get_type(id)
    }

    fn get_points(&self, id: AId) -> Option<&Points> {
        self.get_type(id)
    }

    fn get_str(&self, id: AId) -> Option<&str> {
        self.get_type(id)
    }
}

impl GetValue for Attributes {
    fn get_type<T: FromValue + ?Sized>(&self, id: AId) -> Option<&T> {
        match self.get_value(id) {
            Some(av) => {
                FromValue::get(av)
            }
            None => {
                trace!("Type mismatch.");
                None
            }
        }
    }
}


// TODO: remove

pub trait FindAttribute {
    fn find_attribute<T: FromValue + Display + Clone>(&self, id: AId) -> Option<T>;
    fn find_attribute_with_node<T: FromValue + Display + Clone>(&self, id: AId) -> Option<(Node, T)>;
    fn find_node_with_attribute(&self, id: AId) -> Option<Node>;
}

impl FindAttribute for Node {
    fn find_attribute<T: FromValue + Display + Clone>(&self, id: AId) -> Option<T> {
        self.find_attribute_with_node(id).map(|v| v.1)
    }

    fn find_attribute_with_node<T: FromValue + Display + Clone>(&self, id: AId) -> Option<(Node, T)> {
        for n in self.ancestors() {
            if n.has_attribute(id) {
                let v = FromValue::get(n.attributes().get_value(id).unwrap()).cloned();
                return match v {
                    Some(v) => Some((n.clone(), v)),
                    None => None,
                };
            }
        }

        None
    }

    fn find_node_with_attribute(&self, id: AId) -> Option<Node> {
        for n in self.ancestors() {
            if n.has_attribute(id) {
                return Some(n.clone())
            }
        }

        None
    }
}

