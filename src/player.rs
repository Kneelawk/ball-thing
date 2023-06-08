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
        app.add_startup_system(setup_player)
            .add_systems((rotate_camera, move_player).in_set(OnUpdate(AppState::InGame)))
            .add_system(
                move_camera
                    .in_base_set(CoreSet::PostUpdate)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_transform = Transform::from_xyz(0.0, 0.5, 0.0);
    commands
        .spawn(Player)
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

    let player_camera = PlayerCamera::default();
    commands
        .spawn(player_camera.clone())
        .insert(Camera3dBundle {
            transform: calculate_camera_transform(player_transform.translation, &player_camera),
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

pub fn rotate_camera(mut camera: Query<&mut PlayerCamera>, mut mouse: EventReader<MouseMotion>) {
    let mut camera = camera.single_mut();

    for mouse in mouse.iter() {
        camera.yaw += -mouse.delta.x * MOUSE_SPEED;
        camera.pitch += -mouse.delta.y * MOUSE_SPEED;
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
            distance: 4.0,
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
