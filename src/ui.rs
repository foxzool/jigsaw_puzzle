use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Component)]
pub struct BoardBackgroundImage;
