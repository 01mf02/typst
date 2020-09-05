//! Basic building blocks of layouts.

use std::fmt::{self, Debug, Formatter};

use fontdock::FaceId;
use ttf_parser::GlyphId;

use crate::geom_old::Size;

/// A collection of absolutely positioned layout elements.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LayoutElements(pub Vec<(Size, LayoutElement)>);

impl LayoutElements {
    /// Create an new empty collection.
    pub fn new() -> Self {
        Self(vec![])
    }

    /// Add an element at a position.
    pub fn push(&mut self, pos: Size, element: LayoutElement) {
        self.0.push((pos, element));
    }

    /// Add all elements of another collection, offsetting each by the given
    /// `offset`. This can be used to place a sublayout at a position in another
    /// layout.
    pub fn extend_offset(&mut self, offset: Size, more: Self) {
        for (subpos, element) in more.0 {
            self.0.push((subpos + offset, element));
        }
    }
}

/// A layout element, the basic building block layouts are composed of.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutElement {
    Text(Shaped),
}

/// A shaped run of text.
#[derive(Clone, PartialEq)]
pub struct Shaped {
    pub text: String,
    pub face: FaceId,
    pub glyphs: Vec<GlyphId>,
    /// The horizontal offsets of the glyphs with the same indices. Vertical
    /// offets are not yet supported.
    pub offsets: Vec<f64>,
    /// The font size.
    pub size: f64,
}

impl Shaped {
    /// Create a new shape run with empty `text`, `glyphs` and `offsets`.
    pub fn new(face: FaceId, size: f64) -> Self {
        Self {
            text: String::new(),
            face,
            glyphs: vec![],
            offsets: vec![],
            size,
        }
    }

    /// Encode the glyph ids into a big-endian byte buffer.
    pub fn encode_glyphs_be(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 * self.glyphs.len());
        for &GlyphId(g) in &self.glyphs {
            bytes.push((g >> 8) as u8);
            bytes.push((g & 0xff) as u8);
        }
        bytes
    }
}

impl Debug for Shaped {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Shaped({})", self.text)
    }
}
