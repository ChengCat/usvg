// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


// TODO: mask?

pub fn fix_recursive_pattern(doc: &Document) {
    // e-pattern-021.svg

    // If a pattern child has a link to the pattern itself
    // then we have to replace it with `none`.
    // Otherwise we will get endless loop/recursion and stack overflow.
    for pattern_node in doc.root().descendants().filter(|n| n.is_tag_name(EId::Pattern)) {
        for mut node in pattern_node.descendants() {
            let mut check_attr = |aid: AId| {
                let av = node.attributes().get_value(aid).cloned();
                if let Some(AValue::Paint(link, _)) = av {
                    if link == pattern_node {
                        node.set_attribute((aid, AValue::None));
                    } else {
                        // Check that linked node children doesn't link this pattern.
                        for node2 in link.descendants() {
                            let av2 = node2.attributes().get_value(aid).cloned();
                            if let Some(AValue::Paint(link2, _)) = av2 {
                                if link2 == pattern_node {
                                    node.set_attribute((aid, AValue::None));
                                }
                            }
                        }
                    }
                }
            };

            check_attr(AId::Fill);
            check_attr(AId::Stroke);
        }
    }
}
