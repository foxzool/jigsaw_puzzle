use bevy::prelude::*;
use bevy::winit::WinitSettings;
use jigsaw_puzzle_generator::JigsawPiece;

mod gameplay;
mod ui;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Jigsaw Puzzle Game".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        // resolution: WindowResolution::new(800., 600.),
                        ..Default::default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .insert_resource(WinitSettings::desktop_app());

        app.add_plugins((ui::plugin, gameplay::plugin));
    }
}

#[derive(Debug, Component, Deref, DerefMut)]
pub struct Piece(pub JigsawPiece);
