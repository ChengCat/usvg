// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


pub fn remove_desc_elements(doc: &mut Document) {
    // Remove all `title` and `desc` elements, because they can pop up everywhere
    // which may lead to wrong results.
    let root = doc.root().clone();
    doc.drain(root, |n|    n.is_tag_name(EId::Title)
                        || n.is_tag_name(EId::Desc)
                        || n.is_tag_name(EId::Metadata));
}
