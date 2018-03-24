// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::PathBuf;


/// Processing options.
pub struct Options {
    /// SVG image path.
    ///
    /// Used to resolve relative image paths.
    pub path: Option<PathBuf>,

    /// Target DPI.
    ///
    /// Impact units conversion.
    pub dpi: f64,

    /// Keep named groups.
    ///
    /// If set to `true`, all non-empty groups with `id` attribute will not
    /// be removed.
    pub keep_named_groups: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            path: None,
            dpi: 96.0,
            keep_named_groups: false,
        }
    }
}
