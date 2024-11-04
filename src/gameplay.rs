use crate::ui::BoardBackgroundImage;
use crate::Piece;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use jigsaw_puzzle_generator::image::GenericImageView;
use jigsaw_puzzle_generator::{JigsawGenerator, JigsawPiece};
use rand::Rng;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_generator)
        .add_systems(
            PostUpdate,
            (
                spawn_piece.run_if(resource_changed::<JigsawPuzzleGenerator>),
                handle_tasks,
            ),
        )
        .add_systems(Update, (move_piece,));
}

fn setup_generator(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image_path = "raw.jpg";
    let generator = JigsawGenerator::from_path(image_path, 9, 6).expect("Failed to load image");

    // load image from dynamic image
    let image = asset_server.add(Image::from_dynamic(
        generator.origin_image().clone(),
        true,
        RenderAssetUsages::RENDER_WORLD,
    ));

    commands.spawn((Sprite::from_image(image), BoardBackgroundImage));
    commands.insert_resource(JigsawPuzzleGenerator(generator));
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct JigsawPuzzleGenerator(pub JigsawGenerator);

#[derive(Component)]
struct CropTask(Task<CommandQueue>);

/// Spawn the pieces of the jigsaw puzzle
fn spawn_piece(
    mut commands: Commands,
    generator: Res<JigsawPuzzleGenerator>,
    // window: Single<&Window>,
    // camera: Single<&OrthographicProjection, With<Camera2d>>,
) {
    if let Ok(template) = generator.generate(false) {
        let thread_pool = AsyncComputeTaskPool::get();
        for piece in template.pieces.iter() {
            let template_clone = template.clone();
            let piece_clone = piece.clone();

            // let resolution = &window.resolution;
            // let calc_position = random_position(&piece, resolution.size(), camera.scale);
            let calc_position = calc_position(piece, template.origin_image.dimensions());
            let entity = commands
                .spawn((
                    Piece(piece.clone()),
                    Transform::from_xyz(calc_position.x, calc_position.y, piece.index as f32),
                ))
                .observe(on_click_piece)
                .id();

            let task = thread_pool.spawn(async move {
                let cropped_image = piece_clone.crop(&template_clone.origin_image);
                let mut command_queue = CommandQueue::default();

                command_queue.push(move |world: &mut World| {
                    let mut assets = world.resource_mut::<Assets<Image>>();
                    let image = assets.add(Image::from_dynamic(
                        cropped_image,
                        true,
                        RenderAssetUsages::RENDER_WORLD,
                    ));
                    let sprite = Sprite {
                        image,
                        anchor: Anchor::TopLeft,
                        custom_size: Some(Vec2::new(
                            piece_clone.crop_width as f32,
                            piece_clone.crop_height as f32,
                        )),
                        ..default()
                    };
                    world.entity_mut(entity).insert(sprite).remove::<CropTask>();
                });

                command_queue
            });

            commands.entity(entity).insert(CropTask(task));
        }
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

/// Calculate a random position for the piece
#[allow(dead_code)]
fn random_position(piece: &JigsawPiece, window_size: Vec2, scale: f32) -> Vec2 {
    let window_width = window_size.x / 2.0 * scale;
    let window_height = window_size.y / 2.0 * scale;
    let min_x = -window_width + piece.crop_width as f32;
    let min_y = -window_height + piece.crop_height as f32;
    let max_x = window_width - piece.crop_width as f32;
    let max_y = window_height - piece.crop_height as f32;

    let mut rng = rand::thread_rng();
    let x = rng.gen_range(min_x..max_x);
    let y = rng.gen_range(min_y..max_y);
    Vec2::new(x, y)
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

fn on_click_piece(
    trigger: Trigger<Pointer<Click>>,
    mut image: Query<(&mut Transform, Option<&MoveStart>)>,
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
        } else {
            transform.translation.z = 1.0;
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
    mut moveable: Query<(&Piece, &mut Transform, &MoveStart)>,
    query: Query<(&Piece, &Transform), Without<MoveStart>>,
) {
    let (camera, camera_transform) = *camera_query;
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for (move_piece, mut transform, move_start) in moveable.iter_mut() {
        let cursor_move = point - move_start.click_position;
        let move_end = move_start.image_position.translation + cursor_move.extend(0.0);

        for (piece, other_transform) in query.iter() {
            if piece.index == 0 {
                println!(
                    "{} {} {}",
                    piece.index,
                    move_piece.index,
                    other_transform
                        .translation
                        .truncate()
                        .distance(move_end.truncate())
                );
            }

            if close_to(
                piece,
                other_transform.translation.truncate(),
                move_end.truncate(),
            ) {
                if move_piece.left_edge == piece.right_edge {
                    println!("close {} {}", piece.index, move_piece.index);
                }
            }
        }

        transform.translation = move_end;
    }
}

fn close_to(p1: &Piece, p1_loc: Vec2, p2_loc: Vec2) -> bool {
    (p1_loc.x + p1.crop_width as f32 - p2_loc.x).abs() < 1.0
        && (p1_loc.y + p1.crop_height as f32 - p2_loc.y).abs() < 1.0
}
