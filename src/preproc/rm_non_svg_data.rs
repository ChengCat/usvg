// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


pub fn remove_non_svg_data(doc: &mut Document) {
    // Keep only SVG elements and text nodes.
    let root = doc.root().clone();
    doc.drain(root, |n| !n.is_svg_element() && !n.is_text());

    for mut node in doc.root().descendants() {
        if node.is_element() {
            // Remove non-SVG attributes.
            let mut attrs = node.attributes_mut();
            attrs.retain(|attr| attr.is_svg());
        }
    }
}
