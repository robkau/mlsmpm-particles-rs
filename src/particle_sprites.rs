use bevy::prelude::*;

use crate::components::*;
use crate::defaults::*;

pub(super) fn update_sprites(mut particles: Query<(&mut Transform, &Position), With<ParticleTag>>) {
    // todo adjust size relative to mass, min/max size determined by grid+window sizes
    // todo color based on velocity. (maybe acceleration?)
    // todo color based on constitutive model. (or initial texture.)
    particles.par_for_each_mut(PAR_BATCH_SIZE, |(mut transform, position)| {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;
    });
}
