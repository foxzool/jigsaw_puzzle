use crate::gameplay::{JigsawPuzzleGenerator, MoveTogether, OriginImage, Selected, Shuffle};
use crate::{AppState, Piece};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::WindowMode;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Gameplay), setup_ui)
        .add_event::<AdjustScale>()
        .add_event::<ToggleBackgroundHint>()
        .add_event::<TogglePuzzleHint>()
        .add_event::<ToggleEdgeHint>()
        .add_systems(
            Update,
            (
                adjust_camera_on_added_sprite,
                adjust_camera_scale,
                handle_keyboard_input,
                handle_mouse_wheel_input,
                handle_toggle_background_hint,
                handle_toggle_puzzle_hint,
                exit_fullscreen_on_esc,
                handle_puzzle_hint,
            ),
        );
}

#[derive(Component)]
pub struct MenuIcon;
#[derive(Component)]
pub struct ZoomInButton;
#[derive(Component)]
pub struct ZoomOutButton;
#[derive(Component)]
pub struct HintImageButton;
#[derive(Component)]
pub struct SmallHintImage;
#[derive(Component)]
pub struct FullscreenButton;
#[derive(Component)]
pub struct PauseButton;
#[derive(Component)]
pub struct IdeaButton;
#[derive(Component)]
pub struct EdgeHintButton;
#[derive(Component)]
pub struct PuzzleHintChildButton;
#[derive(Component)]
pub struct BackgroundHintButton;

#[allow(dead_code)]
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let background_color = MAROON.into();
    let root_node = commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .id();

    let left_column =
        commands
            .spawn((
                Node {
                    width: Val::Vw(15.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Start,
                    margin: UiRect::axes(Val::Px(15.), Val::Px(5.)),
                    ..default()
                },
                PickingBehavior::IGNORE,
            ))
            .with_children(|builder| {
                // top left
                builder
                    .spawn((
                        Node {
                            width: Val::Percent(100.),
                            height: Val::Px(50.),
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        // BackgroundColor(BLUE.into()),
                    ))
                    .with_children(|builder| {
                        builder
                            .spawn((
                                UiImage::new(asset_server.load("icons/four-arrows.png")),
                                Node {
                                    height: Val::Px(40.),
                                    ..default()
                                },
                                MenuIcon,
                            ))
                            .observe(
                                |_trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                                    commands.send_event(Shuffle::Edge);
                                },
                            );

                        // zoom out button
                        builder
                            .spawn(Node {
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::End,
                                ..default()
                            })
                            .with_children(|builder| {
                                builder.spawn((
                                UiImage::new(asset_server.load("icons/zoom_out.png")),
                                Node {
                                    height: Val::Px(30.),
                                    margin: UiRect {
                                        left: Val::Px(5.),
                                        right: Val::Px(5.),
                                        ..default()
                                    },
                                    ..default()
                                },
                                ZoomOutButton,
                            )).observe(
                                |_trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                                    commands.send_event(AdjustScale(-0.1));
                                },
                            );

                                // zoom in button
                                builder.spawn((
                                UiImage::new(asset_server.load("icons/zoom_in.png")),
                                Node {
                                    height: Val::Px(30.),
                                    margin: UiRect {
                                        left: Val::Px(5.),
                                        right: Val::Px(5.),
                                        ..default()
                                    },
                                    ..default()
                                },
                                ZoomInButton,
                            )).observe(
                                |_trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                                    commands.send_event(AdjustScale(0.1));
                                },
                            );
                            });
                    });

                // bottom left
                builder.spawn(Node::default()).with_children(|p| {
                    // idea
                    p.spawn((
                        UiImage::new(asset_server.load("icons/lamp.png")),
                        Node {
                            height: Val::Px(40.),
                            margin: UiRect::axes(Val::Px(0.), Val::Px(5.)),
                            ..default()
                        },
                        IdeaButton,
                    ))
                    .observe(
                        |_trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                            commands.send_event(TogglePuzzleHint);
                        },
                    );

                    // puzzle control
                    p.spawn(((
                        Node {
                            margin: UiRect::all(Val::Px(5.)),
                            ..default()
                        },
                        EdgeHintButton,
                    ),))
                        .with_children(|p| {
                            p.spawn((
                                UiImage {
                                    image: asset_server.load("icons/puzzle_s.png"),
                                    flip_x: true,
                                    ..default()
                                },
                                Node {
                                    height: Val::Px(40.),
                                    margin: UiRect::axes(Val::Px(2.), Val::Px(5.)),
                                    ..default()
                                },
                            ));

                            p.spawn((
                                UiImage::new(asset_server.load("icons/puzzle_e.png")),
                                Node {
                                    height: Val::Px(30.),
                                    margin: UiRect {
                                        top: Val::Px(10.),
                                        bottom: Val::Px(10.),

                                        ..default()
                                    },
                                    ..default()
                                },
                                Visibility::Visible,
                                PuzzleHintChildButton,
                            ));

                            p.spawn((
                                UiImage::new(asset_server.load("icons/puzzle_s.png")),
                                Node {
                                    height: Val::Px(40.),
                                    margin: UiRect::axes(Val::Px(2.), Val::Px(5.)),
                                    ..default()
                                },
                            ));
                        })
                        .observe(
                            |_trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                                commands.send_event(ToggleEdgeHint);
                            },
                        );

                    // background hint
                    p.spawn((
                        UiImage::new(asset_server.load("icons/ghost.png")),
                        Node {
                            height: Val::Px(40.),
                            margin: UiRect::axes(Val::Px(0.), Val::Px(5.)),
                            ..default()
                        },
                        BackgroundHintButton,
                    ))
                    .observe(
                        |_trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                            commands.send_event(ToggleBackgroundHint);
                        },
                    );
                });
            })
            .id();

    let right_column = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::End,
                margin: UiRect::axes(Val::Px(5.), Val::Px(5.)),
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .with_children(|builder| {
            // top right
            builder
                .spawn((
                    Node {
                        // width: Val::Px(400.),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::End,
                        ..default()
                    },
                    // TopRightNode,
                ))
                .with_children(|p| {
                    p.spawn((
                        Node {
                            width: Val::Px(400.),
                            ..default()
                        },
                        SmallHintImage,
                    ));
                    p.spawn((
                        Node {
                            height: Val::Px(40.),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        UiImage::new(asset_server.load("icons/photo.png")),
                        HintImageButton,
                        Visibility::Visible,
                    ))
                    .observe(hint_image_click);
                });

            // bottom right
            builder.spawn(Node::default()).with_children(|p| {
                p.spawn((
                    UiImage::new(asset_server.load("icons/pause.png")),
                    Node {
                        height: Val::Px(40.),
                        margin: UiRect {
                            right: Val::Px(10.),
                            ..default()
                        },
                        ..default()
                    },
                    PauseButton,
                ));
                p.spawn((
                    UiImage::new(asset_server.load("icons/fullscreen.png")),
                    Node {
                        height: Val::Px(40.),
                        ..default()
                    },
                    FullscreenButton,
                ))
                .observe(
                    |_trigger: Trigger<Pointer<Click>>, mut window: Single<&mut Window>| {
                        window.mode = WindowMode::Fullscreen(MonitorSelection::Current);
                    },
                );
            });
        })
        .id();
    commands
        .entity(root_node)
        .add_children(&[left_column, right_column]);
}

