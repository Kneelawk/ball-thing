use crate::level::logic::DeathObject;
use crate::level::{LevelObject, LevelPertinentEntities, PlayerSpawnPoint};
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

#[derive(Debug, Copy, Clone, Default)]
pub struct LevelAssetLoader;

impl AssetLoader for LevelAssetLoader {
    type Asset = SerialLevel;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut str = String::new();
            reader.read_to_string(&mut str).await?;
            let level: SerialLevel =
                match knuffel::parse(&load_context.path().to_string_lossy(), &str) {
                    Ok(res) => res,
                    Err(err) => {
                        error!("{:?}", miette::Report::new(err));
                        anyhow::bail!("Error loading level")
                    }
                };

            Ok(level)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level.kdl"]
    }
}

pub struct SpawnArgs<'a, 'w, 's, 'r1, 'r2> {
    pub commands: &'a mut Commands<'w, 's>,
    pub meshes: &'a mut ResMut<'r1, Assets<Mesh>>,
    pub materials: &'a mut ResMut<'r2, Assets<StandardMaterial>>,
}

#[derive(Debug, Clone, knuffel::Decode, TypeUuid, Asset, TypePath)]
#[uuid = "a7b66c53-c270-49eb-a822-822246b6e56a"]
pub struct SerialLevel {
    #[knuffel(child)]
    spawn: SerialSpawnPoint,

    #[knuffel(children(name = "cube"))]
    cubes: Vec<SerialCube>,

    #[knuffel(children(name = "plane"))]
    planes: Vec<SerialPlane>,

    #[knuffel(children(name = "death_plane"))]
    death_planes: Vec<SerialDeathPlane>,
}

impl SerialLevel {
    pub fn spawn(&self, args: &mut SpawnArgs) -> LevelPertinentEntities {
        for cube in self.cubes.iter() {
            cube.spawn(args);
        }

        for plane in self.planes.iter() {
            plane.spawn(args);
        }

        for death_plane in self.death_planes.iter() {
            death_plane.spawn(args);
        }

        let spawn = self.spawn.spawn(args);

        LevelPertinentEntities { spawn }
    }
}

pub trait SerialObject {
    fn spawn(&self, args: &mut SpawnArgs) -> Entity;
}

#[derive(Debug, Clone, knuffel::Decode)]
pub struct SerialSpawnPoint {
    #[knuffel(child)]
    pos: SerialVec3,
}

impl SerialObject for SerialSpawnPoint {
    fn spawn(&self, args: &mut SpawnArgs) -> Entity {
        args.commands
            .spawn(PlayerSpawnPoint)
            .insert(LevelObject)
            .insert(TransformBundle::from_transform(
                Transform::from_translation(self.pos.into()),
            ))
            .id()
    }
}

#[derive(Debug, Clone, knuffel::Decode)]
pub struct SerialDeathPlane {
    #[knuffel(child)]
    pos: SerialVec3,

    #[knuffel(argument)]
    size: f32,
}

impl SerialObject for SerialDeathPlane {
    fn spawn(&self, args: &mut SpawnArgs) -> Entity {
        args.commands
            .spawn(DeathObject)
            .insert(LevelObject)
            .insert(TransformBundle::from_transform(
                Transform::from_translation(
                    Into::<Vec3>::into(self.pos) - Vec3::new(0.0, self.size / 2.0, 0.0),
                ),
            ))
            .insert(Collider::cuboid(
                self.size / 2.0,
                self.size / 2.0,
                self.size / 2.0,
            ))
            .insert(Sensor)
            .insert(RigidBody::Fixed)
            .id()
    }
}

#[derive(Debug, Clone, knuffel::Decode)]
pub struct SerialCube {
    #[knuffel(child)]
    pos: SerialVec3,

    #[knuffel(children(name = "rot"))]
    rotations: Vec<SerialRotation>,

    #[knuffel(argument)]
    size: f32,
}

impl SerialObject for SerialCube {
    fn spawn(&self, args: &mut SpawnArgs) -> Entity {
        let mut rotation = Quat::default();
        for rot in self.rotations.iter() {
            rotation = rotation.mul_quat((*rot).into());
        }

        args.commands
            .spawn(PbrBundle {
                mesh: args.meshes.add(Mesh::from(shape::Cube { size: self.size })),
                material: args.materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(self.pos.into()).with_rotation(rotation),
                ..default()
            })
            .insert(LevelObject)
            .insert(Collider::cuboid(
                self.size / 2.0,
                self.size / 2.0,
                self.size / 2.0,
            ))
            .insert(RigidBody::Dynamic)
            .id()
    }
}

#[derive(Debug, Clone, knuffel::Decode)]
pub struct SerialPlane {
    #[knuffel(child)]
    pos: SerialVec3,

    #[knuffel(children(name = "rot"))]
    rotations: Vec<SerialRotation>,

    #[knuffel(argument)]
    size: f32,

    #[knuffel(argument)]
    size2: Option<f32>,
}

impl SerialObject for SerialPlane {
    fn spawn(&self, args: &mut SpawnArgs) -> Entity {
        let mut rotation = Quat::default();
        for rot in self.rotations.iter() {
            rotation = rotation.mul_quat((*rot).into());
        }

        let size = if let Some(size2) = self.size2 {
            Vec2::new(self.size, size2)
        } else {
            Vec2::new(self.size, self.size)
        };

        args.commands
            .spawn(LevelObject)
            .insert(SpatialBundle {
                transform: Transform::from_translation(self.pos.into()).with_rotation(rotation),
                ..Default::default()
            })
            .with_children(|builder| {
                builder.spawn(PbrBundle {
                    mesh: args.meshes.add(shape::Quad::new(size).into()),
                    material: args.materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
                    transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
                    ..default()
                });
                builder
                    .spawn(Collider::cuboid(size.x / 2.0, 0.1, size.y / 2.0))
                    .insert(RigidBody::Fixed)
                    .insert(TransformBundle::from_transform(Transform::from_xyz(
                        0.0, -0.1, 0.0,
                    )));
            })
            .id()
    }
}

#[derive(Debug, Copy, Clone, knuffel::Decode)]
pub struct SerialRotation {
    /// Rotation axis
    #[knuffel(argument)]
    axis: SerialAxis,

    /// Angle in degrees (for human readability)
    #[knuffel(argument)]
    angle: f32,
}

#[derive(Debug, Copy, Clone, knuffel::DecodeScalar)]
pub enum SerialAxis {
    X,
    Y,
    Z,
}

impl From<SerialRotation> for Quat {
    fn from(value: SerialRotation) -> Self {
        match value.axis {
            SerialAxis::X => Quat::from_rotation_x(value.angle / 180.0 * PI),
            SerialAxis::Y => Quat::from_rotation_y(value.angle / 180.0 * PI),
            SerialAxis::Z => Quat::from_rotation_z(value.angle / 180.0 * PI),
        }
    }
}

#[derive(Debug, Copy, Clone, knuffel::Decode)]
pub struct SerialVec3 {
    #[knuffel(argument)]
    x: f32,
    #[knuffel(argument)]
    y: f32,
    #[knuffel(argument)]
    z: f32,
}

impl From<SerialVec3> for Vec3 {
    fn from(value: SerialVec3) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}
