use crate::level::LevelObject;
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

#[derive(Debug, Copy, Clone, Default)]
pub struct LevelAssetLoader;

impl AssetLoader for LevelAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let level: SerialLevel = match knuffel::parse(
                &load_context.path().to_string_lossy(),
                &String::from_utf8_lossy(bytes),
            ) {
                Ok(res) => res,
                Err(err) => {
                    error!("{:?}", miette::Report::new(err));
                    anyhow::bail!("Error loading level")
                }
            };
            load_context.set_default_asset(LoadedAsset::new(level));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level.kdl"]
    }
}

#[derive(Debug, Clone, knuffel::Decode, TypeUuid)]
#[uuid = "a7b66c53-c270-49eb-a822-822246b6e56a"]
pub struct SerialLevel {
    #[knuffel(children(name = "cube"))]
    cubes: Vec<SerialCube>,
}

impl SerialLevel {
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        for cube in self.cubes.iter() {
            cube.spawn(commands, meshes, materials);
        }
    }
}

pub trait SerialObject {
    fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    );
}

#[derive(Debug, Clone, knuffel::Decode)]
pub struct SerialCube {
    #[knuffel(child)]
    pos: SerialVec,

    #[knuffel(children(name = "rot"))]
    rotations: Vec<SerialRotation>,

    #[knuffel(argument)]
    size: f32,
}

impl SerialObject for SerialCube {
    fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let mut rotation = Quat::default();
        for rot in self.rotations.iter() {
            rotation = rotation.mul_quat((*rot).into());
        }

        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: self.size })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(self.pos.into()).with_rotation(rotation),
                ..default()
            })
            .insert(LevelObject)
            .insert(Collider::cuboid(
                self.size / 2.0,
                self.size / 2.0,
                self.size / 2.0,
            ))
            .insert(RigidBody::Dynamic);
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
pub struct SerialVec {
    #[knuffel(argument)]
    x: f32,
    #[knuffel(argument)]
    y: f32,
    #[knuffel(argument)]
    z: f32,
}

impl From<SerialVec> for Vec3 {
    fn from(value: SerialVec) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}
