use crate::gameplay::JigsawPuzzleGenerator;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .add_systems(Startup, setup)
        .add_systems(Update, (adjust_camera_on_added_sprite,));
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
pub struct FullscreenButton;
#[derive(Component)]
pub struct PauseButton;
#[derive(Component)]
pub struct IdeaButton;
#[derive(Component)]
pub struct PuzzleHintButton;
#[derive(Component)]
pub struct PuzzleHintChildButton;
#[derive(Component)]
pub struct BackgroundHintButton;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // let background_color = MAROON.into();
    let root_node = commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .id();

    let left_column = commands
        .spawn(Node {
            width: Val::Vw(15.),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Start,
            margin: UiRect::axes(Val::Px(15.), Val::Px(5.)),
            ..default()
        })
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
                    builder.spawn((
                        UiImage::new(asset_server.load("icons/menu.png")),
                        Node {
                            height: Val::Px(40.),
                            ..default()
                        },
                        MenuIcon,
                    ));
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
                            ));
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
                            ));
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
                ));

                // puzzle control
                p.spawn((
                    Node {
                        margin: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    PuzzleHintButton,
                ))
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
                });

                // background hint
                p.spawn((
                    UiImage::new(asset_server.load("icons/ghost.png")),
                    Node {
                        height: Val::Px(40.),
                        margin: UiRect::axes(Val::Px(0.), Val::Px(5.)),
                        ..default()
                    },
                    BackgroundHintButton,
                ));
            });
        })
        .id();

    let right_column = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::End,
            margin: UiRect::axes(Val::Px(15.), Val::Px(5.)),
            ..default()
        })
        .with_children(|builder| {
            // top right
            builder.spawn((
                UiImage::new(asset_server.load("icons/photo.png")),
                Node {
                    height: Val::Px(40.),
                    margin: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                HintImageButton,
            ));

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
                ));
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
    sprite: Query<Entity, Added<BoardBackgroundImage>>,
    mut camera_2d: Single<&mut OrthographicProjection, With<Camera2d>>,
    window: Single<&Window>,
    generator: Res<JigsawPuzzleGenerator>,
    mut commands: Commands,
) {
    if let Ok(entity) = sprite.get_single() {
        let window_width = window.resolution.width();
        let image_width = generator.origin_image().width() as f32;
        let scale = image_width / window_width;
        let target_scale = scale / 0.6;
        camera_2d.scale = target_scale;
        commands.entity(entity).insert(Visibility::Hidden);
    }
}
