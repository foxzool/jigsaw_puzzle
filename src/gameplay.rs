use crate::NORMAL_BUTTON;
use crate::{despawn_screen, GameState};
use crate::{AppState, OriginImage, Piece, SelectGameMode, SelectPiece};
use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::basic::{GREEN, YELLOW};
use bevy::ecs::world::CommandQueue;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::time::Stopwatch;
use bevy::utils::HashSet;
use bevy::window::WindowMode;
use flume::{bounded, Receiver};
use jigsaw_puzzle_generator::image::GenericImageView;
use jigsaw_puzzle_generator::{GameMode, JigsawGenerator, JigsawPiece, JigsawTemplate};
use log::debug;
use rand::Rng;
use std::ops::DerefMut;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    // app state
    app.add_systems(OnEnter(GameState::Setup), setup_game)
        .add_systems(
            Update,
            change_to_generate
                .run_if(resource_changed::<OriginImage>)
                .run_if(in_state(GameState::Setup)),
        )
        .add_systems(OnEnter(AppState::Gameplay), enter_app_gameplay)
        .add_systems(OnExit(AppState::Gameplay), exit_app_gameplay);

    // generation piece
    app.add_systems(
        OnEnter(GameState::Generating),
        (setup_generator, setup_generating_ui, spawn_piece).chain(),
    )
    .add_systems(
        OnExit(GameState::Generating),
        despawn_screen::<OnGeneratingScreen>,
    )
    .add_systems(Update, (adjust_camera_on_added_sprite,))
    .add_systems(
        PostUpdate,
        (handle_tasks, count_spawned_piece).run_if(in_state(GameState::Generating)),
    );

    // pause logic
    app.add_systems(OnEnter(GameState::Pause), setup_pause_ui)
        .add_systems(OnExit(GameState::Pause), despawn_screen::<OnPauseScreen>)
        .add_systems(Update, back_to_game.run_if(in_state(GameState::Pause)));

    // play logic
    app.add_event::<Shuffle>()
        .add_systems(OnEnter(GameState::Play), setup_game_ui)
        .add_event::<AdjustScale>()
        .add_event::<ToggleBackgroundHint>()
        .add_event::<TogglePuzzleHint>()
        .add_event::<ToggleEdgeHint>()
        .add_systems(
            Update,
            (
                update_game_time,
                move_piece,
                cancel_all_move,
                shuffle_pieces,
                adjust_camera_scale,
                handle_keyboard_input,
                handle_mouse_wheel_input,
                handle_toggle_background_hint,
                handle_toggle_puzzle_hint,
                exit_fullscreen_on_esc,
                handle_puzzle_hint,
            )
                .run_if(in_state(GameState::Play)),
        )
        .add_observer(combine_together);

    // finish
    app.add_systems(
        OnEnter(GameState::Finish),
        (despawn_screen::<OnPlayScreen>, setup_finish_ui),
    )
    .add_systems(OnExit(GameState::Finish), despawn_screen::<OnFinishScreen>);
}

#[derive(Component)]
struct OnFinishScreen;

