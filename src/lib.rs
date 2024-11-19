use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use jigsaw_puzzle_generator::{GameMode, JigsawPiece};
use std::fmt;
use std::fmt::Formatter;

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
        .init_resource::<SelectPiece>()
        .init_resource::<SelectGameMode>()
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

/// The state of the game.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
    GameOver,
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

#[derive(Resource, Default, Clone, Copy, Debug)]
enum SelectPiece {
    #[default]
    P20,
    P50,
    P100,
    P150,
    P200,
    P250,
    P300,
    P400,
}

impl fmt::Display for SelectPiece {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SelectPiece::P20 => 20,
                SelectPiece::P50 => 50,
                SelectPiece::P100 => 100,
                SelectPiece::P150 => 150,
                SelectPiece::P200 => 200,
                SelectPiece::P250 => 250,
                SelectPiece::P300 => 300,
                SelectPiece::P400 => 400,
            }
        )
    }
}

impl SelectPiece {
    fn next(&mut self) {
        *self = match self {
            SelectPiece::P20 => SelectPiece::P50,
            SelectPiece::P50 => SelectPiece::P100,
            SelectPiece::P100 => SelectPiece::P150,
            SelectPiece::P150 => SelectPiece::P200,
            SelectPiece::P200 => SelectPiece::P250,
            SelectPiece::P250 => SelectPiece::P300,
            SelectPiece::P300 => SelectPiece::P400,
            SelectPiece::P400 => SelectPiece::P20,
        };
    }

    fn previous(&mut self) {
        *self = match self {
            SelectPiece::P20 => SelectPiece::P400,
            SelectPiece::P50 => SelectPiece::P20,
            SelectPiece::P100 => SelectPiece::P50,
            SelectPiece::P150 => SelectPiece::P100,
            SelectPiece::P200 => SelectPiece::P150,
            SelectPiece::P250 => SelectPiece::P200,
            SelectPiece::P300 => SelectPiece::P250,
            SelectPiece::P400 => SelectPiece::P300,
        };
    }
}

#[derive(Debug, Resource, Deref, DerefMut, Default)]
pub struct SelectGameMode(pub GameMode);

impl fmt::Display for SelectGameMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                GameMode::Classic => "Classic",
                GameMode::Square => "Square",
            }
        )
    }
}

impl SelectGameMode {
    pub fn next(&mut self) {
        *self = match self.0 {
            GameMode::Classic => SelectGameMode(GameMode::Square),
            GameMode::Square => SelectGameMode(GameMode::Classic),
        };
    }

    pub fn previous(&mut self) {
        *self = match self.0 {
            GameMode::Classic => SelectGameMode(GameMode::Square),
            GameMode::Square => SelectGameMode(GameMode::Classic),
        };
    }
}
