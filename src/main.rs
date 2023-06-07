mod level;
mod player;

use crate::level::setup_level;
use crate::player::{move_camera, move_player, rotate_camera, setup_player};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_state::<AppState>()
        .add_startup_systems((setup_physics, setup_level, setup_player))
        .add_system(mouse_grab)
        .add_systems((rotate_camera, move_player, move_camera).in_set(OnUpdate(AppState::InGame)))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, Ord, PartialOrd, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

fn setup_physics(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = false;
}

fn mouse_grab(
    mut windows: Query<&mut Window>,
    mut next_state: ResMut<NextState<AppState>>,
    mut physics: ResMut<RapierConfiguration>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        next_state.set(AppState::InGame);
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
        physics.physics_pipeline_active = true;
    }

    if key.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
        physics.physics_pipeline_active = false;
    }
}
