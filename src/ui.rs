use crate::generator::JigsawPuzzleGenerator;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, adjust_camera_on_added_sprite);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
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
        let target_scale = scale / 0.8;
        camera_2d.scale = target_scale;
        commands.entity(entity).insert(Visibility::Hidden);
    }
}
