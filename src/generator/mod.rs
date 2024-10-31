use crate::ui::BoardBackgroundImage;
use crate::JigsawTile;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use jigsaw_puzzle_generator::image::GenericImageView;
use jigsaw_puzzle_generator::{JigsawGenerator, JigsawPiece};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_generator).add_systems(
        Update,
        (
            spawn_piece.run_if(resource_changed::<JigsawPuzzleGenerator>),
            handle_tasks,
        ),
    );
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

fn spawn_piece(mut commands: Commands, generator: Res<JigsawPuzzleGenerator>) {
    if let Ok(template) = generator.generate(false) {
        let thread_pool = AsyncComputeTaskPool::get();
        for piece in template.pieces.iter() {
            let template_clone = template.clone();
            let piece_clone = piece.clone();
            let calc_position = calc_position(&piece, template.origin_image.dimensions());
            let entity = commands
                .spawn((
                    JigsawTile { index: piece.index },
                    Transform::from_xyz(calc_position.0, calc_position.1, 1.0),
                ))
                .id();

            let task = thread_pool.spawn(async move {
                let cropped_image = piece_clone.crop(&template_clone.origin_image);
                let mut command_queue = CommandQueue::default();

                command_queue.push(move |world: &mut World| {
                    let asset_server = world.resource::<AssetServer>();
                    let image = asset_server.add(Image::from_dynamic(
                        cropped_image,
                        true,
                        RenderAssetUsages::RENDER_WORLD,
                    ));
                    let sprite = Sprite {
                        image,
                        anchor: Anchor::TopLeft,
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

fn calc_position(piece: &JigsawPiece, origin_image_size: (u32, u32)) -> (f32, f32) {
    let (width, height) = origin_image_size;
    let image_top_left = (width as f32 / -2.0, height as f32 / 2.0);

    let x = piece.top_left_x as f32;
    let y = piece.top_left_y as f32;

    (image_top_left.0 + x, image_top_left.1 - y)
}

fn handle_tasks(mut commands: Commands, mut crop_tasks: Query<&mut CropTask>) {
    for mut task in &mut crop_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
    }
}