fn setup_finish_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_timer: Res<GameTimer>,
    select_game_mode: Res<SelectGameMode>,
    select_piece: Res<SelectPiece>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(149, 165, 166)),
            OnFinishScreen,
        ))
        .with_children(|p| {
            let font = asset_server.load("fonts/MinecraftEvenings.ttf");
            let text_font = TextFont {
                font: font.clone(),
                font_size: 60.0,
                ..default()
            };

            p.spawn((Text::new("Finish"), TextColor(Color::BLACK), text_font));
            p.spawn((
                Text::new(format!(
                    "{} pieces {}",
                    select_piece.to_string(),
                    select_game_mode.to_string()
                )),
                TextColor(Color::BLACK),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
            ));
            p.spawn((
                Text::new(format!("Use time: {}", game_timer.to_string())),
                TextColor(Color::BLACK),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
            ));
            p.spawn((
                Button,
                Node {
                    width: Val::Px(100.0),
                    height: Val::Px(40.0),
                    border: UiRect::all(Val::Px(5.0)),
                    margin: UiRect::all(Val::Px(5.0)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
                BackgroundColor(NORMAL_BUTTON),
            ))
            .with_child((
                Text::new("Menu"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ))
            .observe(
                |_trigger: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<AppState>>| {
                    next_state.set(AppState::MainMenu);
                },
            );

            p.spawn((
                Button,
                Node {
                    width: Val::Px(100.0),
                    height: Val::Px(40.0),
                    margin: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
                BackgroundColor(NORMAL_BUTTON),
            ))
            .with_child((
                Text::new("Again"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ))
            .observe(
                |_trigger: Trigger<Pointer<Click>>,
                 mut next_state: ResMut<NextState<GameState>>| {
                    next_state.set(GameState::Setup);
                },
            );
        });
}

fn setup_game(
    origin_image: Option<Res<OriginImage>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if origin_image.is_none() {
        debug!("Load default image");
        let image = asset_server.load("images/raw.jpg");
        commands.insert_resource(OriginImage(image));
    } else {
        game_state.set(GameState::Generating);
    }
}

fn change_to_generate(mut game_state: ResMut<NextState<GameState>>) {
    std::thread::sleep(Duration::from_secs(1));
    game_state.set(GameState::Generating);
}

fn enter_app_gameplay(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::Setup);
}

fn exit_app_gameplay(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::Idle);
}

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct GameTimer(pub Stopwatch);

impl std::fmt::Display for GameTimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elapsed = self.elapsed();
        let seconds = elapsed.as_secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        write!(
            f,
            "{}",
            format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
        )
    }
}

fn setup_generator(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    origin_image: Res<OriginImage>,
    select_game_mode: Res<SelectGameMode>,
    select_piece: Res<SelectPiece>,
) {
    let image = images.get(&origin_image.0).unwrap();
    let (columns, rows) = select_piece.to_columns_rows();
    let width = image.texture_descriptor.size.width;
    let height = image.texture_descriptor.size.height;
    let generator = JigsawGenerator::from_rgba8(width, height, &image.data, columns, rows)
        .expect("Failed to load image");

    commands
        .spawn((
            Sprite::from_color(
                Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.6)),
                Vec2::new(width as f32, height as f32),
            ),
            BoardBackgroundImage,
            Visibility::Hidden,
            OnPlayScreen,
        ))
        .with_children(|p| {
            p.spawn((
                Sprite::from_image(origin_image.0.clone()),
                Transform::from_xyz(0.0, 0.0, -1.0),
            ));
        });

    commands.insert_resource(JigsawPuzzleGenerator(generator));
}

#[derive(Component)]
pub struct OnGeneratingScreen;

#[derive(Debug, Resource, Deref, DerefMut, Clone)]
pub struct JigsawPuzzleGenerator(pub JigsawGenerator);

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct JigsawPuzzleTemplate(pub JigsawTemplate);

#[derive(Component)]
struct CropTask(Receiver<CommandQueue>);

#[derive(Component)]
struct WhiteImage;

#[derive(Component)]
struct ColorImage;

