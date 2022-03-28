use bevy::prelude::*;

use crate::components::*;
use crate::world::*;

pub(super) fn delete_old_entities(
    mut commands: Commands,
    world: Res<WorldState>,
    aged_entities: Query<(Entity, &CreatedAt, &MaxAge)>,
) {
    aged_entities.for_each(|(id, created_at, max_age)| {
        if world.current_tick > created_at.0 + max_age.0 {
            commands.entity(id).despawn();
        }
    });
}
