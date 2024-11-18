use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use jigsaw_puzzle_generator::JigsawPiece;

mod gameplay;
mod main_menu;

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
                        // mode: WindowMode::Fullscreen(MonitorSelection::Primary),
                        // resolution: WindowResolution::new(800., 600.),
                        ..Default::default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .init_state::<AppState>();

        app.add_plugins((main_menu::menu_plugin, gameplay::plugin));
    }
}

/// The state of the application.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
    MainMenu,
    Gameplay,
}

#[derive(Resource, Deref)]
pub struct UiCamera(pub Entity);

#[derive(Resource, Deref)]
pub struct AnimeCamera(pub Entity);

pub const UI_LAYERS: RenderLayers = RenderLayers::layer(0);
pub const ANIMATION_LAYERS: RenderLayers = RenderLayers::layer(1);

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct OriginImage(pub Handle<Image>);

#[derive(Debug, Component, Deref, DerefMut)]
pub struct Piece(pub JigsawPiece);

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
