mod level;
mod player;

use crate::level::LevelsPlugin;
use crate::player::PlayerPlugin;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_state::<AppState>()
        .add_plugin(LevelsPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_physics)
        .add_system(mouse_grab)
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
    cur_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut physics: ResMut<RapierConfiguration>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if cur_state.0 == AppState::Menu && mouse.just_pressed(MouseButton::Left) {
        next_state.set(AppState::InGame);
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
        physics.physics_pipeline_active = true;
    }

    if cur_state.0 == AppState::InGame && key.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
        physics.physics_pipeline_active = false;
    }
}
