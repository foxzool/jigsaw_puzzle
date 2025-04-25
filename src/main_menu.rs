use crate::{
    ANIMATION_LAYERS, AnimeCamera, AppState, HOVERED_BUTTON, NORMAL_BUTTON, OriginImage,
    PRESSED_BUTTON, SelectGameMode, SelectPiece, despawn_screen,
};
use bevy::animation::{
    AnimationEntityMut, AnimationEvaluationError, AnimationTarget, AnimationTargetId,
    animated_field,
};
use bevy::color::palettes::basic::BLACK;
use bevy::prelude::*;
use bevy::window::WindowResized;
use core::any::TypeId;

pub(crate) fn menu_plugin(app: &mut App) {
    app.init_resource::<LoadedImages>()
        .init_resource::<Dragging>()
        .add_systems(
            OnEnter(AppState::MainMenu),
            (setup_menu, load_default_images, load_anime).chain(),
        )
        .add_systems(
            Update,
            (
                windows_resize_event,
                menu_countdown,
                button_interaction,
                show_origin_image.run_if(resource_changed::<OriginImage>),
                update_piece_text.run_if(resource_changed::<SelectPiece>),
                update_game_mode_text.run_if(resource_changed::<SelectGameMode>),
                show_images.run_if(resource_changed::<LoadedImages>),
            )
                .run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(OnExit(AppState::MainMenu), despawn_screen::<OnMenuScreen>)
        .add_observer(show_title);
}

#[derive(Component)]
struct OnMenuScreen;

const IMAGE_PATHS: [&str; 5] = [
    "images/raw.jpg",
    "images/rock.jpg",
    "images/mount.jpg",
    "images/sea.jpg",
    "images/dock.jpg",
];

#[derive(Resource, Deref, DerefMut)]
struct MenuTimer(Timer);

#[derive(Event)]
struct ShowTitleAnime;

#[derive(Reflect, Clone)]
struct TextColorProperty;

impl AnimatableProperty for TextColorProperty {
    type Property = Srgba;

    fn evaluator_id(&self) -> EvaluatorId {
        EvaluatorId::Type(TypeId::of::<Self>())
    }

    fn get_mut<'a>(
        &self,
        entity: &'a mut AnimationEntityMut,
    ) -> Result<&'a mut Self::Property, AnimationEvaluationError> {
        let text_color = entity
            .get_mut::<TextColor>()
            .ok_or(AnimationEvaluationError::ComponentNotPresent(TypeId::of::<
                TextColor,
            >(
            )))?
            .into_inner();
        match text_color.0 {
            Color::Srgba(ref mut color) => Ok(color),
            _ => Err(AnimationEvaluationError::PropertyNotPresent(TypeId::of::<
                Srgba,
            >(
            ))),
        }
    }
}

#[derive(Component)]
struct HiddenItem;

