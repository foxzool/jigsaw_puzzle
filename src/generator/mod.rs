use crate::ui::BoardBackgroundImage;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use jigsaw_puzzle_generator::{JigsawGenerator, JigsawTemplate};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_generator);
}

fn setup_generator(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image_path = "raw.jpg";
    let template = JigsawGenerator::from_path(image_path, 9, 6)
        .expect("Failed to load image")
        .generate()
        .expect("Failed to generate puzzle");

    // load image from dynamic image
    let image = asset_server.add(Image::from_dynamic(
        template.origin_image.clone(),
        true,
        RenderAssetUsages::RENDER_WORLD,
    ));

    commands.spawn((Sprite::from_image(image), BoardBackgroundImage));

    commands.insert_resource(JigsawPuzzleGenerator {
        template,
        generated_piece: 0,
    });
}

#[derive(Debug, Resource)]
pub struct JigsawPuzzleGenerator {
    /// The jigsaw generator template
    pub template: JigsawTemplate,
    /// The number of pieces that have been generated
    pub generated_piece: usize,
}
