use crate::{AnimeCamera, AppState, UiCamera, ANIMATION_LAYERS};
use bevy::prelude::*;

// This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
pub(super) fn splash_plugin(app: &mut App) {
    // As this plugin is managing the splash screen, it will focus on the state `GameState::Splash`
    app
        // When entering the state, spawn everything needed for this screen
        .add_systems(OnEnter(AppState::Splash), splash_setup)
        // While in this state, run the `countdown` system
        .add_systems(Update, countdown.run_if(in_state(AppState::Splash)));
    // When exiting the state, despawn everything that was spawned for this screen
    // .add_systems(OnExit(AppState::Splash), despawn_screen::<OnSplashScreen>);
}

// Tag component used to tag entities added on the splash screen
#[derive(Component)]
struct OnSplashScreen;

// New type to use a `Timer` for this screen as a resource
#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>, window: Single<&Window>) {
    println!(
        "Setting up splash screen {}x{}",
        window.width(),
        window.height()
    );

    let camera = commands.spawn(Camera2d).id();
    commands.insert_resource(UiCamera(camera));

    let anime_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                ..default()
            },
            ANIMATION_LAYERS,
        ))
        .id();

    commands.insert_resource(AnimeCamera(anime_camera));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        UiImage::new(asset_server.load("images/puzzle.jpg")),
        TargetCamera(camera),
        OnSplashScreen,
    ));

    // Insert the timer as a resource
    commands.insert_resource(SplashTimer(Timer::from_seconds(0.8, TimerMode::Once)));
}

// Tick the timer, and change state when finished
fn countdown(
    mut game_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(AppState::MainMenu);
    }
}
