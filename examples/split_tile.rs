use bevy_jigsaw_puzzle::build_jigsaw_tiles;
use log::info;

fn main() {
    env_logger::init();
    info!("Start!");
    let img = image::open("images/raw.jpeg").expect("Failed to open image");
    info!("load image successfully!");
    let pieces = build_jigsaw_tiles(img, 9, 6, None, None, None);
}
