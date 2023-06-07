use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

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
