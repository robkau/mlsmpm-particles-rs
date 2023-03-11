use crate::prelude::*;

pub(crate) fn update_sprites(mut particles: Query<(&mut Transform, &Position), With<ParticleTag>>) {
    // todo extra color based on velocity. (maybe acceleration?)
    particles
        .par_iter_mut()
        .for_each_mut(|(mut transform, position)| {
            transform.translation.x = position.0.x;
            transform.translation.y = position.0.y;
        });
}
