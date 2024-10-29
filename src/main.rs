use bevy::prelude::*;
use jigsaw_puzzle::PuzzlePlugin;

fn main() {
    App::new().add_plugins(PuzzlePlugin).run();
}