/// Spawn the pieces of the jigsaw puzzle
fn spawn_piece(mut commands: Commands, generator: Res<JigsawPuzzleGenerator>) {
    debug!("Start to generate pieces");
    let start = std::time::Instant::now();
    if let Ok(template) = generator.generate(GameMode::Classic, false) {
        // commands.insert_resource(JigsawPuzzleTemplate(template.clone()));
        let mut wait_crops = vec![];
        let (tx, rx) = bounded(template.pieces.len());
        for piece in template.pieces.iter() {
            let piece_clone = piece.clone();

            // let calc_position = random_position(&piece, window.resolution.size(), camera.scale);
            let calc_position = init_position(piece, template.origin_image.dimensions());
            let entity = commands
                .spawn((
                    Piece(piece.clone()),
                    MoveTogether::default(),
                    Transform::from_xyz(calc_position.x, calc_position.y, piece.index as f32),
                    Visibility::Visible,
                    OnPlayScreen,
                ))
                .observe(on_click_piece)
                .observe(on_move_end)
                .observe(on_drag_start)
                .observe(on_drag_end)
                .observe(on_add_move_start)
                .observe(on_remove_move_start)
                .observe(on_selected)
                .observe(on_not_selected)
                .id();

            wait_crops.push((entity, piece_clone));
            commands.entity(entity).insert(CropTask(rx.clone()));
        }

        if !wait_crops.is_empty() {
            let template_clone = template.clone();
            std::thread::spawn(move || {
                for (entity, piece) in wait_crops {
                    let mut command_queue = CommandQueue::default();
                    let cropped_image = piece.crop(&template_clone.origin_image);
                    let white_image = piece.fill_white(&cropped_image);
                    command_queue.push(move |mut world: &mut World| {
                        let mut assets = world.deref_mut().resource_mut::<Assets<Image>>();
                        let image = assets.add(Image::from_dynamic(
                            cropped_image,
                            true,
                            RenderAssetUsages::RENDER_WORLD,
                        ));
                        let white_image = assets.add(Image::from_dynamic(
                            white_image,
                            true,
                            RenderAssetUsages::RENDER_WORLD,
                        ));
                        let color_sprite = Sprite {
                            image,
                            anchor: Anchor::TopLeft,
                            custom_size: Some(Vec2::new(
                                piece.crop_width as f32,
                                piece.crop_height as f32,
                            )),
                            ..default()
                        };

                        let color_id = world
                            .spawn((
                                ColorImage,
                                color_sprite,
                                Transform::from_xyz(
                                    -piece.calc_offset().0,
                                    piece.calc_offset().1,
                                    0.0,
                                ),
                            ))
                            .id();
                        let white_sprite = Sprite {
                            image: white_image,
                            anchor: Anchor::TopLeft,
                            custom_size: Some(Vec2::new(
                                piece.crop_width as f32,
                                piece.crop_height as f32,
                            )),
                            ..default()
                        };
                        let white_id = world
                            .spawn((
                                WhiteImage,
                                white_sprite,
                                Transform::from_xyz(
                                    -piece.calc_offset().0,
                                    piece.calc_offset().1,
                                    -1.0,
                                ),
                            ))
                            .id();

                        world
                            .entity_mut(entity)
                            .add_children(&[color_id, white_id])
                            .remove::<CropTask>();
                    });

                    tx.send(command_queue).unwrap();
                }
            });
        }

        debug!("Time to generate pieces: {:?}", start.elapsed());
        commands.send_event(Shuffle::Random);
    };
}

/// Calculate the position of the piece in the world space
#[allow(dead_code)]
fn calc_position(piece: &JigsawPiece, origin_image_size: (u32, u32)) -> Vec2 {
    let (width, height) = origin_image_size;
    let image_top_left = (width as f32 / -2.0, height as f32 / 2.0);

    let x = piece.top_left_x as f32;
    let y = piece.top_left_y as f32;

    Vec2::new(image_top_left.0 + x, image_top_left.1 - y)
}

#[allow(dead_code)]
fn init_position(piece: &JigsawPiece, origin_image_size: (u32, u32)) -> Vec2 {
    let (width, height) = origin_image_size;
    let image_top_left = (width as f32 / -2.0, height as f32 / 2.0);
    Vec2::new(
        image_top_left.0 + piece.start_point.0,
        image_top_left.1 - piece.start_point.1,
    )
}

fn handle_tasks(mut commands: Commands, mut crop_tasks: Query<&CropTask>) {
    for task in crop_tasks.iter_mut() {
        for mut queue in task.0.try_iter() {
            commands.append(&mut queue);
        }
    }
}

fn count_spawned_piece(
    mut text: Single<&mut Text, With<PieceCount>>,
    generator: Res<JigsawPuzzleGenerator>,
    mut game_state: ResMut<NextState<GameState>>,
    q_pieces: Query<Entity, With<ColorImage>>,
) {
    let loaded_pieces = q_pieces.iter().count();
    text.0 = format!("{}/{}", loaded_pieces, generator.pieces_count());
    if loaded_pieces == generator.pieces_count() {
        game_state.set(GameState::Play)
    }
}

