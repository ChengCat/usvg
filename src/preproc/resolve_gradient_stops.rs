// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


pub fn resolve_gradient_stops(doc: &mut Document) {
    let iter = doc.root().descendants()
                  .filter(|n| n.is_gradient())
                  .filter(|n| n.has_attribute(("xlink", AId::Href)))
                  .filter(|n| !n.has_children());
    for mut node in iter {
        let link = node.clone();
        resolve(doc, node.clone(), &link);
    }

    // Remove 'xlink:href' in gradients, because we already resolved everything.
    let iter = doc.root().descendants()
                  .filter(|n| n.is_gradient())
                  .filter(|n| n.has_attribute(("xlink", AId::Href)));
    for mut node in iter {
        node.remove_attribute(("xlink", AId::Href));
    }
}

fn resolve(doc: &mut Document, mut gradient: Node, linked_gradient: &Node) {
    // We can resolve `stop` elements only from gradients.
    if !linked_gradient.is_gradient() {
        return;
    }

    if !linked_gradient.has_children() {
        let av = linked_gradient.attributes().get_value(("xlink", AId::Href)).cloned();
        if let Some(AValue::Link(ref_node)) = av {
            resolve(doc, gradient, &ref_node);
            return;
        }
    }

    for stop in linked_gradient.children() {
        let new_stop = doc.copy_node(stop);
        gradient.append(new_stop);
    }
}