#[derive(Component)]
pub struct BoardBackgroundImage;

/// Adjust the camera to fit the image
fn adjust_camera_on_added_sprite(
    _sprite: Single<Entity, Added<BoardBackgroundImage>>,
    mut camera_2d: Single<&mut OrthographicProjection, With<Camera2d>>,
    window: Single<&Window>,
    generator: Res<JigsawPuzzleGenerator>,
) {
    let window_width = window.resolution.width();
    let image_width = generator.origin_image().width() as f32;
    let scale = image_width / window_width;
    let target_scale = scale / 0.6;
    camera_2d.scale = target_scale;
}

#[derive(Event)]
pub struct AdjustScale(pub f32);

const MAX_SCALE: f32 = 3.0;
const MIN_SCALE: f32 = 0.5;

/// Adjust the camera scale on event
fn adjust_camera_scale(
    mut event: EventReader<AdjustScale>,
    mut camera_2d: Single<&mut OrthographicProjection, With<Camera2d>>,
) {
    for AdjustScale(scale) in event.read() {
        let new_scale = camera_2d.scale + scale;
        debug!("new scale: {}", new_scale);
        if (MIN_SCALE..=MAX_SCALE).contains(&new_scale) {
            camera_2d.scale = new_scale;
        }
    }
}

fn handle_keyboard_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard_input.just_pressed(KeyCode::PageUp) {
        commands.send_event(AdjustScale(0.1));
    } else if keyboard_input.just_pressed(KeyCode::PageDown) {
        commands.send_event(AdjustScale(-0.1));
    } else if keyboard_input.just_pressed(KeyCode::Space) {
        commands.send_event(ToggleBackgroundHint);
    } else if keyboard_input.just_pressed(KeyCode::KeyH) {
        commands.send_event(TogglePuzzleHint);
    } else if keyboard_input.just_pressed(KeyCode::KeyE) {
        commands.send_event(Shuffle::Edge);
    } else if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.send_event(Shuffle::Random);
    }
}

