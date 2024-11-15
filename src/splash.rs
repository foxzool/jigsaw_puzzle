use crate::{despawn_screen, AppState};
use bevy::color::palettes::basic::BLACK;
use bevy::prelude::*;

// This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
pub(super) fn splash_plugin(app: &mut App) {
    // As this plugin is managing the splash screen, it will focus on the state `GameState::Splash`
    app
        // When entering the state, spawn everything needed for this screen
        .add_systems(OnEnter(AppState::Splash), splash_setup)
        // While in this state, run the `countdown` system
        .add_systems(Update, countdown.run_if(in_state(AppState::Splash)))
        // When exiting the state, despawn everything that was spawned for this screen
        .add_systems(OnExit(AppState::Splash), despawn_screen::<OnSplashScreen>);
}

// Tag component used to tag entities added on the splash screen
#[derive(Component)]
struct OnSplashScreen;

// Newtype to use a `Timer` for this screen as a resource
#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/MinecraftEvenings.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 50.0,
        ..default()
    };
    let text_justification = JustifyText::Center;
    // Display the logo
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            OnSplashScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                UiImage::new(asset_server.load("images/puzzle.jpg")),
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Jigsaw Puzzle"),
                text_font.clone(),
                TextLayout::new_with_justify(text_justification),
                TextColor(BLACK.into()),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(100.0),
                    bottom: Val::Px(100.0),
                    ..default()
                },
            ));
        });

    // Insert the timer as a resource
    commands.insert_resource(SplashTimer(Timer::from_seconds(3.0, TimerMode::Once)));
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
