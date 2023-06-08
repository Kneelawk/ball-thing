pub mod serial;

use crate::level::serial::{LevelAssetLoader, SerialLevel};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelState>()
            .add_asset::<SerialLevel>()
            .init_asset_loader::<LevelAssetLoader>()
            .add_startup_system(setup_level)
            .add_system(build_level_on_load.in_base_set(CoreSet::PreUpdate));
    }
}

pub fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(500.0).into()),
            material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
            ..default()
        })
        .with_children(|builder| {
            builder
                .spawn(Collider::cuboid(250.0, 0.1, 250.0))
                .insert(RigidBody::Fixed)
                .insert(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, -0.1, 0.0,
                )));
        });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, -2.0),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(RigidBody::Dynamic);
}

#[derive(Default, Debug, Clone, Resource)]
pub struct LevelState {
    pub handle: Option<Handle<SerialLevel>>,
}

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct LevelObject;

fn build_level_on_load(
    level_state: Res<LevelState>,
    old_objects: Query<Entity, With<LevelObject>>,
    mut asset_events: EventReader<AssetEvent<SerialLevel>>,
    assets: Res<Assets<SerialLevel>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(level_handle) = &level_state.handle {
        for event in asset_events.iter() {
            let (handle, update) = match event {
                AssetEvent::Created { handle } => (handle, true),
                AssetEvent::Modified { handle } => (handle, true),
                AssetEvent::Removed { handle } => (handle, false),
            };

            if handle == level_handle && update {
                if let Some(level) = assets.get(level_handle) {
                    // remove old objects if they're still around
                    for old in old_objects.iter() {
                        commands.entity(old).despawn();
                    }

                    level.spawn(&mut commands, &mut meshes, &mut materials);
                }
            }
        }
    }
}
