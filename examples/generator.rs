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
    let image_path = env::args().nth(1).unwrap_or("raw.jpg".to_string());
    info!("Start to load {}", image_path);
    let img = image::open(image_path).expect("Failed to open image");
    info!("load image successfully!");
    let template = JigsawGenerator::new(img, 9, 6).generate();
    template
        .origin_image
        .save("tiles/origin_image.png")
        .expect("Failed to save image");
    create_dir_all("tiles").expect("Failed to create tiles directory");

    for piece in template.pieces.iter() {
        template
            .crop(piece)
            .save(format!("tiles/puzzle_piece_{}.png", piece.index))
            .expect("Failed to save image");
    }
}
