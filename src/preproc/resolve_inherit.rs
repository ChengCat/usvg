// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use svgdom::{
    Attribute,
};

use super::prelude::*;


/// Resolve 'inherit' attributes.
///
/// The function will fallback to a default value when possible.
//
// a-fill-021.svg
pub fn resolve_inherit(doc: &Document) {
    let mut ids = Vec::new();

    for (_, mut node) in doc.root().descendants().svg() {
        ids.clear();

        {
            let attrs = node.attributes();
            for (aid, attr) in attrs.iter().svg() {
                if let AValue::Inherit = attr.value {
                    ids.push(aid);
                }
            }
        }

        for id in &ids {
            resolve_impl(&mut node, *id);
        }
    }
}

fn resolve_impl(node: &mut Node, attr: AId) {
    if let Some(n) = node.ancestors().skip(1).find(|n| n.has_attribute(attr)) {
        let av = n.attributes().get_value(attr).cloned();
        if let Some(av) = av {
            node.set_attribute((attr, av.clone()));
        }
    } else {
        match Attribute::default(attr) {
            Some(a) => node.set_attribute((attr, a.value)),
            None => {
                warn!("Failed to resolve attribute: {}. Removing it.",
                        node.attributes().get(attr).unwrap());
                node.remove_attribute(attr);
            }
        }
    }
}
