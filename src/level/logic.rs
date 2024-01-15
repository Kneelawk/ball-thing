//! Logic for level objects.

use crate::level::PlayerSpawnPoint;
use crate::player::Player;
use crate::AppState;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn setup(app: &mut App) {
    app.add_system(player_death.run_if(in_state(AppState::InGame)));
}

/// An object that kills the player and resets the player's position to the spawn point.
#[derive(Default, Debug, Copy, Clone, Component)]
pub struct DeathObject;

/// Actually handles death object collision.
fn player_death(
    mut player: Query<
        (Entity, &mut Transform, &mut Velocity),
        (With<Player>, Without<PlayerSpawnPoint>),
    >,
    spawnpoint: Query<&Transform, (With<PlayerSpawnPoint>, Without<Player>)>,
    death_objects: Query<With<DeathObject>>,
    mut events: EventReader<CollisionEvent>,
) {
    let (player_entity, mut player_transform, mut player_velocity) = player.single_mut();

    for event in events.iter() {
        if let CollisionEvent::Started(entity_a, entity_b, _flags) = *event {
            if player_entity == entity_a || player_entity == entity_b {
                if death_objects.contains(entity_a) || death_objects.contains(entity_b) {
                    let spawnpoint = spawnpoint.single().translation;

                    player_transform.translation = spawnpoint;
                    player_velocity.angvel = Vec3::ZERO;
                    player_velocity.linvel = Vec3::ZERO;

                    break;
                }
            }
        }
    }

    events.clear();
}
