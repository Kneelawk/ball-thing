mod level;
mod menu;
mod player;
mod util;

use crate::level::{LevelLoadedEvent, LevelsPlugin};
use crate::menu::MenuPlugin;
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
        .add_plugin(MenuPlugin)
        .add_startup_system(setup_physics)
        .add_systems((pause_game, state_respond, set_in_game))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, Ord, PartialOrd, PartialEq, Hash, States)]
enum AppState {
    #[default]
    MainMenu,
    Loading,
    PauseMenu,
    InGame,
}

fn setup_physics(mut physics: ResMut<RapierConfiguration>) {
    physics.physics_pipeline_active = false;
}

fn state_respond(
    mut windows: Query<&mut Window>,
    cur_state: Res<State<AppState>>,
    mut physics: ResMut<RapierConfiguration>,
) {
    let mut window = windows.single_mut();

    if cur_state.is_changed() {
        if cur_state.0 == AppState::InGame {
            window.cursor.visible = false;
            window.cursor.grab_mode = CursorGrabMode::Locked;
            physics.physics_pipeline_active = true;
        } else {
            window.cursor.visible = true;
            window.cursor.grab_mode = CursorGrabMode::None;
            physics.physics_pipeline_active = false;
        }
    }
}

fn pause_game(
    cur_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    key: Res<Input<KeyCode>>,
) {
    if cur_state.0 == AppState::InGame && key.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::PauseMenu);
    }
}

fn set_in_game(
    cur_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut level_load: EventReader<LevelLoadedEvent>,
) {
    if cur_state.0 == AppState::Loading {
        if let Some(_) = level_load.iter().next() {
            next_state.set(AppState::InGame);
        }
    }

    level_load.clear();
}
