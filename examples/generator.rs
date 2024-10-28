use bevy_jigsaw_puzzle::JigsawGenerator;
use env_logger::{Builder, Env};
use log::info;
use std::env;
use std::fs::create_dir_all;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug")
    }
    let env = Env::default();
    Builder::from_env(env).format_timestamp_millis().init();
    info!("Start to load image");
    let img = image::open("images/raw.jpeg").expect("Failed to open image");
    info!("load image successfully!");
    let template = JigsawGenerator::new(img, 9, 6).generate();
    create_dir_all("tiles").expect("Failed to create tiles directory");
    for piece in template.pieces.iter() {
        piece
            .image
            .save(format!("tiles/puzzle_piece_{}.png", piece.index))
            .expect("Failed to save image");
    }
}