#[derive(Component)]
struct MoveStart {
    image_position: Transform,
    click_position: Vec2,
}

fn on_drag_start(
    trigger: Trigger<Pointer<DragStart>>,
    mut piece: Query<&mut Transform, With<Piece>>,
    camera: Single<(&Camera, &GlobalTransform), (With<Camera2d>, With<IsDefaultUiCamera>)>,
    mut commands: Commands,
) {
    if let Ok(mut transform) = piece.get_mut(trigger.entity()) {
        let click_position = trigger.event().pointer_location.position;
        let (camera, camera_global_transform) = camera.into_inner();
        let point = camera
            .viewport_to_world_2d(camera_global_transform, click_position)
            .unwrap();
        transform.translation.z = 100.0;
        commands.entity(trigger.entity()).insert(MoveStart {
            image_position: *transform,
            click_position: point,
        });
    }
}

fn on_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut image: Query<&mut Transform, (With<MoveStart>, With<Piece>)>,
    mut commands: Commands,
) {
    if let Ok(mut transform) = image.get_mut(trigger.entity()) {
        transform.translation.z = 0.0;
        commands.entity(trigger.entity()).remove::<MoveStart>();
        commands.trigger_targets(MoveEnd, vec![trigger.entity()]);
    }
}

fn on_click_piece(
    trigger: Trigger<Pointer<Click>>,
    mut image: Query<(&mut Transform, Option<&MoveStart>), With<Piece>>,
    camera: Single<(&Camera, &GlobalTransform), (With<Camera2d>, With<IsDefaultUiCamera>)>,
    mut commands: Commands,
) {
    if let Ok((mut transform, opt_moveable)) = image.get_mut(trigger.entity()) {
        let click_position = trigger.event().pointer_location.position;
        let (camera, camera_global_transform) = camera.into_inner();
        let point = camera
            .viewport_to_world_2d(camera_global_transform, click_position)
            .unwrap();

        if opt_moveable.is_some() {
            transform.translation.z = 0.0;
            commands.entity(trigger.entity()).remove::<MoveStart>();
            commands.trigger_targets(MoveEnd, vec![trigger.entity()]);
        } else {
            transform.translation.z = 100.0;
            commands.entity(trigger.entity()).insert(MoveStart {
                image_position: *transform,
                click_position: point,
            });
        }
    }
}

fn move_piece(
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform), With<IsDefaultUiCamera>>,
    moveable: Single<(&mut Transform, &MoveStart, &MoveTogether)>,
    mut other_piece: Query<&mut Transform, Without<MoveStart>>,
) {
    let (camera, camera_transform) = *camera_query;
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let (mut transform, move_start, move_together) = moveable.into_inner();
    let cursor_move = point - move_start.click_position;
    let move_end = move_start.image_position.translation + cursor_move.extend(0.0);
    let offset = move_end - transform.translation;
    transform.translation = move_end;

    for other in move_together.iter() {
        if let Ok(mut other_transform) = other_piece.get_mut(*other) {
            other_transform.translation += offset;
        }
    }
}

#[derive(Event)]
struct MoveEnd;

#[derive(Component, Deref, DerefMut, Default)]
pub struct MoveTogether(pub HashSet<Entity>);