#[allow(clippy::too_many_arguments)]
fn show_title(
    _trigger: Trigger<ShowTitleAnime>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    window: Single<&Window>,
    anime_camera: Single<Entity, With<AnimeCamera>>,
    old_title: Query<Entity, With<AnimationTarget>>,
) {
    for entity in old_title.iter() {
        commands.entity(entity).despawn();
    }

    let font = asset_server.load("fonts/MinecraftEvenings.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 55.0,
        ..default()
    };
    let text_justification = JustifyText::Center;

    let start_pos = (
        window.width() / -2.0 + window.width() * 0.2,
        window.height() / -2.0 + window.height() * 0.6,
    );

    let title = Name::new("title");
    // Creating the animation
    let mut animation = AnimationClip::default();
    // A curve can modify a single part of a transform: here, the translation.
    let title_animation_target_id = AnimationTargetId::from_name(&title);
    animation.add_curve_to_target(
        title_animation_target_id,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            UnevenSampleAutoCurve::new([0.0, 0.5, 1.0, 2.0, 3.0].into_iter().zip([
                Vec3::new(start_pos.0, start_pos.1, 0.0),
                Vec3::new(start_pos.0, start_pos.1 + 50.0, 0.0),
                Vec3::new(start_pos.0, start_pos.1 + 110.0, 0.0),
                Vec3::new(start_pos.0, start_pos.1 + 180.0, 0.0),
            ]))
            .expect("should be able to build translation curve because we pass in valid samples"),
        ),
    );

    animation.add_curve_to_target(
        title_animation_target_id,
        AnimatableCurve::new(
            TextColorProperty,
            AnimatableKeyframeCurve::new([0.0, 1.0, 2.0, 3.0].into_iter().zip([
                Srgba::new(0.0, 0.0, 0.0, 0.1),
                Srgba::new(0.0, 0.0, 0.0, 0.3),
                Srgba::new(0.0, 0.0, 0.0, 0.6),
                Srgba::new(0.0, 0.0, 0.0, 1.0),
            ]))
            .expect("should be able to build translation curve because we pass in valid samples"),
        ),
    );

    // Create the animation graph
    let (graph, animation_index) = AnimationGraph::from_clip(animations.add(animation));

    // Create the animation player
    let mut player = AnimationPlayer::default();
    player.play(animation_index);

    let title_id = commands
        .spawn((
            Text2d::new("Jigsaw Puzzle"),
            text_font.clone(),
            TextLayout::new_with_justify(text_justification),
            TextColor(BLACK.into()),
            ANIMATION_LAYERS,
            UiTargetCamera(*anime_camera),
            Transform::from_xyz(start_pos.0, start_pos.1, 1.0),
            // Transform::from_xyz(0.0, 0.0, 0.0),
            title,
            AnimationGraphHandle(graphs.add(graph)),
            player,
            OnMenuScreen,
        ))
        .id();

    commands.entity(title_id).insert(AnimationTarget {
        id: title_animation_target_id,
        player: title_id,
    });
}

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    select_piece: Res<SelectPiece>,
    select_mode: Res<SelectGameMode>,
) {
    let image = asset_server.load("images/raw.jpg");
    commands.insert_resource(OriginImage(image));
    let text_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    // let title_font = asset_server.load("fonts/MinecraftEvenings.ttf");
    let down_arrow = asset_server.load("icons/down-arrow.png");

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
            ImageNode::new(asset_server.load("images/puzzle.jpg")),
            OnMenuScreen,
        ))
        .id();

    let left_column = commands
        .spawn((
            Node {
                width: Val::Percent(40.),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            // BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.5)),
            Pickable::IGNORE,
            Visibility::Hidden,
            HiddenItem,
        ))
        .with_children(|p| {
            // top container holder
            p.spawn(Node {
                height: Val::Percent(52.0),
                ..default()
            });
            // bottom container
            p.spawn((
                Node {
                    height: Val::Percent(48.0),
                    // width: Val::Px(300.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    // justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect {
                        top: Val::Px(70.0),
                        ..default()
                    },
                    ..default()
                },
                // BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.5)),
                BackgroundColor(Color::srgba(0.2, 0.7, 0.5, 0.5)),
            ))
            .with_children(|p| {
                // selector container
                p.spawn((
                    Node {
                        // width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        column_gap: Val::Px(10.0),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    // BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.5)),
                ))
                .with_children(|p| {
                    // piece selection
                    p.spawn(Node {
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|p| {
                        // up arrow
                        p.spawn((
                            ImageNode {
                                image: down_arrow.clone(),
                                flip_y: true,
                                ..default()
                            },
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                ..default()
                            },
                        ))
                        .observe(
                            |_trigger: Trigger<Pointer<Click>>,
                             mut select_piece: ResMut<SelectPiece>| {
                                select_piece.previous();
                            },
                        );
                        p.spawn((
                            PieceNumText,
                            Text::new(select_piece.to_string()),
                            TextFont {
                                font: text_font.clone(),
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                            Node {
                                margin: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                                ..default()
                            },
                        ));
                        // down arrow
                        p.spawn((
                            ImageNode::new(down_arrow.clone()),
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                ..default()
                            },
                        ))
                        .observe(
                            |_trigger: Trigger<Pointer<Click>>,
                             mut select_piece: ResMut<SelectPiece>| {
                                select_piece.next();
                            },
                        );
                    });

                    // text
                    p.spawn((
                        Text::new("pieces"),
                        TextFont {
                            font: text_font.clone(),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                        Node {
                            margin: UiRect::axes(Val::Px(0.0), Val::Px(31.0)),
                            ..default()
                        },
                    ));

                    // mode selection
                    p.spawn(Node {
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|p| {
                        // up arrow
                        p.spawn((
                            ImageNode {
                                image: down_arrow.clone(),
                                flip_y: true,
                                ..default()
                            },
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                ..default()
                            },
                        ))
                        .observe(
                            |_trigger: Trigger<Pointer<Click>>,
                             mut select_mode: ResMut<SelectGameMode>| {
                                select_mode.previous();
                            },
                        );
                        p.spawn((
                            GameModeText,
                            Text::new(select_mode.to_string()),
                            TextFont {
                                font: text_font.clone(),
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                            Node {
                                margin: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                                ..default()
                            },
                        ));
                        // down arrow
                        p.spawn((
                            ImageNode::new(down_arrow.clone()),
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                ..default()
                            },
                        ))
                        .observe(
                            |_trigger: Trigger<Pointer<Click>>,
                             mut select_mode: ResMut<SelectGameMode>| {
                                select_mode.next();
                            },
                        );
                    });
                });

                // start button
                p.spawn((
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
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    // BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((
                    Text::new("Start"),
                    TextFont {
                        font: text_font.clone(),
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                ))
                .observe(
                    |_trigger: Trigger<Pointer<Click>>,
                     mut app_state: ResMut<NextState<AppState>>| {
                        app_state.set(AppState::Gameplay);
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
                width: Val::Percent(60.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
            // BackgroundColor(Color::srgba(0.5, 0.1, 0.0, 0.5)),
        ))
        .with_children(|p| {
            // image preview
            p.spawn((Node {
                width: Val::Percent(100.0),
                height: Val::Percent(70.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },))
                .with_children(|p| {
                    p.spawn((
                        HiddenItem,
                        Visibility::Hidden,
                        OriginImageContainer,
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        Outline {
                            width: Val::Px(3.0),
                            color: Color::BLACK,
                            offset: Val::Px(2.0),
                        },
                    ));
                });

            // images collection container
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(30.0),
                    margin: UiRect::all(Val::Px(4.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                // BackgroundColor(Color::srgba(0.7, 0.1, 0.5, 0.5)),
            ))
            .with_children(|p| {
                p.spawn((
                    Node {
                        // width: Val::Percent(100.0),
                        height: Val::Percent(80.0),
                        display: Display::Flex,
                        justify_content: JustifyContent::SpaceBetween,
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        margin: UiRect::all(Val::Px(30.)),
                        ..default()
                    },
                    // BackgroundColor(Color::srgba(0.4, 0.5, 0.5, 0.5)),
                    ImagesContainer,
                    Visibility::Hidden,
                    HiddenItem,
                ))
                .observe(drag_start)
                .observe(drag_end)
                .observe(drag_images_collection);
            });
        })
        .id();

    commands
        .entity(root_node)
        .add_children(&[left_column, right_column]);

    commands.insert_resource(MenuTimer(Timer::from_seconds(2.9, TimerMode::Once)));
}

fn windows_resize_event(mut commands: Commands, mut resize_events: EventReader<WindowResized>) {
    for _ev in resize_events.read() {
        commands.trigger(ShowTitleAnime);
    }
}

#[derive(Component)]
struct OriginImageContainer;

#[derive(Component)]
struct ImagesContainer;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct LoadedImages(Vec<Handle<Image>>);

fn load_default_images(asset_server: Res<AssetServer>, mut loaded_images: ResMut<LoadedImages>) {
    for path in IMAGE_PATHS {
        let image_handle = asset_server.load(path);

        loaded_images.0.push(image_handle);
    }
}

fn load_anime(mut commands: Commands) {
    commands.trigger(ShowTitleAnime);
}

fn menu_countdown(
    time: Res<Time>,
    mut timer: ResMut<MenuTimer>,
    mut items: Query<&mut Visibility, With<HiddenItem>>,
    mut commands: Commands,
    image_handle: Res<LoadedImages>,
) {
    if timer.tick(time.delta()).just_finished() {
        for mut visible in items.iter_mut() {
            *visible = Visibility::Visible;
        }

        let image_handle = image_handle.0.first().unwrap();

        commands.insert_resource(OriginImage(image_handle.clone()));
    }
}

fn button_interaction(
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, children) in interaction_query.iter() {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                text_color.0 = PRESSED_BUTTON;
            }
            Interaction::Hovered => {
                text_color.0 = HOVERED_BUTTON;
            }
            Interaction::None => {
                text_color.0 = NORMAL_BUTTON;
            }
        }
    }
}

fn show_origin_image(
    container: Single<Entity, With<OriginImageContainer>>,
    mut commands: Commands,
    origin_image: Res<OriginImage>,
) {
    commands
        .entity(*container)
        .insert(ImageNode::new(origin_image.0.clone()));
}

fn show_images(
    container: Single<Entity, With<ImagesContainer>>,
    mut commands: Commands,
    loaded_images: Res<LoadedImages>,
) {
    for image in loaded_images.0.iter() {
        let child_node = commands
            .spawn((
                ImageNode::new(image.clone()),
                Node {
                    margin: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                    ..default()
                },
                // Outline {
                //     width: Val::Px(2.0),
                //     color: Color::BLACK,
                //     offset: Val::Px(0.0),
                // },
            ))
            .observe(
                |trigger: Trigger<Pointer<Click>>,
                 mut origin_image: ResMut<OriginImage>,
                 dragging: Res<Dragging>,
                 image_query: Query<&ImageNode>| {
                    if dragging.0 {
                        return;
                    }
                    let image = image_query.get(trigger.target()).unwrap();
                    origin_image.0 = image.image.clone();
                },
            )
            .id();

        commands.entity(*container).add_child(child_node);
    }
}

#[derive(Component)]
struct PieceNumText;

#[derive(Component)]
struct GameModeText;

fn update_game_mode_text(
    select_mode: Res<SelectGameMode>,
    mut mode_query: Query<&mut Text, With<GameModeText>>,
) {
    for mut text in mode_query.iter_mut() {
        text.0 = select_mode.to_string();
    }
}

fn update_piece_text(
    select_piece: Res<SelectPiece>,
    mut piece_query: Query<&mut Text, With<PieceNumText>>,
) {
    for mut text in piece_query.iter_mut() {
        text.0 = select_piece.to_string();
    }
}

#[derive(Resource, Default)]
struct Dragging(bool);

fn drag_start(_trigger: Trigger<Pointer<DragStart>>, mut dragging: ResMut<Dragging>) {
    dragging.0 = true;
}

fn drag_end(_trigger: Trigger<Pointer<DragEnd>>, mut dragging: ResMut<Dragging>) {
    dragging.0 = false;
}

fn drag_images_collection(
    trigger: Trigger<Pointer<Drag>>,
    container: Single<(&mut Node, &ComputedNode, &Children), With<ImagesContainer>>,
    compute_node: Query<&ComputedNode>,
) {
    let (mut container, current_node, children) = container.into_inner();
    let Val::Px(px) = container.left else {
        return;
    };

    let child_node = compute_node.get(*children.first().unwrap()).unwrap();
    let child_width = child_node.size().x;

    let min_x = -(current_node.size().x + child_width);
    let max_x = current_node.size().x - child_width;
    let new_left = px + trigger.event.delta.x;

    if new_left < min_x {
        container.left = Val::Px(min_x);
        return;
    }

    if new_left > max_x {
        container.left = Val::Px(max_x);
        return;
    }

    container.left = Val::Px(new_left);
}
