// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::prelude::*;


/// Removes well-defined, but invisible, elements.
pub fn remove_invisible_elements(doc: &mut Document) {
    rm_display_none(doc);
    rm_zero_opacity(doc);

    // Since 'svgdom' automatically removes (Func)IRI attributes
    // from linked elements, 'use' elements may became obsolete, because
    // a 'use' element without 'xlink:href' is invalid.
    rm_use(doc);
}

fn rm_display_none(doc: &mut Document) {
    // a-display-001.svg

    // From the SVG spec:
    //
    // The `display` property does not apply to the `clipPath` element;
    // thus, `clipPath` elements are not directly rendered even if the `display` property
    // is set to a value other than none, and `clipPath` elements are
    // available for referencing even when the `display` property on the
    // `clipPath` element or any of its ancestors is set to `none`.

    let root = doc.root();
    doc.drain(root, |n| {
        if let Some(&AValue::None) = n.attributes().get_value(AId::Display) {
            if !n.is_tag_name(EId::ClipPath) {
                return true;
            }
        }

        false
    });
}

fn rm_zero_opacity(doc: &mut Document) {
    // Remove elements with opacity="0".
    //
    // Remove only unused elements that are not inside the `defs` element.

    let root = doc.root();
    doc.drain(root, |n| {
        if !n.is_used() {
            if let Some(&AValue::Number(opacity)) = n.attributes().get_value(AId::Opacity) {
                if opacity.is_fuzzy_zero() {
                    // This check is expensive so run it at last.
                    if !n.ancestors().any(|n| n.is_tag_name(EId::Defs)) {
                        return true;
                    }
                }
            }
        }

        false
    });
}

fn rm_use(doc: &mut Document) {
    fn _rm(doc: &mut Document) -> usize {
        let root = doc.root();
        doc.drain(root, |n| {
            if n.is_tag_name(EId::Use) {
                if !n.has_attribute(("xlink", AId::Href)) {
                    // remove 'use' element without the 'xlink:href' attribute
                    return true;
                } else {
                    // remove 'use' element with invalid 'xlink:href' attribute value
                    let attrs = n.attributes();
                    if let Some(&AValue::Link(_)) = attrs.get_value(("xlink", AId::Href)) {
                        // nothing
                    } else {
                        // NOTE: actually, an attribute with 'String' type is valid
                        // if it contain a path to an external file, like '../img.svg#rect1',
                        // but we don't support external SVG, so we treat it like an invalid
                        return true;
                    }
                }
            }

            false
        })
    }

    // 'use' can be linked to another 'use' and if it was removed
    // the first one will became invalid, so we need to check DOM again.
    // Loop until there are no drained elements.
    while _rm(doc) > 0 {}
}
