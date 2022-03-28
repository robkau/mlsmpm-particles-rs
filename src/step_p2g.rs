use std::ops::{Add, Mul, Sub};

use bevy::math::Mat2;
use bevy::prelude::*;
use bevy::tasks::ComputeTaskPool;

use crate::components::*;
use crate::defaults::*;
use crate::grid::*;
use crate::world::*;

pub(super) fn particles_to_grid_solids(
    pool: Res<ComputeTaskPool>,
    grid: Res<Grid>,
    world: Res<WorldState>,
    mut particles_solid: Query<
        (
            &Position,
            &Mass,
            &AffineMomentum,
            &ConstitutiveModelNeoHookeanHyperElastic,
            &mut CellMassMomentumContributions,
        ),
        With<ParticleTag>,
    >,
) {
    let num_particles = particles_solid.iter().count();
    if num_particles < 1 {
        return;
    }
    particles_solid.par_for_each_mut(
        &pool,
        PAR_BATCH_SIZE,
        |(position, mass, affine_momentum, pp, mut mmc)| {
            let cell_x: u32 = position.0.x as u32;
            let cell_y: u32 = position.0.y as u32;
            let cell_diff = Vec2::new(
                position.0.x - cell_x as f32 - 0.5,
                position.0.y - cell_y as f32 - 0.5,
            );
            let weights = quadratic_interpolation_weights(cell_diff);

            // check surrounding 9 cells to get volume from density
            let mut density: f32 = 0.0;
            for gx in 0..3 {
                for gy in 0..3 {
                    let weight = weights[gx].x * weights[gy].y;
                    let cell_pos_x = (cell_x as i32 + gx as i32) - 1;
                    let cell_pos_y = (cell_y as i32 + gy as i32) - 1;
                    let cell_at_index = grid.index_at(cell_pos_x as usize, cell_pos_y as usize);
                    density += grid.cells[cell_at_index].mass * weight;
                }
            }

            let volume = mass.0 / density;

            let j: f32 = pp.deformation_gradient.determinant();
            let volume_scaled = volume * j;

            let f_t: Mat2 = pp.deformation_gradient.transpose();
            let f_inv_t = f_t.inverse();
            let f_minus_f_inv_t = pp.deformation_gradient.sub(f_inv_t);

            let p_term_0: Mat2 = f_minus_f_inv_t.mul(pp.elastic_mu);
            let p_term_1: Mat2 = f_inv_t.mul(j.log10() * pp.elastic_lambda);
            let p_combined: Mat2 = p_term_0.add(p_term_1);

            let stress: Mat2 = p_combined.mul_mat2(&f_t).mul(1.0 / j);
            let eq_16_term_0 = stress * (-volume_scaled * 4.0 * world.dt);

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
                    // store the fused force/momentum update from MLS-MPM to apply onto grid later.
                    // todo combine into grid(x,y) = total changes as they come in here...?
                    mmc.0[gx + 3 * gy] = GridMassAndMomentumChange(
                        cell_at_index,
                        0.,
                        eq_16_term_0.mul_scalar(weight).mul_vec2(cell_dist),
                    );
                }
            }
        },
    );
}

pub(super) fn particles_to_grid_fluids(
    pool: Res<ComputeTaskPool>,
    world: Res<WorldState>,
    grid: Res<Grid>,
    mut particles_fluid: Query<
        (
            &Position,
            &Mass,
            &AffineMomentum,
            &ConstitutiveModelFluid,
            &mut CellMassMomentumContributions,
        ),
        With<ParticleTag>,
    >,
) {
    let num_particles = particles_fluid.iter().count();
    if num_particles < 1 {
        return;
    }
    particles_fluid.par_for_each_mut(
        &pool,
        PAR_BATCH_SIZE,
        |(position, mass, affine_momentum, pp, mut mmc)| {
            let cell_x: u32 = position.0.x as u32;
            let cell_y: u32 = position.0.y as u32;
            let cell_diff = Vec2::new(
                position.0.x - cell_x as f32 - 0.5,
                position.0.y - cell_y as f32 - 0.5,
            );
            let weights = quadratic_interpolation_weights(cell_diff);

            // check surrounding 9 cells to get volume from density
            let mut density: f32 = 0.0;
            for gx in 0..3 {
                for gy in 0..3 {
                    let weight = weights[gx].x * weights[gy].y;
                    let cell_pos_x = (cell_x as i32 + gx as i32) - 1;
                    let cell_pos_y = (cell_y as i32 + gy as i32) - 1;
                    let cell_at_index = grid.index_at(cell_pos_x as usize, cell_pos_y as usize);
                    density += grid.cells[cell_at_index].mass * weight;
                }
            }

            let volume = mass.0 / density;

            // fluid constitutive model
            let pressure = f32::max(
                -0.1,
                pp.eos_stiffness * (f32::powf(density / pp.rest_density, pp.eos_power) - 1.0),
            );
            let mut stress = Mat2::from_cols(Vec2::new(-pressure, 0.0), Vec2::new(0.0, -pressure));
            let mut strain = affine_momentum.0.clone();
            let trace = strain.y_axis.x + strain.x_axis.y;
            strain.y_axis.x = trace;
            strain.x_axis.y = trace;
            let viscosity_term = strain * pp.dynamic_viscosity;
            stress += viscosity_term;

            let eq_16_term_0 = stress * (-volume * 4.0 * world.dt);

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
                    let momentum = eq_16_term_0 * weight * cell_dist;
                    mmc.0[gx + 3 * gy] = GridMassAndMomentumChange(cell_at_index, 0., momentum);
                }
            }
        },
    );
}
