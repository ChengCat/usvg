// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


// Full list can be found here: https://www.w3.org/TR/SVG/feature.html

static FEATURES: &[&str] = &[
    "http://www.w3.org/TR/SVG11/feature#SVGDOM-static",
    // "http://www.w3.org/TR/SVG11/feature#SVG-static", // not yet
    "http://www.w3.org/TR/SVG11/feature#CoreAttribute", // no xml:base and xml:lang
    "http://www.w3.org/TR/SVG11/feature#Structure",
    "http://www.w3.org/TR/SVG11/feature#BasicStructure",
    // "http://www.w3.org/TR/SVG11/feature#ContainerAttribute", // not yet
    "http://www.w3.org/TR/SVG11/feature#ConditionalProcessing", // no systemLanguage
    "http://www.w3.org/TR/SVG11/feature#Image",
    "http://www.w3.org/TR/SVG11/feature#Style",
    // "http://www.w3.org/TR/SVG11/feature#ViewportAttribute", // not yet
    "http://www.w3.org/TR/SVG11/feature#Shape",
    "http://www.w3.org/TR/SVG11/feature#Text", // partial
    "http://www.w3.org/TR/SVG11/feature#BasicText",
    "http://www.w3.org/TR/SVG11/feature#PaintAttribute", // no color-interpolation and color-rendering
    "http://www.w3.org/TR/SVG11/feature#BasicPaintAttribute", // no color-interpolation
    "http://www.w3.org/TR/SVG11/feature#OpacityAttribute",
    // "http://www.w3.org/TR/SVG11/feature#GraphicsAttribute", // not yet
    "http://www.w3.org/TR/SVG11/feature#BasicGraphicsAttribute",
    // "http://www.w3.org/TR/SVG11/feature#Marker", // not yet
    // "http://www.w3.org/TR/SVG11/feature#ColorProfile", // not yet
    "http://www.w3.org/TR/SVG11/feature#Gradient",
    "http://www.w3.org/TR/SVG11/feature#Pattern",
    "http://www.w3.org/TR/SVG11/feature#Clip",
    "http://www.w3.org/TR/SVG11/feature#Mask",
    // "http://www.w3.org/TR/SVG11/feature#Filter", // not yet
    // "http://www.w3.org/TR/SVG11/feature#BasicFilter", // not yet
    "http://www.w3.org/TR/SVG11/feature#Hyperlinking", // kinda
    "http://www.w3.org/TR/SVG11/feature#XlinkAttribute", // only xlink:href
];

pub fn ungroup_switch(doc: &mut Document) {
    let mut rm_nodes = Vec::with_capacity(16);

    while let Some(mut node) = doc.root().descendants().find(|n| n.is_tag_name(EId::Switch)) {
        let mut valid_child = None;

        // Find first valid node.
        for (_, child) in node.children().svg() {
            if is_valid_child(&child) {
                valid_child = Some(child.clone());
                break;
            }
        }

        let valid_child = match valid_child {
            Some(v) => v,
            None => continue,
        };

        // Remove all invalid nodes.
        for child in node.children().filter(|n| *n != valid_child) {
            rm_nodes.push(child.clone());
        }
        rm_nodes.iter_mut().for_each(|n| doc.remove_node(n.clone()));
        rm_nodes.clear();

        // 'switch' -> 'g'
        node.set_tag_name(EId::G);

        // Remember that this group was 'switch' before.
        node.set_attribute(("usvg-group", 1));
    }
}

fn is_valid_child(node: &Node) -> bool {
    let attrs = node.attributes();

    if attrs.contains(AId::RequiredExtensions) {
        return false;
    }

    // TODO: systemLanguage

    // 'The value is a list of feature strings, with the individual values separated by white space.
    // Determines whether all of the named features are supported by the user agent.
    // Only feature strings defined in the Feature String appendix are allowed.
    // If all of the given features are supported, then the attribute evaluates to true;
    // otherwise, the current element and its children are skipped and thus will not be rendered.'
    if let Some(features) = attrs.get_value(AId::RequiredFeatures) {
        if let AValue::String(ref features) = *features {
            for feature in features.split(' ') {
                if !FEATURES.contains(&feature) {
                    return false;
                }
            }
        }
    }

    true
}
