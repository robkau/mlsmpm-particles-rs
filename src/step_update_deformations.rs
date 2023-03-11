use std::ops::{Add, Mul};

use crate::prelude::*;

pub(crate) fn update_deformation_gradients(
    world: Res<WorldState>,
    mut particles_solid: Query<
        (&AffineMomentum, &mut NeoHookeanHyperElasticModel),
        With<ParticleTag>,
    >,
) {
    particles_solid
        .par_iter_mut()
        .for_each_mut(|(affine_momentum, mut pp)| {
            let deformation_new: Mat2 = Mat2::IDENTITY
                .add(affine_momentum.0.mul(world.dt))
                .mul_mat2(&pp.deformation_gradient);
            pp.deformation_gradient = deformation_new;

            // todo investigate plastic deformation that makes material want to keep its damaged state.
        });
}
