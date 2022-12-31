use std::collections::{BTreeSet, HashMap};
use std::num::NonZeroU8;

use clap::Parser;
use rusttype::Font;

use raster_fonts::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the font file to convert
    font_path: String,
    /// Path to where the output image should be written
    img_path: String,
    /// Path to where the output meta data should be written.
    meta_path: String,
    /// List of Unicode codepoint ranges written in hex
    charset: Vec<String>,
    /// Enable coverage-based (as opposed to SDF) rasterization with the specified number of distinct levels above 0
    #[arg(short = 'l', long)]
    coverage_levels: Option<u8>,
    /// Desired font pixel height
    #[arg(short, long, default_value_t = 24.0)]
    scale: f32,
    /// Desired number of pixels between glyphs in output texture
    #[arg(short, long, default_value_t = 8)]
    padding: u32,
    /// Side length of the square output texture
    #[arg(short, long, default_value_t = 512)]
    output_image_size: u32,
    /// Exclude additional kerning information from output metadata
    #[arg(long)]
    skip_kerning_table: bool,
}

fn main() {
    let args = Args::parse();

    let current_dir = std::env::current_dir().expect("Failed to retrieve current directory");
    let font_path = current_dir.join(args.font_path);
    let font_data = std::fs::read(font_path).expect("Failed to read font data");
    let font = Font::try_from_vec(font_data).expect("Failed to parse font data");
    let scale = rusttype::Scale::uniform(args.scale);

    let max_dist = (args.padding as f32).powi(2);

    let mut outbuf = image::ImageBuffer::new(args.output_image_size, args.output_image_size);
    outbuf.fill(0x00);

    let mut out_metadata = {
        let rusttype::VMetrics {
            ascent, descent, line_gap
        } = font.v_metrics(scale);

        BitmapFont {
            glyphs: HashMap::new(),
            kerning_table: None,
            ascent,
            descent,
            line_gap,
            padding: args.padding,
        }
    };

    let mut next_x = 0;
    let mut next_y = 0;
    let mut next_y_adv = 0;

    let mut outside_buf = vec![];
    let mut inside_buf = vec![];

    let charset = {
        let mut charset = BTreeSet::<char>::new();
        let mut errors = false;

        'outer: for arg in args.charset.iter() {
            let mut piece_iter = arg.split('-');
            let fst = piece_iter.next();
            let snd = piece_iter.next();
            if piece_iter.next().is_some() {
                eprintln!("Error parsing charset specifier: {arg}");
                errors = true;
                continue;
            }

            match (fst, snd) {
                (Some(single_char), None) => {
                    let Ok(codepoint) = u32::from_str_radix(single_char, 16) else {
                        eprintln!("Error parsing charset specifier: {arg}");
                        errors = true;
                        continue;
                    };

                    let Ok(single_char) = codepoint.try_into() else {
                        eprintln!("{codepoint:x} is not a valid Unicode codepoint!");
                        errors = true;
                        continue;
                    };

                    charset.insert(single_char);
                },
                (Some(fst), Some(snd)) => {
                    let fst = u32::from_str_radix(fst, 16);
                    let snd = u32::from_str_radix(snd, 16);

                    let (Ok(min), Ok(max)) = (fst, snd) else {
                        eprintln!("Error parsing charset specifier: {arg}");
                        errors = true;
                        continue;
                    };

                    for codepoint in min..=max {
                        let Ok(single_char) = codepoint.try_into() else {
                            eprintln!("{codepoint:x} is not a valid Unicode codepoint!");
                            errors = true;
                            continue 'outer;
                        };

                        charset.insert(single_char);
                    }
                },
                _ => {
                    eprintln!("Error parsing charset specifier: {arg}");
                    errors = true;
                    continue;
                }
            }
        }

        if errors {
            eprintln!("! Valid charset specifiers are:");
            eprintln!("    [SINGLE_CHARACTER]");
            eprintln!("    [MIN_INCLUSIVE]-[MAX_INCLUSIVE]");
            eprintln!("All codepoints written in hex, with no prefix, i.e. as in 5F or 20-7f");
            return;
        }

        if charset.is_empty() {
            eprintln!("No charset specified. Defaulting to ASCII Range (20-7f)");
            for codepoint in 0x20u8..=0x7F {
                charset.insert(codepoint as char);
            }
        }

        charset
    };

    for &glyph_id in charset.iter() {
        let glyph = font.glyph(glyph_id).scaled(scale).positioned(rusttype::Point::default());
        let Some(bounding_box) = glyph.pixel_bounding_box() else {
            if !glyph_id.is_whitespace() {
                eprintln!("Failed to obtain bounding box for non-whitespace glyph {:x}", glyph_id as u32);
            }

            let rusttype::HMetrics {
                advance_width,
                left_side_bearing,
            } = glyph.unpositioned().h_metrics();
    
            let glyph_metadata = BitmapGlyph { bitmap_source: None, advance_width, left_side_bearing, ascent: f32::NAN };
    
            out_metadata.glyphs.insert(glyph_id, glyph_metadata); 
            continue;
        };
        
        let width = bounding_box.width() as u32;
        let height = bounding_box.height() as u32;

        let padded_w = width + 2 * args.padding;
        let padded_h = height + 2 * args.padding;
        let n_pixels = padded_w * padded_h;
        
        if next_x + padded_w > args.output_image_size {
            next_x = 0;
            next_y += next_y_adv;
            next_y_adv = 0;
        }

        next_y_adv = u32::max(next_y_adv, padded_h);
        
        if let Some(levels) = args.coverage_levels {
            glyph.draw(|x, y, v| {
                let x = next_x + args.padding + x;
                let y = next_y + args.padding + y;
                let pixel_value = (((v * (levels as f32)).round() / (levels as f32)) * 255.0).round() as u8;
                
                *outbuf.get_pixel_mut(x, y) = image::Luma([pixel_value; 1]);
            });
        } else {
            outside_buf.clear();
            outside_buf.resize(n_pixels as usize, max_dist);

            glyph.draw(|x, y, v| {
                let idx = (((args.padding + y) * padded_w) + args.padding + x) as usize;
                if v <= 0.5 { outside_buf[idx] = max_dist; }
                else { outside_buf[idx] = 0.0; }
            });

            // assign vertical distances column-wise
            for x in 0..width {
                let x = x + args.padding;

                // propagate distances downwards
                let mut dist_step = 1.0;
                for y in 1..padded_h {
                    let idx_here = ((y * padded_w) + x) as usize;
                    let idx_up = (((y-1) * padded_w) + x) as usize;

                    if outside_buf[idx_here] > outside_buf[idx_up] + dist_step {
                        outside_buf[idx_here] = outside_buf[idx_up] + dist_step;
                        dist_step += 2.0;
                    } else {
                        dist_step = 1.0;
                    }
                }

                // propagate distances upwards
                let mut dist_step = 1.0;
                for y in (0..padded_h-1).rev() {
                    let idx_here = ((y * padded_w) + x) as usize;
                    let idx_down = (((y+1) * padded_w) + x) as usize;

                    if outside_buf[idx_here] > outside_buf[idx_down] + dist_step {
                        outside_buf[idx_here] = outside_buf[idx_down] + dist_step;
                        dist_step += 2.0;
                    } else {
                        dist_step = 1.0;
                    }
                }
            }

            inside_buf.clear();
            inside_buf.resize(n_pixels as usize, 0.0);

            glyph.draw(|x, y, v| {
                let idx = (((args.padding + y) * padded_w) + args.padding + x) as usize;
                if v <= 0.5 { inside_buf[idx] = 0.0; }
                else { inside_buf[idx] = max_dist; }
            });

            // assign vertical distances column-wise
            for x in 0..width {
                let x = x + args.padding;
                
                // propagate distances downwards
                let mut dist_step = 1.0;
                for y in 1..padded_h {
                    let idx_here = ((y * padded_w) + x) as usize;
                    let idx_up = (((y-1) * padded_w) + x) as usize;

                    if inside_buf[idx_here] > inside_buf[idx_up] + dist_step {
                        inside_buf[idx_here] = inside_buf[idx_up] + dist_step;
                        dist_step += 2.0;
                    } else {
                        dist_step = 1.0;
                    }
                }

                // propagate distances upwards
                let mut dist_step = 1.0;
                for y in (0..padded_h-1).rev() {
                    let idx_here = ((y * padded_w) + x) as usize;
                    let idx_down = (((y+1) * padded_w) + x) as usize;

                    if inside_buf[idx_here] > inside_buf[idx_down] + dist_step {
                        inside_buf[idx_here] = inside_buf[idx_down] + dist_step;
                        dist_step += 2.0;
                    } else {
                        dist_step = 1.0;
                    }
                }
            }

            // determine actual distances row-wise
            for y in 0..padded_h {
                for x_here in 0..padded_w {
                    let idx_here = ((y * padded_w) + x_here) as usize;
                    let mut dist_min = outside_buf[idx_here];
                    for x_there in 0..padded_w {
                        let idx_there = ((y * padded_w) + x_there) as usize;
                        let dist = outside_buf[idx_there] + (x_there as f32 - x_here as f32).powi(2);
                        if dist_min > dist {
                            dist_min = dist;
                        }
                    }

                    let outside_distance = (dist_min / max_dist).clamp(0.0, 1.0);

                    let mut dist_min = inside_buf[idx_here];
                    for x_there in 0..padded_w {
                        let idx_there = ((y * padded_w) + x_there) as usize;
                        let dist = inside_buf[idx_there] + (x_there as f32 - x_here as f32).powi(2);
                        if dist_min > dist {
                            dist_min = dist;
                        }
                    }

                    let inside_distance = (dist_min / max_dist).clamp(0.0, 1.0);
                    
                    let signed_distance = if outside_distance > 0.0 { -outside_distance } else { inside_distance };
                    let pixel_value = (((signed_distance + 1.0) / 2.0) * 255.0).round() as u8;
                    *outbuf.get_pixel_mut(next_x + x_here, next_y + y) = image::Luma([pixel_value; 1]);
                }
            }
        }

        let bitmap_source = Some(SourceRect {
            x: next_x as u16, y: next_y as u16, width: NonZeroU8::new(padded_w as u8).expect(""), height: NonZeroU8::new(padded_h as u8).expect("")
        });

        let rusttype::HMetrics {
            advance_width,
            left_side_bearing,
        } = glyph.unpositioned().h_metrics();

        let ascent = bounding_box.min.y as f32 * -1.0;

        let glyph_metadata = BitmapGlyph { bitmap_source, advance_width, left_side_bearing, ascent };

        out_metadata.glyphs.insert(glyph_id, glyph_metadata); 

        next_x += padded_w;
    }

    if !args.skip_kerning_table {
        let mut kerning_table = HashMap::new();
        for &first in charset.iter() {
            for &second in charset.iter() {
                let kerning_offset = font.pair_kerning(scale, first, second);
                if kerning_offset != 0.0 {
                    kerning_table.insert((first, second), kerning_offset);
                }
            }
        }

        if !kerning_table.is_empty() {
            out_metadata.kerning_table = Some(kerning_table);
        }
    }

    outbuf.save(args.img_path).expect("Failed to write output image");

    let meta_path = current_dir.join(&args.meta_path);
    match meta_path.extension().map(|os_str| os_str.to_str()) {
        Some(Some("ron")) => {
            let serialized_meta = ron::ser::to_string_pretty(&out_metadata, ron::ser::PrettyConfig::default()).expect("Failed to serialize output metadata");
            std::fs::write(meta_path, serialized_meta).expect("Unable to write file");
        },
        Some(Some("json")) => {
            if out_metadata.kerning_table.is_some() {
                eprintln!("Cannot encode kerning table into JSON.");
                eprintln!("This is because JSON requires dictionary keys to be strings,");
                eprintln!("and we don't want to push this requirement into other formats.");
                out_metadata.kerning_table = None;
            }

            let serialized_meta = serde_json::to_string(&out_metadata).expect("Failed to serialize output metadata");
            std::fs::write(meta_path, serialized_meta).expect("Unable to write file");
        },
        Some(Some("rkyv")) => {
            let serialized_meta = rkyv::to_bytes::<_, 4096>(&out_metadata).expect("Failed to serialize output metadata");
            std::fs::write(meta_path, serialized_meta).expect("Unable to write file");
        },
        _ => {
            eprintln!("Failed to deduce meta data format from path: {}", args.meta_path);
            eprintln!("Supported formats are: ron, json, rkyv");
            eprintln!("Note that JSON serialization currently does not support kerning tables.");
            return;
        },
    }

    println!("Ok.");
}