fn on_move_end(
    trigger: Trigger<MoveEnd>,
    generator: Res<JigsawPuzzleGenerator>,
    mut query: Query<(Entity, &Piece, &mut Transform, &mut MoveTogether)>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut iter = query.iter_combinations_mut();
    let end_entity = trigger.entity();

    let mut all_entities = HashSet::default();
    let mut max_z = 0f32;
    while let Some([(e1, p1, transform1, together1), (e2, p2, transform2, together2)]) =
        iter.fetch_next()
    {
        let (mut target_transform, compare_transform, target, compare) = if e1 == end_entity {
            (transform1, transform2, p1, p2)
        } else if e2 == end_entity {
            (transform2, transform1, p2, p1)
        } else {
            continue;
        };

        // calculate the max z value if close enough
        if target_transform
            .translation
            .xy()
            .distance(compare_transform.translation.xy())
            < (target.crop_width.max(target.crop_height) as f32)
        {
            max_z = max_z.max(compare_transform.translation.z);
        }

        let target_loc = (
            target_transform.translation.x,
            target_transform.translation.y,
        );
        let compare_loc = (
            compare_transform.translation.x,
            compare_transform.translation.y,
        );

        let mut has_snapped = false;

        if target.is_on_the_left_side(compare, target_loc, compare_loc) {
            debug!("{} on the left side {}", target.index, compare.index);
            target_transform.translation.x = compare_transform.translation.x - target.width;
            target_transform.translation.y = compare_transform.translation.y;
            has_snapped = true;
        }

        if target.is_on_the_right_side(compare, target_loc, compare_loc) {
            debug!("{} on the right side {}", target.index, compare.index);
            target_transform.translation.x = compare_transform.translation.x + compare.width;
            target_transform.translation.y = compare_transform.translation.y;
            has_snapped = true;
        }

        if target.is_on_the_top_side(compare, target_loc, compare_loc) {
            debug!("{} on the top side {}", target.index, compare.index);
            target_transform.translation.x = compare_transform.translation.x;
            target_transform.translation.y = compare_transform.translation.y + target.height;
            has_snapped = true;
        }

        if target.is_on_the_bottom_side(compare, target_loc, compare_loc) {
            debug!("{} on the bottom side {}", target.index, compare.index);
            target_transform.translation.x = compare_transform.translation.x;
            target_transform.translation.y = compare_transform.translation.y - compare.height;
            has_snapped = true;
        }

        if has_snapped {
            let mut merged_set: HashSet<_> = together1.union(&together2).cloned().collect();
            merged_set.insert(e1);
            merged_set.insert(e2);

            all_entities.extend(merged_set);
        }
    }

    if all_entities.len() == generator.pieces_count() {
        debug!("All pieces have been merged");
        next_state.set(GameState::Finish);
    }

    if let Ok((_e, _p, mut transform, _together)) = query.get_mut(trigger.entity()) {
        transform.translation.z = max_z + 1.0;
    }

    commands.trigger(CombineTogether(all_entities));
}

#[derive(Event)]
struct CombineTogether(HashSet<Entity>);

fn combine_together(trigger: Trigger<CombineTogether>, mut query: Query<&mut MoveTogether>) {
    let entities: Vec<Entity> = trigger.event().0.iter().cloned().collect();
    let mut together_iter = query.iter_many_mut(&entities);
    while let Some(mut move_together) = together_iter.fetch_next() {
        move_together.0 = trigger.event().0.clone();
    }
}

fn cancel_all_move(
    key: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<MoveStart>>,
    mut commands: Commands,
) {
    if key.just_pressed(KeyCode::Escape) {
        for entity in query.iter() {
            commands.entity(entity).remove::<MoveStart>();
        }
    }
}

#[derive(Component)]
pub struct Selected;

fn on_selected(
    trigger: Trigger<OnInsert, Selected>,
    query: Query<&Children>,
    mut q_image: Query<&mut Transform, (With<ColorImage>, Without<WhiteImage>)>,
    mut w_image: Query<&mut Sprite, (With<WhiteImage>, Without<ColorImage>)>,
) {
    let children = query.get(trigger.entity()).unwrap();

    for child in children.iter() {
        if let Ok(mut transform) = q_image.get_mut(*child) {
            transform.translation.x -= 4.0;
            transform.translation.y += 4.0;
        }
        if let Ok(mut image) = w_image.get_mut(*child) {
            image.color = Color::Srgba(YELLOW);
        }
    }
}

fn on_not_selected(
    trigger: Trigger<OnRemove, Selected>,
    query: Query<&Children>,
    mut q_image: Query<&mut Transform, (With<ColorImage>, Without<WhiteImage>)>,
    mut w_image: Query<&mut Sprite, (With<WhiteImage>, Without<ColorImage>)>,
) {
    let children = query.get(trigger.entity()).unwrap();

    for child in children.iter() {
        if let Ok(mut transform) = q_image.get_mut(*child) {
            transform.translation.x += 4.0;
            transform.translation.y -= 4.0;
        }
        if let Ok(mut image) = w_image.get_mut(*child) {
            image.color = Color::Srgba(Srgba::WHITE);
        }
    }
}

