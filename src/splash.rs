use crate::{despawn_screen, AppState};
use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::color::palettes::basic::{BLACK, YELLOW};
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

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const UI_LAYERS: RenderLayers = RenderLayers::layer(0);
const ANIMATION_LAYERS: RenderLayers = RenderLayers::layer(1);

fn splash_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    window: Single<&Window>,
) {
    println!(
        "Setting up splash screen {}x{}",
        window.width(),
        window.height()
    );
    let font = asset_server.load("fonts/MinecraftEvenings.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 55.0,
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

    let title = Name::new("title");

    let start_pos = (
        window.width() / -2.0 + window.width() * 0.2,
        window.height() / -2.0 + window.height() * 0.6,
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
    player.play(animation_index).repeat();

    let title_id = commands
        .spawn((
            Text2d::new("Jigsaw Puzzle"),
            text_font.clone(),
            TextLayout::new_with_justify(text_justification),
            TextColor(BLACK.into()),
            ANIMATION_LAYERS,
            TargetCamera(anime_camera),
            // Transform::from_xyz(start_pos.0, start_pos.1, 0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
            title,
            AnimationGraphHandle(graphs.add(graph)),
            player,
        ))
        .id();

    commands.entity(title_id).insert(AnimationTarget {
        id: title_animation_target_id,
        player: title_id,
    });

    // Display the logo
    let root_node = commands
        .spawn((
            Node {
                // align_items: AlignItems::Center,
                // justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -10.0),
            UiImage::new(asset_server.load("images/puzzle.jpg")),
            TargetCamera(camera),
            OnSplashScreen,
        ))
        .id();

    let left_column = commands
        .spawn((
            Node {
                width: Val::Percent(40.),
                height: Val::Percent(100.0),
                // flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.5)),
            PickingBehavior::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    // BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((
                    Text::new("Start"),
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                ));
        })
        .id();

    let right_column = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(60.0),
                height: Val::Percent(100.0),
                ..default()
            },
            PickingBehavior::IGNORE,
            BackgroundColor(Color::srgba(0.5, 0.5, 0.0, 0.5)),
        ))
        .with_children(|p| {
            // p.spawn((
            //     Node {
            //         width: Val::Percent(100.0),
            //         height: Val::Percent(100.0),
            //         ..default()
            //     },
            //     BackgroundColor(Color::srgba(0.5, 0.5, 0.0, 0.5)),
            // ));
        })
        .id();

    commands
        .entity(root_node)
        .add_children(&[left_column, right_column]);

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
