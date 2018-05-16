// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


/// We don't care about `a` elements, but we can't just remove them.
/// So, if an `a` element is inside a `text` - change the tag name to `tspan`.
/// Otherwise, to `g`.
pub fn ungroup_a(doc: &Document) {
    for (id, mut node) in doc.root().descendants().svg() {
        if id != EId::A {
            continue;
        }

        node.remove_attribute(("xlink", AId::Href));

        if node.ancestors().skip(1).any(|n| n.is_tag_name(EId::Text)) {
            node.set_tag_name(EId::Tspan);
        } else {
            node.set_tag_name(EId::G);
        }
    }
}
