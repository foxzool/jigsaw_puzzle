use bevy::prelude::*;

mod generator;
mod ui;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Jigsaw Puzzle Game".to_string(),
                canvas: Some("#bevy".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                // resolution: WindowResolution::new(800., 600.),
                ..Default::default()
            }),
            ..default()
        }));

        app.add_plugins((ui::plugin, generator::plugin));
    }
}

#[derive(Debug, Component)]
pub struct JigsawTile {
    pub index: usize,
}
