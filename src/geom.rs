// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::f64;
use std::fmt;

use tree::Transform;

/// Bounds `f64` number.
#[inline]
pub fn f64_bound(min: f64, val: f64, max: f64) -> f64 {
    if val > max {
        return max;
    } else if val < min {
        return min;
    }

    val
}


/// Line representation.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Line {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Line {
    /// Creates a new line.
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Line {
        Line { x1, y1, x2, y2 }
    }

    /// Calculates the line length.
    pub fn length(&self) -> f64 {
        let x = self.x2 - self.x1;
        let y = self.y2 - self.y1;
        (x*x + y*y).sqrt()
    }

    /// Sets the line length.
    pub fn set_length(&mut self, len: f64) {
        let x = self.x2 - self.x1;
        let y = self.y2 - self.y1;
        let len2 = (x*x + y*y).sqrt();
        let line = Line {
            x1: self.x1, y1: self.y1,
            x2: self.x1 + x/len2, y2: self.y1 + y/len2
        };

        self.x2 = self.x1 + (line.x2 - line.x1) * len;
        self.y2 = self.y1 + (line.y2 - line.y1) * len;
    }
}


/// A 2D point representation.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Creates a new `Point` from values.
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

impl From<(f64, f64)> for Point {
    fn from(v: (f64, f64)) -> Self {
        Point::new(v.0, v.1)
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point({} {})", self.x, self.y)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


/// A 2D size representation.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    /// Creates a new `Size` from values.
    pub fn new(width: f64, height: f64) -> Self {
        Size { width, height }
    }

    /// Converts the current size to `Rect` at provided position.
    pub fn to_rect(&self, x: f64, y: f64) -> Rect {
        Rect::new(x, y, self.width, self.height)
    }
}

impl From<(f64, f64)> for Size {
    fn from(v: (f64, f64)) -> Self {
        Size::new(v.0, v.1)
    }
}

impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Size({} {})", self.width, self.height)
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


/// A rect representation.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// Creates a new `Rect` from values.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rect { x, y, width, height }
    }

    /// Returns rect's size.
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Returns transformed rect.
    pub fn transform(&self, ts: Transform) -> Self {
        let (x, y) = ts.apply(self.x, self.y);
        let (sx, sy) = ts.get_scale();
        let (w, h) = (self.width * sx, self.height * sy);
        Self::new(x, y, w, h)
    }
}

impl fmt::Debug for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rect({} {} {} {})", self.x, self.y, self.width, self.height)
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<(f64, f64, f64, f64)> for Rect {
    fn from(v: (f64, f64, f64, f64)) -> Self {
        Rect::new(v.0, v.1, v.2, v.3)
    }
}
