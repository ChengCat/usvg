// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use svgdom::QName;

use super::prelude::*;


pub fn remove_non_svg_data(doc: &mut Document) {
    // Keep only SVG elements and text nodes.
    let root = doc.root().clone();
    doc.drain(root, |n| !n.is_svg_element() && !n.is_text());

    let mut names = Vec::new();
    for (_, mut node) in doc.root().descendants().svg() {
        names.clear();

        // Remove non-SVG attributes.
        for attr in node.attributes().iter() {
            if let QName::Name(_, _) = attr.name {
                names.push(attr.name.clone());
            }
        }

        for name in &names {
            node.remove_attribute(name.as_ref());
        }
    }
}
