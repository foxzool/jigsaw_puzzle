use bevy::prelude::*;
use jigsaw_puzzle_generator::JigsawGenerator;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, load_image);
}

fn load_image() {
    let image_path = "raw.jpg";
    JigsawGenerator::from_path(image_path, 9, 6)
        .expect("Failed to load image")
        .generate()
        .expect("Failed to generate puzzle");
}
