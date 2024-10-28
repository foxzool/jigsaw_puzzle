use bevy_jigsaw_puzzle::build_jigsaw_tiles;
use log::info;
use std::fs::create_dir_all;

fn main() {
    env_logger::init();
    info!("Start to load image");
    let img = image::open("images/raw.jpeg").expect("Failed to open image");
    info!("load image successfully!");
    let tiles = build_jigsaw_tiles(img, 9, 6, None, None, None);
    create_dir_all("tiles").expect("Failed to create tiles directory");
    for (i, tiles) in tiles.iter().enumerate() {
        tiles
            .image
            .save(format!("tiles/puzzle_tile_{}.png", i))
            .expect("Failed to save image");
    }
}
