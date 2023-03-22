//! De-/Serializable runtime representation of bitmap font metadata.
//! 
//! # Usage
//! ## RON
//! ```
//! # fn test() -> Result<(), ron::Error> {
//! // Requires Cargo feature `serde-deserialize` and the `ron` crate:
//! const FONT_METADATA: &'static str = include_str!("../font-metadata.ron");
//! let font: raster_fonts::BitmapFont = ron::from_str(FONT_METADATA)?;
//! # Ok(())
//! # }
//! # test().unwrap();
//! ```
//! 
//! ## JSON
//! ```
//! # fn test() -> Result<(), serde_json::Error> {
//! // Requires Cargo feature `serde-deserialize` and the `serde_json` crate:
//! const FONT_METADATA: &'static str = include_str!("../font-metadata.json");
//! let font: raster_fonts::BitmapFont = serde_json::from_str(FONT_METADATA)?;
//! # Ok(())
//! # }
//! # test().unwrap();
//! ```
//! 
//! ## RKYV
//! ```
//! // `rkyv` requires the data to be aligned for zero-copy deserialization,
//! // which in turn requires some trickery to achieve in a `const` context:
//! const FONT_METADATA: &'static [u8] = {
//!     #[repr(C)]
//!     struct Aligned<T: ?Sized> {
//!         _align: [usize; 0],
//!         bytes: T,
//!     }
//! 
//!     const ALIGNED: &'static Aligned<[u8]> = &Aligned {
//!         _align: [],
//!         bytes: *include_bytes!("../font-metadata.rkyv"),
//!     };
//!     
//!     &ALIGNED.bytes
//! };
//! 
//! // Using the unsafe API for maximum performance:
//! use raster_fonts::BitmapFont;
//! let archived_font = unsafe { rkyv::archived_root::<BitmapFont>(FONT_METADATA) };
//! // Optionally, unpack the archived metadata before use:
//! use rkyv::Deserialize;
//! let deserialized_font: BitmapFont = archived_font.deserialize(&mut rkyv::Infallible).unwrap();
//! ```

#![cfg_attr(docs_rs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]

mod meta;

pub use meta::{BitmapFont, BitmapGlyph, SourceRect};

#[cfg(feature = "bin")]
mod cli;

#[cfg(feature = "bin")]
pub use cli::{Args, font_to_image};
