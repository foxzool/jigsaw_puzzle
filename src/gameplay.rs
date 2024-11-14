use crate::ui::BoardBackgroundImage;
use crate::Piece;
use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::basic::YELLOW;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use bevy::utils::HashSet;
use jigsaw_puzzle_generator::image::GenericImageView;
use jigsaw_puzzle_generator::{JigsawGenerator, JigsawPiece};
use log::debug;
use rand::Rng;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_generator)
        .add_event::<Shuffle>()
        .add_systems(Update, (move_piece, cancel_all_move, shuffle_pieces))
        .add_systems(
            PostUpdate,
            (
                spawn_piece.run_if(resource_changed::<JigsawPuzzleGenerator>),
                handle_tasks,
            ),
        )
        .add_observer(combine_together);
}

#[derive(Resource)]
pub struct OriginImage {
    pub image: Handle<Image>,
    pub size: Vec2,
}

fn setup_generator(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image_path = "raw.jpg";
    let generator = JigsawGenerator::from_path(image_path, 9, 6).expect("Failed to load image");

    // load image from dynamic image
    let image = Image::from_dynamic(
        generator.origin_image().clone(),
        true,
        RenderAssetUsages::RENDER_WORLD,
    );
    let image_size = image.size_f32();
    let image_handle = asset_server.add(image);
    commands.insert_resource(OriginImage {
        image: image_handle.clone(),
        size: image_size,
    });

    commands
        .spawn((
            Sprite::from_color(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.6)), image_size),
            BoardBackgroundImage,
            Visibility::Hidden,
        ))
        .with_children(|p| {
            p.spawn((
                Sprite::from_image(image_handle),
                Transform::from_xyz(0.0, 0.0, -1.0),
            ));
        });

    commands.insert_resource(JigsawPuzzleGenerator(generator));
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct JigsawPuzzleGenerator(pub JigsawGenerator);

#[derive(Component)]
struct CropTask(Task<CommandQueue>);

#[derive(Component)]
struct WhiteImage;

#[derive(Component)]
struct ColorImage;

/// Spawn the pieces of the jigsaw puzzle
fn spawn_piece(mut commands: Commands, generator: Res<JigsawPuzzleGenerator>) {
    if let Ok(template) = generator.generate(false) {
        let thread_pool = AsyncComputeTaskPool::get();
        for piece in template.pieces.iter() {
            let template_clone = template.clone();
            let piece_clone = piece.clone();

            // let calc_position = random_position(&piece, window.resolution.size(), camera.scale);
            let calc_position = init_position(piece, template.origin_image.dimensions());
            let entity = commands
                .spawn((
                    Piece(piece.clone()),
                    MoveTogether::default(),
                    Transform::from_xyz(calc_position.x, calc_position.y, piece.index as f32),
                    Visibility::Visible,
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

            let task = thread_pool.spawn(async move {
                let mut command_queue = CommandQueue::default();
                let cropped_image = piece_clone.crop(&template_clone.origin_image);
                let white_image = piece_clone.fill_white(&cropped_image);

                command_queue.push(move |world: &mut World| {
                    let mut assets = world.resource_mut::<Assets<Image>>();
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
                            piece_clone.crop_width as f32,
                            piece_clone.crop_height as f32,
                        )),
                        ..default()
                    };

                    let color_id = world
                        .spawn((
                            ColorImage,
                            color_sprite,
                            Transform::from_xyz(
                                -piece_clone.calc_offset().0,
                                piece_clone.calc_offset().1,
                                0.0,
                            ),
                        ))
                        .id();
                    let white_sprite = Sprite {
                        image: white_image,
                        anchor: Anchor::TopLeft,
                        custom_size: Some(Vec2::new(
                            piece_clone.crop_width as f32,
                            piece_clone.crop_height as f32,
                        )),
                        ..default()
                    };
                    let white_id = world
                        .spawn((
                            WhiteImage,
                            white_sprite,
                            Transform::from_xyz(
                                -piece_clone.calc_offset().0,
                                piece_clone.calc_offset().1,
                                -1.0,
                            ),
                        ))
                        .id();

                    world
                        .entity_mut(entity)
                        .add_children(&[color_id, white_id])
                        .remove::<CropTask>();
                });

                command_queue
            });

            commands.entity(entity).insert(CropTask(task));
        }
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

fn handle_tasks(mut commands: Commands, mut crop_tasks: Query<&mut CropTask>) {
    for mut task in &mut crop_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
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
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
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
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
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
    camera_query: Single<(&Camera, &GlobalTransform)>,
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
    camera: Single<&OrthographicProjection, With<Camera2d>>,
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
