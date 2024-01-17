pub mod logic;
pub mod serial;

use crate::level::serial::{LevelAssetLoader, SerialLevel, SpawnArgs};
use bevy::prelude::*;

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelState>()
            .init_resource::<LevelStateOld>()
            .add_event::<LevelRemovedEvent>()
            .add_event::<LevelLoadedEvent>()
            .init_asset::<SerialLevel>()
            .init_asset_loader::<LevelAssetLoader>()
            .add_systems(PreUpdate, (remove_level, build_level_on_load));
        logic::setup(app);
    }
}

#[derive(Default, Debug, Clone, Resource)]
pub struct LevelState {
    pub handle: Option<Handle<SerialLevel>>,
}

#[derive(Default, Debug, Clone, Resource)]
pub struct LevelStateOld {
    pub handle: Option<Handle<SerialLevel>>,
}

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct LevelObject;

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct PlayerSpawnPoint;

#[derive(Default, Debug, Copy, Clone, Event)]
pub struct LevelRemovedEvent;

#[derive(Debug, Copy, Clone, Event)]
pub struct LevelLoadedEvent {
    pub entities: LevelPertinentEntities,
}

#[derive(Debug, Copy, Clone)]
pub struct LevelPertinentEntities {
    pub spawn: Entity,
}

/// Removes all level objects if the level is set to None.
fn remove_level(
    mut commands: Commands,
    level_state: Res<LevelState>,
    mut level_state_old: ResMut<LevelStateOld>,
    mut level_events: EventWriter<LevelRemovedEvent>,
    old_objects: Query<Entity, With<LevelObject>>,
) {
    if level_state.is_changed() {
        if level_state.handle.is_none() && level_state_old.handle.is_some() {
            level_state_old.handle = level_state.handle.clone();

            for old in old_objects.iter() {
                commands.entity(old).despawn_recursive();
            }

            level_events.send(LevelRemovedEvent);

            info!("Level removed.");
        }
    }
}

/// Adds level objects once the given level has loaded, re-adding if reloaded.
fn build_level_on_load(
    level_state: Res<LevelState>,
    mut level_state_old: ResMut<LevelStateOld>,
    old_objects: Query<Entity, With<LevelObject>>,
    mut level_events: EventWriter<LevelLoadedEvent>,
    mut asset_events: EventReader<AssetEvent<SerialLevel>>,
    assets: Res<Assets<SerialLevel>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(level_handle) = &level_state.handle {
        for event in asset_events.read() {
            let (&handle, update) = match event {
                AssetEvent::Added { id } => (id, false),
                AssetEvent::Modified { id } => (id, false),
                AssetEvent::Removed { id } => (id, false),
                AssetEvent::LoadedWithDependencies { id } => (id, true),
            };

            if handle == level_handle.id() && update {
                if let Some(level) = assets.get(level_handle) {
                    // remove old objects if they're still around
                    for old in old_objects.iter() {
                        commands.entity(old).despawn_recursive();
                    }

                    level_state_old.handle = Some(level_handle.clone());

                    let entities = level.spawn(&mut SpawnArgs {
                        commands: &mut commands,
                        meshes: &mut meshes,
                        materials: &mut materials,
                    });

                    level_events.send(LevelLoadedEvent { entities });

                    info!("Level Loaded.");

                    break;
                }
            }
        }
    }

    asset_events.clear();
}
