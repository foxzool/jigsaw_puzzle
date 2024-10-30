use crate::ui::BoardBackgroundImage;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use jigsaw_puzzle_generator::JigsawGenerator;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_generator);
}

fn setup_generator(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image_path = "raw.jpg";
    let generator = JigsawGenerator::from_path(image_path, 9, 6).expect("Failed to load image");

    // load image from dynamic image
    let image = asset_server.add(Image::from_dynamic(
        generator.origin_image().clone(),
        true,
        RenderAssetUsages::RENDER_WORLD,
    ));

    commands.spawn((Sprite::from_image(image), BoardBackgroundImage));
    commands.insert_resource(JigsawPuzzleGenerator(generator));
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct JigsawPuzzleGenerator(pub JigsawGenerator);

fn spawn_piece(
    mut commands: Commands,
    generator: Res<JigsawPuzzleGenerator>,
    asset_server: Res<AssetServer>,
) {
}
