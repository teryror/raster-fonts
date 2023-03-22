mod cli;
mod meta;

use cli::{Args, font_to_image};

fn main() {
    let args = Args::parse();

    font_to_image(args);
}
