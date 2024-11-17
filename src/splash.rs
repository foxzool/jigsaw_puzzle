use crate::{despawn_screen, AppState};
use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::color::palettes::basic::BLACK;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

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

// New type to use a `Timer` for this screen as a resource
#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

const UI_LAYERS: RenderLayers = RenderLayers::layer(0);
const ANIMATION_LAYERS: RenderLayers = RenderLayers::layer(1);

fn splash_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    window: Single<&Window>,
) {
    let font = asset_server.load("fonts/MinecraftEvenings.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 60.0,
        ..default()
    };
    let text_justification = JustifyText::Center;
    let camera = commands.spawn(Camera2d).id();
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
            TargetCamera(camera),
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
        });

    let title = Name::new("title");

    let start_pos = (
        window.width() / -2.0 + 270.0,
        window.height() / -2.0 + 400.0,
    );

    // Creating the animation
    let mut animation = AnimationClip::default();
    // A curve can modify a single part of a transform: here, the translation.
    let title_animation_target_id = AnimationTargetId::from_name(&title);
    animation.add_curve_to_target(
        title_animation_target_id,
        UnevenSampleAutoCurve::new([0.0, 1.0, 2.0, 3.0].into_iter().zip([
            Vec3::new(start_pos.0, start_pos.1, 0.0),
            Vec3::new(start_pos.0, start_pos.1 + 50.0, 0.0),
            Vec3::new(start_pos.0, start_pos.1 + 100.0, 0.0),
        ]))
        .map(TranslationCurve)
        .expect("should be able to build translation curve because we pass in valid samples"),
    );

    animation.add_curve_to_target(
        title_animation_target_id,
        AnimatableKeyframeCurve::new([0.0, 1.0, 2.0, 3.0].into_iter().zip([
            Srgba::new(0.0, 0.0, 0.0, 0.1),
            Srgba::new(0.0, 0.0, 0.0, 0.3),
            Srgba::new(0.0, 0.0, 0.0, 0.6),
            Srgba::new(0.0, 0.0, 0.0, 1.0),
        ]))
        .map(AnimatableCurve::<TextColorProperty, _>::from_curve)
        .expect("should be able to build translation curve because we pass in valid samples"),
    );

    // Create the animation graph
    let (graph, animation_index) = AnimationGraph::from_clip(animations.add(animation));

    // Create the animation player, and set it to repeat
    let mut player = AnimationPlayer::default();
    player.play(animation_index);

    let title_id = commands
        .spawn((
            Text2d::new("Jigsaw Puzzle"),
            text_font.clone(),
            TextLayout::new_with_justify(text_justification),
            TextColor(BLACK.into()),
            ANIMATION_LAYERS,
            TargetCamera(anime_camera),
            title,
            AnimationGraphHandle(graphs.add(graph)),
            player,
        ))
        .id();

    commands.entity(title_id).insert(AnimationTarget {
        id: title_animation_target_id,
        player: title_id,
    });

    // Insert the timer as a resource
    commands.insert_resource(SplashTimer(Timer::from_seconds(3.0, TimerMode::Once)));
}

#[derive(Reflect)]
struct TextColorProperty;

impl AnimatableProperty for TextColorProperty {
    type Component = TextColor;

    type Property = Srgba;

    fn get_mut(component: &mut Self::Component) -> Option<&mut Self::Property> {
        match component.0 {
            Color::Srgba(ref mut color) => Some(color),
            _ => None,
        }
    }
}

// Tick the timer, and change state when finished
fn countdown(
    mut game_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        // game_state.set(AppState::MainMenu);
    }
}
