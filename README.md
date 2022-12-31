# `raster-fonts` [![Crates.io][crates-img]][crates] [![Documentation][docs-img]][docs]

[crates-img]: https://img.shields.io/crates/v/raster-fonts.svg
[crates]:     https://crates.io/crates/raster-fonts
[docs-img]:   https://docs.rs/raster-fonts/badge.svg
[docs]:       https://docs.rs/raster-fonts

This crate provides a command line utility for rasterizing true type fonts into bitmaps,
both traditional and signed distance fields. It also provides a tiny library for deserializing
the metadata required for dynamic text layout calculations; both [`serde`][serde] and
[`rkyv`][rkyv] are supported here.

[serde]: https://crates.io/crates/serde
[rkyv]:  https://crates.io/crates/rkyv

## The `font2img` CLI

### Installation Using `cargo`

```
> cargo install raster-fonts
```

### Basic Usage

```
> font2img <InputFontFile> <OutputImagePath> <OutputMetadataPath>
```

This will generate a monochrome signed distance field texture containing all glyphs in the printable
ASCII range from the given font file. [As described by Chris Green, then working for Valve](valve-paper):

[valve-paper]: https://steamcdn-a.akamaihd.net/apps/valve/2007/SIGGRAPH2007_AlphaTestedMagnification.pdf

> In the simplest case, this texture can then be rendered simply by using the alpha-testing and
> alpha-thresholding feature of modern GPUs, without a custom shader. [...]
> 
> With the use of programmable shading, the technique is extended to perform various special effect
> renderings, including soft edges, outlining, drop shadows, multi-colored images, and sharp corners.

For a short video demonstration of this technique, see [this video by Martin Donald][mdonald-demo].

[mdonald-demo]: https://www.youtube.com/watch?v=1b5hIMqz_wM

**Supported output image formats** include PNG, BMP, TIFF, TGA. The **supported metadata formats**
are [RON][ron], [JSON][json], and [RKYV][rkyv]. Note that kerning information cannot be exported to
JSON, because JSON dictionaries must be indexed by strings, whereas the other formats support indexing
by pairs of characters.

[ron]:  https://crates.io/crates/ron
[json]: https://crates.io/crates/serde_json
[rkyv]: https://crates.io/crates/rkyv

### Specifying Custom Character Ranges

```
> font2img <Font> <Img> <Meta> [Charset]...
```

To rasterize a different set of glyphs, one or more Unicode codepoint ranges must be specified after the
input/output paths. Codepoints are given in hexadecimal, without any prefix, and may stand alone.
Alternatively, a contiguous range may be specified by the inclusive minimum and maximum. For example:

```
# This is equivalent to the basic invocation from the previous section:
> font2img <Font> <Img> <Meta> 20-7F

# Additioanlly rasterize printable characters from the Latin-1 Supplement:
> font2img <Font> <Img> <Meta> 20-7F A1-FF

# Throw in the Pilcrow sign (¶) only:
> font2img <Font> <Img> <Meta> 20-7F B1
```

### Generating Conventional Bitmap Fonts

If you do not wish to use signed distance fields for whatever reason, you can switch to conventional
glyph rendering with the `--coverage-levels <N>` option (`-l <N>` for short), where `<N>` is the number
of distinct grayscale values greater than 0 you want in the output image. Regardless of the number of
levels, the output image will be a monochrome 8bpp image, with the highest level set to 255.

```
# Rasterize with maximum fidelity:
> font2img -l 255 <Font> <Img> <Meta>

# Generate a binary image, i.e. rasterize without anti-aliasing:
> font2img -l 1 <Font> <Img> <Meta>

# Use a happy medium with 16 total distinct values:
> font2img -l 15 <Font> <Img> <Meta>
```

This is still useful, especially if you know the exact text size you need, and want to manually process
the texture after generating it with this tool.

### Full Help Output

```
> font2img --help
Bitmap font creation tool and accompanying metadata deserialization library

Usage: font2img [OPTIONS] <FONT_PATH> <IMG_PATH> <META_PATH> [CHARSET]...

Arguments:
  <FONT_PATH>   Path to the font file to convert
  <IMG_PATH>    Path to where the output image should be written
  <META_PATH>   Path to where the output meta data should be written
  [CHARSET]...  List of Unicode codepoint ranges written in hex

Options:
  -l, --coverage-levels <COVERAGE_LEVELS>
          Enable coverage-based (as opposed to SDF) rasterization with the specified number of distinct levels above 0
  -s, --scale <SCALE>
          Desired font pixel height [default: 24]
  -p, --padding <PADDING>
          Desired number of pixels between glyphs in output texture [default: 8]
  -o, --output-image-size <OUTPUT_IMAGE_SIZE>
          Side length of the square output texture [default: 512]
      --skip-kerning-table
          Exclude additional kerning information from output metadata
  -h, --help
          Print help information
  -V, --version
          Print version information
```

## The `raster_fonts` Library

This is a single-module library of just a few plain-old data types that implement various de-/serialization
traits, depending on the cargo features you set. As such, you'll want to use different features depending on
the data format of your choice. So if you prefer `rkyv`, put this into your `Cargo.toml`:

```toml
[dependencies]
raster-fonts = { version = "0.1", features = ["rkyv-deserialize"] }
rkyv = "0.7"
```

If you want to use `serde`, you'll also need either `serde_json` or the `ron` crate:

```toml
[dependencies]
raster-fonts = { version = "0.1", features = ["serde-deserialize"] }
# depending on data format:
serde_json = "1"
ron = "0.8"
```

### [Documentation][docs]

[docs]: https://docs.rs/raster-fonts