fn handle_mouse_wheel_input(
    mut mouse_wheel_input: EventReader<MouseWheel>,
    mut commands: Commands,
) {
    for event in mouse_wheel_input.read() {
        commands.send_event(AdjustScale(event.y * 0.1));
    }
}

#[derive(Event)]
pub struct ToggleBackgroundHint;

fn handle_toggle_background_hint(
    mut event: EventReader<ToggleBackgroundHint>,
    mut query: Query<&mut Visibility, With<BoardBackgroundImage>>,
) {
    for _ in event.read() {
        for mut visible in query.iter_mut() {
            visible.toggle_visible_hidden();
        }
    }
}

#[derive(Event)]
pub struct TogglePuzzleHint;

fn handle_toggle_puzzle_hint(
    mut event: EventReader<TogglePuzzleHint>,
    selected_query: Query<Entity, With<Selected>>,
    piece_query: Query<(Entity, &Piece, &MoveTogether), Without<Selected>>,
    mut commands: Commands,
) {
    for _ in event.read() {
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        let mut first_piece = None;
        let mut first_entity = None;
        let mut second_entity = None;
        'f1: for (entity, piece, move_together) in piece_query.iter() {
            if move_together.len() > 0 {
                continue 'f1;
            }
            first_piece = Some(piece);
            first_entity = Some(entity);
            break 'f1;
        }
        if let Some(first_piece) = first_piece {
            'f2: for (entity, piece, move_together) in piece_query.iter() {
                if move_together.len() > 0 {
                    continue 'f2;
                }
                if first_piece.beside(&piece) {
                    second_entity = Some(entity);
                    break 'f2;
                }
            }
        }
        if let (Some(first_entity), Some(second_entity)) = (first_entity, second_entity) {
            commands.entity(first_entity).insert(Selected);
            commands.entity(second_entity).insert(Selected);
        }
    }
}

fn exit_fullscreen_on_esc(mut window: Single<&mut Window>, input: Res<ButtonInput<KeyCode>>) {
    if !window.focused {
        return;
    }

    if input.just_pressed(KeyCode::Escape) {
        window.mode = WindowMode::Windowed;
    }
}

#[derive(Event)]
pub struct ToggleEdgeHint;

fn handle_puzzle_hint(
    mut event: EventReader<ToggleEdgeHint>,
    mut piece_query: Query<(&Piece, &mut Visibility), Without<PuzzleHintChildButton>>,
    mut ui: Single<&mut Visibility, With<PuzzleHintChildButton>>,
    mut show_all: Local<bool>,
) {
    for _ in event.read() {
        ui.toggle_visible_hidden();
        if *show_all {
            for (_, mut visibility) in piece_query.iter_mut() {
                *visibility = Visibility::Visible;
            }
        } else {
            for (piece, mut visibility) in piece_query.iter_mut() {
                if piece.is_edge() {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }

        *show_all = !*show_all;
    }
}

fn hint_image_click(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    // top_right: Single<Entity, With<TopRightNode>>,
    mut hint_visible: Single<
        &mut Visibility,
        (
            With<HintImageButton>,
            Without<SmallHintImage>,
            Without<BoardBackgroundImage>,
        ),
    >,
    small_hint_image: Single<Entity, With<SmallHintImage>>,
    origin_image: Res<OriginImage>,
) {
    hint_visible.toggle_visible_hidden();
    let aspect_ratio = origin_image.size.x / origin_image.size.y;

    commands
        .entity(*small_hint_image)
        .insert((
            UiImage::new(origin_image.image.clone()),
            Node {
                width: Val::Px(400.0),
                aspect_ratio: Some(aspect_ratio),
                ..default()
            },
            SmallHintImage,
            // BackgroundColor(Color::rgba(1.0, 1.0, 0.0, 0.5)),
            Visibility::Visible,
        ))
        .observe(hint_small_image_click);
}

fn hint_small_image_click(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut hint: Single<&mut Visibility, (With<HintImageButton>, Without<SmallHintImage>)>,
    small_img: Single<Entity, (With<SmallHintImage>, Without<HintImageButton>)>,
) {
    **hint = Visibility::Visible;
    commands.entity(*small_img).remove::<UiImage>();
}