fn on_add_move_start(
    trigger: Trigger<OnInsert, MoveStart>,
    query: Query<&MoveTogether>,
    mut commands: Commands,
) {
    let move_together = query.get(trigger.entity()).unwrap();
    commands.entity(trigger.entity()).insert(Selected);
    for entity in move_together.iter() {
        if entity == &trigger.entity() {
            continue;
        }
        commands.entity(*entity).insert(Selected);
    }
}

fn on_remove_move_start(
    trigger: Trigger<OnRemove, MoveStart>,
    query: Query<&MoveTogether>,
    mut commands: Commands,
) {
    let move_together = query.get(trigger.entity()).unwrap();
    commands.entity(trigger.entity()).remove::<Selected>();
    for entity in move_together.iter() {
        commands.entity(*entity).remove::<Selected>();
    }
}

/// Calculate a random position for the piece
#[allow(dead_code)]
fn random_position(piece: &JigsawPiece, window_size: Vec2, scale: f32) -> Vec2 {
    let half_width = window_size.x / 2.0 * scale;
    let half_height = window_size.y / 2.0 * scale;
    let min_x = -half_width + piece.crop_width as f32;
    let min_y = -half_height + piece.crop_height as f32;
    let max_x = half_width - piece.crop_width as f32;
    let max_y = half_height - piece.crop_height as f32;

    let mut rng = rand::thread_rng();
    let x = rng.gen_range(min_x..max_x);
    let y = rng.gen_range(min_y..max_y);
    Vec2::new(x, y)
}

/// Calculate an edge position for the piece
#[allow(dead_code)]
fn edge_position(piece: &JigsawPiece, window_size: Vec2, scale: f32) -> Vec2 {
    let half_width = window_size.x / 2.0 * scale;
    let half_height = window_size.y / 2.0 * scale;
    let min_y = -half_height + piece.crop_height as f32;
    let max_x = half_width - piece.crop_width as f32;

    let mut rng = rand::thread_rng();
    let ran_side = rng.gen_range(0..4);
    let (x, y) = match ran_side {
        // top
        0 => (rng.gen_range(-half_width..max_x), half_height),
        // right
        1 => (max_x, rng.gen_range(min_y..half_height)),
        // bottom
        2 => (rng.gen_range(-half_width..max_x), min_y),
        // left
        3 => (-half_width, rng.gen_range(min_y..half_height)),
        _ => (0.0, 0.0),
    };

    Vec2::new(x, y)
}

#[derive(Event)]
pub enum Shuffle {
    Random,
    Edge,
}

fn shuffle_pieces(
    mut shuffle_events: EventReader<Shuffle>,
    mut query: Query<(&Piece, &mut Transform)>,
    window: Single<&Window>,
    camera: Single<&OrthographicProjection, (With<Camera2d>, With<IsDefaultUiCamera>)>,
) {
    for event in shuffle_events.read() {
        match event {
            Shuffle::Random => {
                for (piece, mut transform) in &mut query.iter_mut() {
                    let random_pos =
                        random_position(&piece, window.resolution.size(), camera.scale);
                    transform.translation = random_pos.extend(piece.index as f32);
                }
            }
            Shuffle::Edge => {
                for (piece, mut transform) in &mut query.iter_mut() {
                    let edge_pos = edge_position(&piece, window.resolution.size(), camera.scale);
                    transform.translation = edge_pos.extend(piece.index as f32);
                }
            }
        }
    }
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

fn setup_generating_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    generator: Res<JigsawPuzzleGenerator>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnGeneratingScreen,
        ))
        .with_children(|p| {
            let font = asset_server.load("fonts/MinecraftEvenings.ttf");
            let text_font = TextFont {
                font: font.clone(),
                font_size: 55.0,
                ..default()
            };

            p.spawn((
                Text::new("Loading pieces...."),
                TextColor(Color::BLACK),
                text_font,
            ));
            p.spawn((
                Text::new(format!("0/{}", generator.pieces_count())),
                TextColor(Color::BLACK),
                PieceCount,
            ));
        });
}
#[derive(Component)]
struct OnPauseScreen;

