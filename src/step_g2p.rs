use bevy::math::Mat2;
use bevy::prelude::*;

use crate::components::*;
use crate::defaults::*;
use crate::grid::*;
use crate::world::*;

// G2P MPM step
pub(super) fn grid_to_particles(
    world: Res<WorldState>,
    grid: Res<Grid>,
    mut particles: Query<(&mut Position, &mut Velocity, &mut AffineMomentum), With<ParticleTag>>,
) {
    particles.par_for_each_mut(
        PAR_BATCH_SIZE,
        |(mut position, mut velocity, mut affine_momentum)| {
            //// reset particle velocity. we calculate it from scratch each step using the grid
            velocity.0 = Vec2::ZERO;

            let cell_x: u32 = position.0.x as u32;
            let cell_y: u32 = position.0.y as u32;
            let cell_diff = Vec2::new(
                position.0.x - cell_x as f32 - 0.5,
                position.0.y - cell_y as f32 - 0.5,
            );
            let weights = quadratic_interpolation_weights(cell_diff);

            // affine per-particle momentum matrix from APIC / MLS-MPM.
            // see APIC paper (https://web.archive.org/web/20190427165435/https://www.math.ucla.edu/~jteran/papers/JSSTS15.pdf), page 6
            // below equation 11 for clarification. this is calculating C = B * (D^-1) for APIC equation 8,
            // where B is calculated in the inner loop at (D^-1) = 4 is a constant when using quadratic interpolation functions
            let mut b = Mat2::ZERO;
            // for all surrounding 9 cells
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
                    let weighted_velocity = grid.cells[cell_at_index].velocity * weight;
                    b += weighted_velocity_and_cell_dist_to_term(weighted_velocity, cell_dist);
                    velocity.0 += weighted_velocity;
                }
            }

            affine_momentum.0 = b * 4.0;

            // advect particles
            position.0 += velocity.0 * world.dt;

            //// safety clamp to ensure particles don't exit simulation domain
            position.0.x = position.0.x.clamp(1.0, (grid.width - 2) as f32);
            position.0.y = position.0.y.clamp(1.0, (grid.width - 2) as f32);

            // predictive boundary velocity cap
            let wall_min: f32 = 3.0;
            let wall_max: f32 = (grid.width - 1) as f32 - wall_min;

            // apply boundary conditions about 0.1 seconds before reaching edge
            let dt_multiplier = 0.1 / world.dt;
            let position_next = position.0 + velocity.0 * world.dt * dt_multiplier;
            if position_next.x < wall_min {
                velocity.0.x += wall_min - position_next.x;
            }
            if position_next.x > wall_max {
                velocity.0.x += wall_max - position_next.x;
            }
            if position_next.y < wall_min {
                velocity.0.y += wall_min - position_next.y;
            }
            if position_next.y > wall_max {
                velocity.0.y += wall_max - position_next.y;
            }
        },
    );
}
