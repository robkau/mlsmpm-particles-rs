use bevy::prelude::*;
use bevy::tasks::ComputeTaskPool;

use crate::components::*;
use crate::defaults::*;
use crate::grid::*;
use crate::world::*;

pub(super) fn update_cells(
    pool: Res<ComputeTaskPool>,
    world: Res<WorldState>,
    grid: Res<Grid>,
    mut particles: Query<
        (
            &Position,
            &Velocity,
            &Mass,
            &AffineMomentum,
            &mut CellMassMomentumContributions,
        ),
        With<ParticleTag>,
    >,
) {
    let num_particles = particles.iter().count();
    if num_particles < 1 {
        return;
    }
    particles.par_for_each_mut(
        &pool,
        PAR_BATCH_SIZE,
        |(position, velocity, mass, affine_momentum, mut mmc)| {
            let cell_x: u32 = position.0.x as u32;
            let cell_y: u32 = position.0.y as u32;
            let cell_diff = Vec2::new(
                position.0.x - cell_x as f32 - 0.5,
                position.0.y - cell_y as f32 - 0.5,
            );
            let weights = quadratic_interpolation_weights(cell_diff);

            //collect momentum changes for surrounding 9 cells.
            for gx in 0..3 {
                for gy in 0..3 {
                    let weight = weights[gx].x * weights[gy].y;
                    let cell_pos_x = (cell_x as i32 + gx as i32) - 1;
                    let cell_pos_y = (cell_y as i32 + gy as i32) - 1;
                    let cell_dist = Vec2::new(
                        cell_pos_x as f32 - position.0.x + 0.5,
                        cell_pos_y as f32 - position.0.y + 0.5,
                    );
                    let cell_at_index = grid.index_at(cell_pos_x as usize, cell_pos_y as usize);

                    let q = affine_momentum.0 * cell_dist;
                    let mass_contrib = weight * mass.0;
                    // mass and momentum update
                    mmc.0[gx + 3 * gy] = GridMassAndMomentumChange(
                        cell_at_index,
                        mass_contrib,
                        (velocity.0 + q) * mass_contrib,
                    );
                }
            }
        },
    );
}

// todo look into replacing CellMassMomentumContributions with bevy events ..
// .. after this one is done https://github.com/bevyengine/bevy/issues/2648
pub(super) fn apply_update_cell_computations(
    mut grid: ResMut<Grid>,
    particles: Query<(&CellMassMomentumContributions, ), With<ParticleTag>>,
) {
    particles.for_each(|mmc| {
        for change in mmc.0.0.iter() {
            grid.cells[change.0].mass += change.1;
            grid.cells[change.0].velocity += change.2;
        }
    });
}