fn setup_pause_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(149, 165, 166)),
            OnPauseScreen,
        ))
        .observe(
            |_trigger: Trigger<Pointer<Click>>, mut game_state: ResMut<NextState<GameState>>| {
                game_state.set(GameState::Play);
            },
        )
        .with_children(|p| {
            let font = asset_server.load("fonts/MinecraftEvenings.ttf");
            let text_font = TextFont {
                font: font.clone(),
                font_size: 55.0,
                ..default()
            };

            p.spawn((Text::new("Paused"), TextColor(Color::BLACK), text_font));
            p.spawn((
                Text::new("click or press ESC to continue"),
                TextColor(Color::BLACK),
            ));
        });
}

fn back_to_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Play);
    }
}

#[derive(Component)]
struct PieceCount;

#[derive(Component)]
struct OnPlayScreen;

fn setup_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_node: Query<Entity, With<MenuIcon>>,
) {
    if !q_node.is_empty() {
        return;
    }
    commands.insert_resource(GameTimer(Stopwatch::new()));

    // let background_color = MAROON.into();
    let root_node = commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            OnPlayScreen,
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
                        // exit button
                        builder
                            .spawn((
                                UiImage::new(asset_server.load("icons/cross.png")),
                                Node {
                                    height: Val::Px(40.),
                                    ..default()
                                },
                                MenuIcon,
                            ))
                            .observe(
                                |_trigger: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<GameState>>| {
                                    next_state.set(GameState::Finish);
                                },);

                        // shuffle button
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
                                        commands.send_event(AdjustScale(0.1));
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
                                        commands.send_event(AdjustScale(-0.1));
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
                    Text::new("00:00:00"),
                    TextColor(GREEN.into()),
                    TimerText,
                    Node {
                        margin: UiRect {
                            top: Val::Px(7.0),
                            right: Val::Px(10.0),
                            ..default()
                        },
                        ..default()
                    },
                ));

                // pause button
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
                ))
                .observe(
                    |_trigger: Trigger<Pointer<Click>>,
                     mut game_state: ResMut<NextState<GameState>>| {
                        game_state.set(GameState::Pause)
                    },
                );
                // fullscreen button
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

    commands.send_event(Shuffle::Random);
}

#[derive(Component)]
struct TimerText;

#[derive(Component)]
pub struct BoardBackgroundImage;

/// Adjust the camera to fit the image
fn adjust_camera_on_added_sprite(
    _sprite: Single<Entity, Added<BoardBackgroundImage>>,
    mut camera_2d: Single<&mut OrthographicProjection, (With<Camera2d>, With<IsDefaultUiCamera>)>,
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
    mut camera_2d: Single<&mut OrthographicProjection, (With<Camera2d>, With<IsDefaultUiCamera>)>,
) {
    for AdjustScale(scale) in event.read() {
        let new_scale = camera_2d.scale + scale;
        debug!("new scale: {}", new_scale);
        if (MIN_SCALE..=MAX_SCALE).contains(&new_scale) {
            camera_2d.scale = new_scale;
        }
    }
}

fn update_game_time(
    mut game_timer: ResMut<GameTimer>,
    time: Res<Time>,
    mut text: Single<&mut Text, With<TimerText>>,
) {
    game_timer.tick(time.delta());
    text.0 = game_timer.to_string();
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
) {
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
    } else if keyboard_input.just_pressed(KeyCode::KeyQ) {
        game_state.set(GameState::Finish);
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
    // let aspect_ratio = origin_image.size.x / origin_image.size.y;

    commands
        .entity(*small_hint_image)
        .insert((
            UiImage::new(origin_image.0.clone()),
            Node {
                width: Val::Px(400.0),
                // aspect_ratio: Some(aspect_ratio),
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
