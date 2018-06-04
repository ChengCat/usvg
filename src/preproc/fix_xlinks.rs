// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


pub fn fix_xlinks(doc: &Document) {
    // Remove all `xlink:href` that is not a `Link` type.
    // Except `image` element.
    for mut node in doc.root().descendants().filter(|n| !n.is_tag_name(EId::Image)) {
        let av = node.attributes().get_value(("xlink", AId::Href)).cloned();
        if let Some(av) = av {
            match av {
                AValue::Link(_) => {}
                _ => {
                    node.remove_attribute(("xlink", AId::Href));
                }
            }
        }
    }


    // Check that `xlink:href` reference a proper element type.
    for (eid, mut node) in doc.root().descendants().svg() {
        let av = node.attributes().get_value(("xlink", AId::Href)).cloned();
        if let Some(AValue::Link(link)) = av {
            let is_valid = match eid {
                EId::LinearGradient | EId::RadialGradient => link.is_gradient(),
                EId::Pattern => link.is_tag_name(EId::Pattern),
                _ => true,
            };

            if !is_valid {
                node.remove_attribute(("xlink", AId::Href));
            }
        }
    }
}

// Removes all `xlink:href` attributes because we already resolved everything.
pub fn remove_xlinks(doc: &Document) {
    let iter = doc.root().descendants()
                  .filter(|n| !n.is_tag_name(EId::Image))
                  .filter(|n| n.has_attribute(("xlink", AId::Href)));

    for mut node in iter {
        node.remove_attribute(("xlink", AId::Href));
    }
}
