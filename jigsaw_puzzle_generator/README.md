[![crates.io](https://img.shields.io/crates/v/jigsaw_puzzle_generator)](https://crates.io/crates/jigsaw_puzzle_generator)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Seldom-SE/seldom_pixel#license)
[![Documentation](https://docs.rs/jigsaw_puzzle_generator/badge.svg)](https://docs.rs/jigsaw_puzzle_generator)

# Jigsaw Puzzle Generator

`jigsaw_puzzle_generator` is a simple helper to generate jigsaw puzzle in Rust.
Inspired by the [puzzle-paths](https://gitlab.switch.ch/ub-unibas/puzzle-app/puzzle-paths)

## Usage

``` rust, no_run
use env_logger::{Builder, Env};
use jigsaw_puzzle_generator::JigsawGenerator;
use std::env;
use std::fs::create_dir_all;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug")
    }
    let env = Env::default();
    Builder::from_env(env).format_timestamp_millis().init();
    let image_path = env::args().nth(1).unwrap_or("raw.jpg".to_string());
    let template = JigsawGenerator::from_path(&image_path, 9, 6)
        .expect("Failed to load image")
        .generate(false)
        .expect("Failed to generate puzzle");
    create_dir_all("images").expect("Failed to create images directory");
    template
        .origin_image
        .save("images/origin_image.png")
        .expect("Failed to save image");

    for piece in template.pieces.iter() {
        piece
            .crop(&template.origin_image)
            .save(format!("images/puzzle_piece_{}.png", piece.index))
            .expect("Failed to save image");
    }
}

```