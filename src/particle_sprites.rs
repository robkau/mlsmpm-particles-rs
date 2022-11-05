use bevy::prelude::*;

use crate::components::*;
use crate::defaults::*;

pub(super) fn update_sprites(mut particles: Query<(&mut Transform, &Position), With<ParticleTag>>) {
    // todo extra color based on velocity. (maybe acceleration?)
    particles.par_for_each_mut(PAR_BATCH_SIZE, |(mut transform, position)| {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;
    });
}
