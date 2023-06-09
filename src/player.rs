use crate::level::{LevelLoadedEvent, LevelRemovedEvent, PlayerSpawnPoint};
use crate::AppState;
use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomSettings};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

const MOUSE_SPEED: f32 = 0.0025;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera)
            .add_system(add_player.run_if(no_player_exists))
            .add_system(remove_player.run_if(player_exists))
            .add_systems(
                (rotate_camera, jump_player, move_player)
                    .in_set(OnUpdate(AppState::InGame))
                    .distributive_run_if(player_exists),
            )
            .add_system(
                move_camera
                    .in_base_set(CoreSet::PostUpdate)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn(PlayerCamera::default())
        .insert(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        })
        .insert(BloomSettings {
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        });
}

pub fn no_player_exists(player: Query<Entity, With<Player>>) -> bool {
    player.is_empty()
}

pub fn player_exists(player: Query<Entity, With<Player>>) -> bool {
    !no_player_exists(player)
}

pub fn add_player(
    mut level_load: EventReader<LevelLoadedEvent>,
    spawnpoint: Query<&Transform, With<PlayerSpawnPoint>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(level) = level_load.iter().next() {
        let spawnpoint = match spawnpoint.get(level.entities.spawn) {
            Ok(trans) => trans.translation,
            Err(_) => Vec3::new(0.0, 0.5, 0.0),
        };

        let player_transform = Transform::from_translation(spawnpoint);
        commands
            .spawn(Player::default())
            .insert(PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius: 0.5,
                        sectors: 32,
                        stacks: 32,
                    }
                    .into(),
                ),
                material: materials.add(Color::rgb(0.0, 10.0, 12.0).into()),
                ..default()
            })
            .insert(Collider::ball(0.5))
            .insert(RigidBody::Dynamic)
            .insert(ExternalForce::default())
            .insert(Damping {
                angular_damping: 0.25,
                linear_damping: 0.25,
            })
            .insert(Sleeping::disabled())
            .insert(TransformBundle::from_transform(player_transform.clone()))
            .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
            .with_children(|builder| {
                builder.spawn(PointLightBundle {
                    point_light: PointLight {
                        intensity: 500.0,
                        shadows_enabled: true,
                        color: Color::rgb(0.0, 0.833, 1.0),
                        ..default()
                    },
                    ..default()
                });
            });
    }

    level_load.clear();
}

pub fn remove_player(
    mut level_remove: EventReader<LevelRemovedEvent>,
    players: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    if let Some(_) = level_remove.iter().next() {
        for player in players.iter() {
            commands.entity(player).despawn();
        }
    }

    level_remove.clear();
}

pub fn rotate_camera(mut camera: Query<&mut PlayerCamera>, mut mouse: EventReader<MouseMotion>) {
    let mut camera = camera.single_mut();

    for mouse in mouse.iter() {
        camera.yaw += -mouse.delta.x * MOUSE_SPEED;
        camera.pitch =
            (camera.pitch - mouse.delta.y * MOUSE_SPEED).clamp(-PI / 2.0 + 0.001, PI / 2.0 - 0.001);
    }
}

pub fn move_player(
    mut force: Query<&mut ExternalForce, With<Player>>,
    camera: Query<&PlayerCamera>,
    key: Res<Input<KeyCode>>,
) {
    let camera = camera.single();

    let mut movement = Vec3::default();
    if key.pressed(KeyCode::W) {
        movement += camera.get_looking();
    }
    if key.pressed(KeyCode::S) {
        movement += -camera.get_looking();
    }

    if key.pressed(KeyCode::A) {
        movement += -Vec3::cross(camera.get_looking(), Vec3::Y);
    }
    if key.pressed(KeyCode::D) {
        movement += Vec3::cross(camera.get_looking(), Vec3::Y);
    }

    let torque = Vec3::cross(Vec3::Y, movement.normalize_or_zero());
    let mut force = force.single_mut();
    force.torque = torque;
}

pub fn jump_player(
    mut player: Query<(Entity, &mut ExternalForce), With<Player>>,
    mut events: EventReader<ContactForceEvent>,
    key: Res<Input<KeyCode>>,
) {
    let (player_entity, mut player_force) = player.single_mut();

    let mut jumping = false;
    for event in events.iter() {
        if (player_entity == event.collider1 || player_entity == event.collider2)
            && Vec3::dot(event.max_force_direction, Vec3::Y) > 0.8
        {
            if key.pressed(KeyCode::Space) {
                jumping = true;
            }
        }
    }

    if jumping {
        player_force.force = Vec3::new(0.0, 150.0, 0.0);
    } else {
        player_force.force = Vec3::ZERO;
    }
}

pub fn move_camera(
    mut camera: Query<(&mut Transform, &PlayerCamera)>,
    player: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    let (mut transform, player_camera) = camera.single_mut();
    let player = player.single();

    *transform = calculate_camera_transform(player.translation, player_camera);
}

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct Player;

#[derive(Debug, Clone, Component)]
pub struct PlayerCamera {
    pitch: f32,
    yaw: f32,
    distance: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        PlayerCamera {
            pitch: -PI / 4.0,
            yaw: 0.0,
            distance: 5.0,
        }
    }
}

impl PlayerCamera {
    pub fn get_looking(&self) -> Vec3 {
        -Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos())
    }
}

fn calculate_camera_transform(player_pos: Vec3, player_camera: &PlayerCamera) -> Transform {
    let camera_offset = Vec3::new(
        player_camera.pitch.cos() * player_camera.yaw.sin(),
        -player_camera.pitch.sin(),
        player_camera.pitch.cos() * player_camera.yaw.cos(),
    ) * player_camera.distance;

    Transform::from_translation(player_pos + camera_offset).looking_at(player_pos, Vec3::Y)
}
