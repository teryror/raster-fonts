//! De-/Serializable runtime representation of bitmap font metadata.

#![cfg_attr(docs_rs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]

use std::collections::HashMap;
use std::num::NonZeroU8;

/// Coordinates and size of a rendered glyph in a packed bitmap.
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "serde-deserialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive))]
#[cfg_attr(feature = "rkyv-serialize", derive(rkyv::Serialize))]
#[cfg_attr(feature = "rkyv-deserialize", derive(rkyv::Deserialize))]
pub struct SourceRect {
    /// Horizontal position in the bitmap in pixels.
    pub x: u16,
    /// Vertical position in the bitmap in pixels.
    pub y: u16,
    /// Horizontal extent in pixels.
    pub width: NonZeroU8,
    /// Vertical extent in pixels.
    pub height: NonZeroU8,
}

/// [`SourceRect`] and horizontal metrics of a glyph required for text layout.
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "serde-deserialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive))]
#[cfg_attr(feature = "rkyv-serialize", derive(rkyv::Serialize))]
#[cfg_attr(feature = "rkyv-deserialize", derive(rkyv::Deserialize))]
pub struct BitmapGlyph {
    /// The bounding box of the rendered glyph in the bitmap.
    /// 
    /// None for whitespace characters.
    pub bitmap_source: Option<SourceRect>,
    /// The horizontal offset that the origin of the next glyph should be from the origin of this glyph.
    pub advance_width: f32,
    /// The horizontal offset between the origin of this glyph and the leftmost point of the glyph.
    pub left_side_bearing: f32,
    /// The vertical offset between the origin of this glyph and the baseline. Typhically positive.
    pub ascent: f32,
}

/// Runtime representation of all metadata for a single bitmap font.
/// 
/// Does not own or even reference the bitmap itself.
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "serde-deserialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive))]
#[cfg_attr(feature = "rkyv-serialize", derive(rkyv::Serialize))]
#[cfg_attr(feature = "rkyv-deserialize", derive(rkyv::Deserialize))]
pub struct BitmapFont {
    /// Map of unicode codepoints to glyphs in the font.
    pub glyphs: HashMap<char, BitmapGlyph>,
    /// Additional kerning to apply as well as that given by [`BitmapGlyph`] metrics to a pair of glyphs.
    pub kerning_table: Option<HashMap<(char, char), f32>>,
    /// The highest point that any glyph in the font extends above the baseline. Typically positive.
    pub ascent: f32,
    /// The lowest point that any glyph in the font extends below the baseline. Typically negative.
    pub descent: f32,
    /// The gap to leave between the descent of one line and the ascent of the next.
    /// 
    /// This is of course only a guideline given by the font's designers.
    pub line_gap: f32,
    /// The distance from the true pixel bounding box of any given glyph to the bounding box given by [`BitmapGlyph.bitmap_source`](BitmapGlyph).
    pub padding: u32,
}
