use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use core::fmt::Formatter;
use bevy::camera::visibility::RenderLayers;
use jigsaw_puzzle_generator::{GameMode, JigsawPiece};

mod gameplay;
mod main_menu;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
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
        .init_state::<AppState>()
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera);

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

/// game state
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Idle,
    Setup,
    Generating,
    Play,
    Pause,
    Finish,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct AnimeCamera;

pub const ANIMATION_LAYERS: RenderLayers = RenderLayers::layer(1);

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct OriginImage(pub Handle<Image>);

#[derive(Debug, Component, Deref, DerefMut, Clone)]
pub struct Piece(pub JigsawPiece);

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        ANIMATION_LAYERS,
        AnimeCamera,
    ));
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
    P500,
}

impl core::fmt::Display for SelectPiece {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
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
                SelectPiece::P500 => 500,
            }
        )
    }
}

impl SelectPiece {
    fn get_columns_rows(&self) -> (usize, usize) {
        match self {
            SelectPiece::P20 => (5, 4),
            SelectPiece::P50 => (10, 5),
            SelectPiece::P100 => (10, 10),
            SelectPiece::P150 => (15, 10),
            SelectPiece::P200 => (20, 10),
            SelectPiece::P250 => (25, 10),
            SelectPiece::P300 => (30, 10),
            SelectPiece::P400 => (20, 20),
            SelectPiece::P500 => (25, 20),
        }
    }

    fn next(&mut self) {
        *self = match self {
            SelectPiece::P20 => SelectPiece::P50,
            SelectPiece::P50 => SelectPiece::P100,
            SelectPiece::P100 => SelectPiece::P150,
            SelectPiece::P150 => SelectPiece::P200,
            SelectPiece::P200 => SelectPiece::P250,
            SelectPiece::P250 => SelectPiece::P300,
            SelectPiece::P300 => SelectPiece::P400,
            SelectPiece::P400 => SelectPiece::P500,
            SelectPiece::P500 => SelectPiece::P20,
        };
    }

    fn previous(&mut self) {
        *self = match self {
            SelectPiece::P20 => SelectPiece::P500,
            SelectPiece::P50 => SelectPiece::P20,
            SelectPiece::P100 => SelectPiece::P50,
            SelectPiece::P150 => SelectPiece::P100,
            SelectPiece::P200 => SelectPiece::P150,
            SelectPiece::P250 => SelectPiece::P200,
            SelectPiece::P300 => SelectPiece::P250,
            SelectPiece::P400 => SelectPiece::P300,
            SelectPiece::P500 => SelectPiece::P400,
        };
    }
}

#[derive(Debug, Resource, Deref, DerefMut, Default)]
pub struct SelectGameMode(pub GameMode);

impl core::fmt::Display for SelectGameMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
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
