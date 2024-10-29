use bevy::prelude::*;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Jigsaw Puzzle Game".to_string(),
                canvas: Some("#bevy".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..Default::default()
            }),
            ..default()
        }));
    }
}
