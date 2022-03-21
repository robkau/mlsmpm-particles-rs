use std::ops::{Add, Mul, Sub};
use std::sync::Mutex;

use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
};
use bevy::{
    math::{f32::*, Mat2},
    prelude::*,
    tasks::prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use rand::Rng;

const DEFAULT_DT: f32 = 0.0005;
const DEFAULT_GRAVITY: f32 = -1.;
const PAR_BATCH_SIZE: usize = usize::pow(2, 12);

// Tags particle entities
#[derive(Component)]
struct ParticleTag;

// Tags particle spawner entities
#[derive(Component)]
struct ParticleSpawnerTag;

// XY position
#[derive(Component, Debug)]
struct Position(Vec2);

// XY velocity
#[derive(Component, Debug)]
struct Velocity(Vec2);

// mass
#[derive(Component)]
struct Mass(f32);

// 2x2 affine momentum matrix
#[derive(Component)]
struct AffineMomentum(Mat2);

#[derive(Clone, Component)]
struct ConstitutiveModelFluid {
    rest_density: f32,
    dynamic_viscosity: f32,
    eos_stiffness: f32,
    eos_power: f32,
}

#[derive(Clone, Component)]
struct ConstitutiveModelNeoHookeanHyperElastic {
    deformation_gradient: Mat2,
    elastic_lambda: f32, // youngs modulus
    elastic_mu: f32,     // shear modulus
}

// tick the entity was created on
#[derive(Component)]
struct CreatedAt(usize);

// entity deleted after this many ticks
#[derive(Component)]
struct MaxAge(usize);

// todo refactor.
#[derive(Clone)]
enum SpawnerPattern {
    SingleParticle,
    LineHorizontal,
    LineVertical,
    Cube,
    Tower,
    TriangleLeft,
    TriangleRight,
}

#[derive(Clone)]
enum ParticleType {
    Fluid,
    Solid,
}

#[derive(Clone, Component)]
struct ParticleSpawnerInfo {
    created_at: usize,
    pattern: SpawnerPattern,
    spawn_frequency: usize,
    max_particles: usize,
    particle_duration: usize,
    particle_origin: Vec2,
    particle_velocity: Vec2,
    particle_velocity_random_vec_a: Vec2,
    particle_velocity_random_vec_b: Vec2,
    particle_mass: f32,
    particle_fluid_properties: Option<ConstitutiveModelFluid>,
    particle_solid_properties: Option<ConstitutiveModelNeoHookeanHyperElastic>,
}

// MPM grid resource
#[derive(Clone)]
struct Grid {
    cells: Vec<Cell>,
    width: usize,
    dt: f32,
    gravity: f32,
    gravity_enabled: bool,
    current_tick: usize,
}
impl Grid {
    pub fn index_at(&self, x: usize, y: usize) -> usize {
        x * self.width + y
    }
    pub fn reset(&mut self) {
        for mut cell in self.cells.iter_mut() {
            cell.velocity = Vec2::ZERO;
            cell.mass = 0.0;
        }
    }

    pub fn toggle_gravity(&mut self) {
        self.gravity_enabled = !self.gravity_enabled;
    }

    pub fn update(&mut self) {
        self.current_tick += 1;
        for (i, cell) in self.cells.iter_mut().enumerate() {
            if cell.mass > 0.0 {
                // convert momentum to velocity, apply gravity
                cell.velocity *= (1.0 / cell.mass);
                if self.gravity_enabled {
                    cell.velocity.y += self.dt * self.gravity;
                }

                // boundary conditions
                let x = i / self.width;
                let y = i % self.width;
                if x < 2 {
                    // can only stay in place or go right
                    if cell.velocity.x < 0.0 {
                        cell.velocity.x = 0.0;
                    }
                }
                if x > self.width - 3 {
                    // can only stay in place or go left
                    if cell.velocity.x > 0.0 {
                        cell.velocity.x = 0.0;
                    }
                }
                if y < 2 {
                    // can only stay in place or go up
                    if cell.velocity.y < 0.0 {
                        cell.velocity.y = 0.0;
                    }
                }
                if y > self.width - 3 {
                    // can only stay in place or go down
                    if cell.velocity.y > 0.0 {
                        cell.velocity.y = 0.0;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Cell {
    velocity: Vec2,
    mass: f32,
}

fn quadratic_interpolation_weights(cell_diff: Vec2) -> [Vec2; 3] {
    [
        Vec2::new(
            0.5 * f32::powi(0.5 - cell_diff.x, 2),
            0.5 * f32::powi(0.5 - cell_diff.y, 2),
        ),
        Vec2::new(
            0.75 - f32::powi(cell_diff.x, 2),
            0.75 - f32::powi(cell_diff.y, 2),
        ),
        Vec2::new(
            0.5 * f32::powi(0.5 + cell_diff.x, 2),
            0.5 * f32::powi(0.5 + cell_diff.y, 2),
        ),
    ]
}

fn weighted_velocity_and_cell_dist_to_term(weighted_velocity: Vec2, cell_dist: Vec2) -> Mat2 {
    Mat2::from_cols(
        Vec2::new(
            weighted_velocity[0] * cell_dist[0],
            weighted_velocity[1] * cell_dist[0],
        ),
        Vec2::new(
            weighted_velocity[0] * cell_dist[1],
            weighted_velocity[1] * cell_dist[1],
        ),
    )
}

// G2P MPM step
fn grid_to_particles(
    pool: Res<ComputeTaskPool>,
    grid: Res<Grid>,

    mut particles: Query<(&mut Position, &mut Velocity, &mut AffineMomentum), With<ParticleTag>>,
) {
    particles.par_for_each_mut(
        &pool,
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
                    let weighted_velocity = grid.cells[cell_at_index].velocity.mul(weight);
                    b += weighted_velocity_and_cell_dist_to_term(weighted_velocity, cell_dist);
                    velocity.0 += weighted_velocity;
                }
            }

            affine_momentum.0 = b * 4.0;

            // advect particles
            position.0 += velocity.0 * grid.dt;

            // safety clamp to ensure particles don't exit simulation domain
            position.0.x = f32::max(position.0.x, 1.0);
            position.0.x = f32::min(position.0.x, (grid.width - 2) as f32);

            position.0.y = f32::max(position.0.y, 1.0);
            position.0.y = f32::min(position.0.y, (grid.width - 2) as f32);

            // todo this is strange
            // predictive boundary velocity cap
            //let position_next = position.0 + velocity.0;
            //let wall_min: f32 = 3.0;
            //let wall_max: f32 = (grid.width - 1) as f32 - wall_min;
            //if position_next.x < wall_min {
            //    velocity.0.x += wall_min - position_next.x;
            //}
            //if position_next.x > wall_max {
            //    velocity.0.x += wall_max - position_next.x;
            //}
            //if position_next.y < wall_min {
            //    velocity.0.y += wall_min - position_next.y;
            //}
            //if position_next.y > wall_max {
            //    velocity.0.y += wall_max - position_next.y;
            //}
        },
    );
}

fn update_deformation_gradients(
    pool: Res<ComputeTaskPool>,
    grid: Res<Grid>,
    mut particles_solid: Query<
        (
            &mut Position,
            &mut Velocity,
            &mut AffineMomentum,
            &mut ConstitutiveModelNeoHookeanHyperElastic,
        ),
        With<ParticleTag>,
    >,
) {
    particles_solid.par_for_each_mut(
        &pool,
        PAR_BATCH_SIZE,
        |(mut position, mut velocity, mut affine_momentum, mut pp)| {
            let deformation_new: Mat2 = Mat2::IDENTITY
                .add(affine_momentum.0.mul(grid.dt))
                .mul_mat2(&pp.deformation_gradient);
            pp.deformation_gradient = deformation_new;
        },
    );
}

fn delete_old_entities(
    mut commands: Commands,
    grid: Res<Grid>,
    aged_entities: Query<(Entity, &CreatedAt, &MaxAge)>,
) {
    aged_entities.for_each(|(id, created_at, max_age)| {
        if grid.current_tick > created_at.0 + max_age.0 {
            commands.entity(id).despawn();
        }
    });
}

fn update_sprites(
    pool: Res<ComputeTaskPool>,
    mut particles: Query<(&mut Transform, &Position), With<ParticleTag>>,
) {
    // todo adjust size relative to mass, min/max size determined by grid+window sizes
    // todo color based on velocity. (maybe acceleration?)
    // todo color based on constitutive model. (or initial texture.)
    particles.par_for_each_mut(&pool, PAR_BATCH_SIZE, |(mut transform, position)| {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;
    });
}

fn particles_to_grid(
    pool: Res<ComputeTaskPool>,
    mut grid: ResMut<Grid>,
    particles_fluid: Query<
        (&Position, &Mass, &AffineMomentum, &ConstitutiveModelFluid),
        With<ParticleTag>,
    >,
    particles_solid: Query<
        (
            &Position,
            &Mass,
            &AffineMomentum,
            &ConstitutiveModelNeoHookeanHyperElastic,
        ),
        With<ParticleTag>,
    >,
) {
    let momentum_changes = Mutex::new(vec![Vec2::ZERO; grid.width * grid.width]);
    particles_fluid.par_for_each(
        &pool,
        PAR_BATCH_SIZE,
        |(position, mass, affine_momentum, pp)| {
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

            let eq_16_term_0 = stress * (-volume * 4.0 * grid.dt);

            // for all surrounding 9 cells
            let mut changes: [(usize, Vec2); 9] = Default::default();
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
                    let changes_cell = gx + 3 * gy;
                    let momentum = eq_16_term_0 * weight * cell_dist;
                    changes[changes_cell].0 = cell_at_index;
                    changes[changes_cell].1 = momentum;
                }
            }

            let mut m = momentum_changes.lock().unwrap();
            for change in changes.iter() {
                m[change.0] += change.1;
            }
        },
    );

    particles_solid.par_for_each(
        &pool,
        PAR_BATCH_SIZE,
        |(position, mass, affine_momentum, pp)| {
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
            let eq_16_term_0 = stress * (-volume_scaled * 4.0 * grid.dt);

            // for all surrounding 9 cells
            let mut changes: [(usize, Vec2); 9] = Default::default();
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
                    let changes_cell = gx + 3 * gy;

                    // fused force/momentum update from MLS-MPM
                    changes[changes_cell].0 = cell_at_index;
                    changes[changes_cell].1 = eq_16_term_0.mul_scalar(weight).mul_vec2(cell_dist);
                }
            }

            let mut m = momentum_changes.lock().unwrap();
            for change in changes.iter() {
                m[change.0] += change.1;
            }
        },
    );

    // apply calculated momentum changes
    for (i, change) in momentum_changes.lock().unwrap().iter().enumerate() {
        grid.cells[i].velocity += *change;
    }
}

fn reset_grid(mut grid: ResMut<Grid>) {
    grid.reset();
}

fn tick_spawners(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    grid: Res<Grid>,
    particles: Query<(), With<ParticleTag>>,
    spawners: Query<(&ParticleSpawnerInfo), With<ParticleSpawnerTag>>,
) {
    // todo recreate spiral spawn pattern - rate per spawn and rotation per spawn
    let solid_tex = asset_server.load("solid_particle.png");
    let liquid_tex = asset_server.load("liquid_particle.png");

    let mut rng = rand::thread_rng();
    spawners.for_each(|(spawner_info)| {
        if (grid.current_tick - spawner_info.created_at) % spawner_info.spawn_frequency == 0 {
            if particles.iter().count() < spawner_info.max_particles {
                let base_vel = spawner_info.particle_velocity;
                let random_a_contrib = Vec2::new(
                    rng.gen::<f32>() * spawner_info.particle_velocity_random_vec_a.x,
                    rng.gen::<f32>() * spawner_info.particle_velocity_random_vec_a.y,
                );
                let random_b_contrib = Vec2::new(
                    rng.gen::<f32>() * spawner_info.particle_velocity_random_vec_b.x,
                    rng.gen::<f32>() * spawner_info.particle_velocity_random_vec_b.y,
                );
                let spawn_vel = base_vel + random_a_contrib + random_b_contrib;

                if spawner_info.particle_fluid_properties.is_none()
                    && spawner_info.particle_solid_properties.is_none()
                {
                    // incorrectly configured spawner. no particle properties.
                    return;
                }
                // todo refactor.
                let mut spawn_type = ParticleType::Fluid;
                if !spawner_info.particle_solid_properties.is_none() {
                    spawn_type = ParticleType::Solid;
                }

                match spawner_info.pattern {
                    SpawnerPattern::SingleParticle => {
                        if let ParticleType::Fluid = spawn_type {
                            new_fluid_particle(
                                &mut commands,
                                liquid_tex.clone(),
                                grid.current_tick,
                                spawner_info.particle_origin,
                                Some(base_vel),
                                Some(spawner_info.particle_mass),
                                spawner_info.particle_fluid_properties.clone(),
                                Some(spawner_info.particle_duration),
                            );
                        } else {
                            new_solid_particle(
                                &mut commands,
                                solid_tex.clone(),
                                grid.current_tick,
                                spawner_info.particle_origin,
                                Some(base_vel),
                                Some(spawner_info.particle_mass),
                                spawner_info.particle_solid_properties.clone(),
                                Some(spawner_info.particle_duration),
                            );
                        }
                    }
                    SpawnerPattern::LineHorizontal => {
                        for x in 0..100 {
                            if let ParticleType::Fluid = spawn_type {
                                new_fluid_particle(
                                    &mut commands,
                                    liquid_tex.clone(),
                                    grid.current_tick,
                                    spawner_info.particle_origin + Vec2::new(x as f32, 0.),
                                    Some(base_vel),
                                    Some(spawner_info.particle_mass),
                                    spawner_info.particle_fluid_properties.clone(),
                                    Some(spawner_info.particle_duration),
                                );
                            } else {
                                new_solid_particle(
                                    &mut commands,
                                    solid_tex.clone(),
                                    grid.current_tick,
                                    spawner_info.particle_origin + Vec2::new(x as f32, 0.),
                                    Some(base_vel),
                                    Some(spawner_info.particle_mass),
                                    spawner_info.particle_solid_properties.clone(),
                                    Some(spawner_info.particle_duration),
                                );
                            }
                        }
                    }
                    SpawnerPattern::LineVertical => {
                        for y in 0..15 {
                            if let ParticleType::Fluid = spawn_type {
                                new_fluid_particle(
                                    &mut commands,
                                    liquid_tex.clone(),
                                    grid.current_tick,
                                    spawner_info.particle_origin + Vec2::new(0. as f32, y as f32),
                                    Some(base_vel),
                                    Some(spawner_info.particle_mass),
                                    spawner_info.particle_fluid_properties.clone(),
                                    Some(spawner_info.particle_duration),
                                );
                            } else {
                                new_solid_particle(
                                    &mut commands,
                                    solid_tex.clone(),
                                    grid.current_tick,
                                    spawner_info.particle_origin + Vec2::new(0., y as f32),
                                    Some(base_vel),
                                    Some(spawner_info.particle_mass),
                                    spawner_info.particle_solid_properties.clone(),
                                    Some(spawner_info.particle_duration),
                                );
                            }
                        }
                    }
                    SpawnerPattern::Cube => {
                        for x in 0..15 {
                            for y in 0..15 {
                                if let ParticleType::Fluid = spawn_type {
                                    new_fluid_particle(
                                        &mut commands,
                                        liquid_tex.clone(),
                                        grid.current_tick,
                                        spawner_info.particle_origin
                                            + Vec2::new((x as f32 / 2.), (y as f32 / 2.)),
                                        Some(spawn_vel),
                                        Some(spawner_info.particle_mass),
                                        spawner_info.particle_fluid_properties.clone(),
                                        Some(spawner_info.particle_duration),
                                    );
                                } else {
                                    new_solid_particle(
                                        &mut commands,
                                        solid_tex.clone(),
                                        grid.current_tick,
                                        spawner_info.particle_origin
                                            + Vec2::new((x as f32 / 2.), (y as f32 / 2.)),
                                        Some(spawn_vel),
                                        Some(spawner_info.particle_mass),
                                        spawner_info.particle_solid_properties.clone(),
                                        Some(spawner_info.particle_duration),
                                    );
                                }
                            }
                        }
                    }
                    SpawnerPattern::Tower => {
                        for x in 0..120 {
                            for y in 0..400 {
                                if let ParticleType::Fluid = spawn_type {
                                    new_fluid_particle(
                                        &mut commands,
                                        liquid_tex.clone(),
                                        grid.current_tick,
                                        spawner_info.particle_origin
                                            + Vec2::new(x as f32 / 4., y as f32 / 4.),
                                        Some(spawn_vel),
                                        Some(spawner_info.particle_mass),
                                        spawner_info.particle_fluid_properties.clone(),
                                        Some(spawner_info.particle_duration),
                                    );
                                } else {
                                    new_solid_particle(
                                        &mut commands,
                                        solid_tex.clone(),
                                        grid.current_tick,
                                        spawner_info.particle_origin
                                            + Vec2::new(x as f32 / 4., y as f32 / 4.),
                                        Some(spawn_vel),
                                        Some(spawner_info.particle_mass),
                                        spawner_info.particle_solid_properties.clone(),
                                        Some(spawner_info.particle_duration),
                                    );
                                }
                            }
                        }
                    }
                    SpawnerPattern::TriangleLeft => {
                        for x in 0..15 {
                            for y in 0..x {
                                // offset y by 0.5 every other time
                                let mut ya: f32;
                                if x % 2 == 0 {
                                    ya = y as f32 - 0.25;
                                } else {
                                    ya = y as f32 + 0.25;
                                }
                                ya -= x as f32 / 2.;

                                if let ParticleType::Fluid = spawn_type {
                                    new_fluid_particle(
                                        &mut commands,
                                        liquid_tex.clone(),
                                        grid.current_tick,
                                        spawner_info.particle_origin
                                            + Vec2::new(x as f32, ya as f32),
                                        Some(spawn_vel),
                                        Some(spawner_info.particle_mass),
                                        spawner_info.particle_fluid_properties.clone(),
                                        Some(spawner_info.particle_duration),
                                    );
                                } else {
                                    new_solid_particle(
                                        &mut commands,
                                        solid_tex.clone(),
                                        grid.current_tick,
                                        spawner_info.particle_origin
                                            + Vec2::new(x as f32, ya as f32),
                                        Some(spawn_vel),
                                        Some(spawner_info.particle_mass),
                                        spawner_info.particle_solid_properties.clone(),
                                        Some(spawner_info.particle_duration),
                                    );
                                }
                            }
                        }
                    }
                    SpawnerPattern::TriangleRight => {
                        for x in 0..15 {
                            for y in 0..x {
                                // offset y by 0.5 every other time
                                let mut ya: f32;
                                if (x) % 2 == 0 {
                                    ya = y as f32 - 0.25;
                                } else {
                                    ya = y as f32 + 0.25;
                                }
                                ya -= x as f32 / 2.;

                                if let ParticleType::Fluid = spawn_type {
                                    new_fluid_particle(
                                        &mut commands,
                                        liquid_tex.clone(),
                                        grid.current_tick,
                                        spawner_info.particle_origin
                                            + Vec2::new((15 - x) as f32, ya as f32),
                                        Some(spawn_vel),
                                        Some(spawner_info.particle_mass),
                                        spawner_info.particle_fluid_properties.clone(),
                                        Some(spawner_info.particle_duration),
                                    );
                                } else {
                                    new_solid_particle(
                                        &mut commands,
                                        solid_tex.clone(),
                                        grid.current_tick,
                                        spawner_info.particle_origin
                                            + Vec2::new((15 - x) as f32, ya as f32),
                                        Some(spawn_vel),
                                        Some(spawner_info.particle_mass),
                                        spawner_info.particle_solid_properties.clone(),
                                        Some(spawner_info.particle_duration),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

fn apply_cursor_effects(
    pool: Res<ComputeTaskPool>,
    clicks: Res<Input<MouseButton>>, // has mouse clicks
    windows: Res<Windows>,           // has cursor position
    grid: Res<Grid>,
    mut particles: Query<(&Position, &mut Velocity, &mut Mass), With<ParticleTag>>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(win_pos) = window.cursor_position() {
        // cursor is inside the window.
        // translate window position to grid position
        let scale = window.width() / grid.width as f32;
        let grid_pos = win_pos / scale;
        // if particle is near cursor, push it away.
        particles.par_for_each_mut(
            &pool,
            PAR_BATCH_SIZE,
            |(position, mut velocity, mut mass)| {
                let dist = Vec2::new((position.0.x - grid_pos.x), (position.0.y - grid_pos.y));

                let mouse_radius = 6.;

                if dist.dot(dist) < mouse_radius * mouse_radius {
                    let norm_factor = dist.length() / mouse_radius;
                    let force = dist.normalize() * (norm_factor / 2.);
                    velocity.0 += force;
                }
            },
        );
    }
}

fn new_solid_particle(
    commands: &mut Commands,
    tex: Handle<Image>,
    tick: usize,
    at: Vec2,
    vel: Option<Vec2>,
    mass: Option<f32>,
    pp: Option<ConstitutiveModelNeoHookeanHyperElastic>,
    max_age: Option<usize>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: tex.clone(),
            transform: Transform::from_scale(Vec3::splat(0.002)), // todo scale me from mass.
            ..Default::default()
        })
        .insert_bundle((
            Position(at),
            Velocity(vel.unwrap_or(Vec2::ZERO)),
            Mass(mass.unwrap_or(3.)),
            AffineMomentum(Mat2::ZERO),
            pp.unwrap_or(ConstitutiveModelNeoHookeanHyperElastic {
                deformation_gradient: Mat2::IDENTITY,
                elastic_lambda: 1.,
                elastic_mu: 8000.,
            }),
            MaxAge(max_age.unwrap_or(5000)),
            CreatedAt(tick),
            ParticleTag,
        ));
}

fn new_fluid_particle(
    commands: &mut Commands,
    tex: Handle<Image>,
    tick: usize,
    at: Vec2,
    vel: Option<Vec2>,
    mass: Option<f32>,
    pp: Option<ConstitutiveModelFluid>,
    max_age: Option<usize>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: tex.clone(),
            transform: Transform::from_scale(Vec3::splat(0.001)), // todo scale me from mass.
            ..Default::default()
        })
        .insert_bundle((
            Position(at),
            Velocity(vel.unwrap_or(Vec2::ZERO)),
            Mass(mass.unwrap_or(1.)),
            AffineMomentum(Mat2::ZERO),
            pp.unwrap_or(ConstitutiveModelFluid {
                rest_density: 4.,
                dynamic_viscosity: 0.1,
                eos_stiffness: 100.,
                eos_power: 4.,
            }),
            MaxAge(max_age.unwrap_or(5000)),
            CreatedAt(tick),
            ParticleTag,
        ));
}

// todo 1: update each particle list in order
// todo 2: one system for each constitutive model

fn update_grid(mut grid: ResMut<Grid>) {
    grid.update();
}

// todo this might only want to target fluid particles since solids do this change inside p2g. or combine them both here.
fn update_cells(
    pool: Res<ComputeTaskPool>,
    mut grid: ResMut<Grid>,
    particles: Query<(&Position, &Velocity, &Mass, &AffineMomentum), With<ParticleTag>>,
) {
    let mass_contrib_changes = Mutex::new(vec![(0.0, Vec2::ZERO); grid.width * grid.width]);

    particles.par_for_each(
        &pool,
        PAR_BATCH_SIZE,
        |(position, velocity, mass, affine_momentum)| {
            let cell_x: u32 = position.0.x as u32;
            let cell_y: u32 = position.0.y as u32;
            let cell_diff = Vec2::new(
                position.0.x - cell_x as f32 - 0.5,
                position.0.y - cell_y as f32 - 0.5,
            );
            let weights = quadratic_interpolation_weights(cell_diff);

            // for all surrounding 9 cells

            //collect momentum changes for surrounding 9 cells.
            let mut changes: [(usize, f32, Vec2); 9] = Default::default();
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
                    // todo try crossbeam channel instead of mutex.
                    let changes_cell = gx + 3 * gy;
                    changes[changes_cell].0 = cell_at_index;
                    changes[changes_cell].1 = mass_contrib;
                    changes[changes_cell].2 = (velocity.0 + q) * mass_contrib;
                }
            }
            let mut mc = mass_contrib_changes.lock().unwrap();
            for change in changes.iter() {
                mc[change.0].0 += change.1;
                mc[change.0].1 += change.2;
            }
        },
    );

    for (i, changes) in mass_contrib_changes.lock().unwrap().iter().enumerate() {
        grid.cells[i].mass += (*changes).0;
        grid.cells[i].velocity += (*changes).1;
    }
}

fn create_initial_spawners(mut commands: Commands, grid: Res<Grid>) {
    // shoot arrows to the right
    // young's modulus and shear modulus of steel.
    // 180 Gpa young's
    // 78Gpa shear
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::TriangleRight,
            spawn_frequency: 1000,
            max_particles: 200000,
            particle_duration: 40000,
            particle_origin: Vec2::new(1. * grid.width as f32 / 4., 2. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(100.3, -1.3),
            particle_velocity_random_vec_a: Vec2::new(-0.0, -0.0),
            particle_velocity_random_vec_b: Vec2::new(0.0, 0.0),
            particle_mass: 2.,
            particle_fluid_properties: None,
            particle_solid_properties: Some(ConstitutiveModelNeoHookeanHyperElastic {
                deformation_gradient: Default::default(),
                elastic_lambda: 10. * 180. * 1000.,
                elastic_mu: 10. * 78. * 1000.,
            }),
        },
        ParticleSpawnerTag,
    ));

    // spawn tower on first turn.
    // young's modulus and shear modulus of wood/plywood
    //9Gpa young's modulus
    //0.6Gpa shear modulus
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Tower,
            spawn_frequency: 999999999999,
            max_particles: 50000,
            particle_duration: 500000,
            particle_origin: Vec2::new(3. * grid.width as f32 / 4., 1.),
            particle_velocity: Vec2::ZERO,
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: 1.,
            particle_fluid_properties: None,
            particle_solid_properties: Some(ConstitutiveModelNeoHookeanHyperElastic {
                deformation_gradient: Default::default(),
                elastic_lambda: 9. * 1000.,
                elastic_mu: 0.6 * 1000.,
            }),
        },
        ParticleSpawnerTag,
    ));

    // make it rain!
    //commands.spawn_bundle((
    //    ParticleSpawnerInfo {
    //        created_at: 0,
    //        pattern: SpawnerPattern::Tower,
    //        spawn_frequency: 500000000,
    //        max_particles: 50000,
    //        particle_duration: 100000,
    //        particle_origin: Vec2::new(5., 1.),
    //        particle_velocity: Vec2::new(10., 0.),
    //        particle_velocity_random_vec_a: Vec2::new(-3., 10.),
    //        particle_velocity_random_vec_b: Vec2::new(3., -10.),
    //        particle_mass: 1.,
    //        particle_fluid_properties: Some(ConstitutiveModelFluid {
    //            rest_density: 4.,
    //            dynamic_viscosity: 0.1,
    //            eos_stiffness: 100.,
    //            eos_power: 4.,
    //        }),
    //        particle_solid_properties: None,
    //    },
    //    ParticleSpawnerTag,
    //));
}

fn setup_camera(mut commands: Commands, grid: Res<Grid>, wnds: Res<Windows>) {
    let mut cb = OrthographicCameraBundle::new_2d();

    let wnd = wnds.get_primary().unwrap();
    let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
    let scale = wnd.width() / grid.width as f32;

    cb.transform = Transform::from_translation(Vec3::new(
        size.x / (scale * 2.0),
        size.y / (scale * 2.0),
        0.0,
    ));
    cb.orthographic_projection.scale = 1.0 / scale;
    commands.spawn_bundle(cb);
}

fn handle_inputs(
    mut commands: Commands,
    windows: Res<Windows>,
    keys: Res<Input<KeyCode>>,
    mut egui_context: ResMut<EguiContext>,
    mut egui_settings: ResMut<EguiSettings>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut grid: ResMut<Grid>,
    particles: Query<(Entity), With<ParticleTag>>,
) {
    // todo place spawners and drag direction and click to configure

    egui::Window::new("Controls").show(egui_context.ctx_mut(), |ui| {
        if ui.button("(R)eset").clicked() || keys.just_pressed(KeyCode::R) {
            particles.for_each(|(id)| {
                commands.entity(id).despawn();
            });
            grid.dt = DEFAULT_DT;
            return;
        };
        if ui.button("(G)ravity toggle").clicked() || keys.just_pressed(KeyCode::G) {
            grid.toggle_gravity();
            return;
        };

        // slider for gravity
        ui.add(egui::Slider::new(&mut grid.gravity, -10.0..=10.).text("gravity"));

        // slider for DT.
        ui.add(egui::Slider::new(&mut grid.dt, 0.0001..=0.01).text("dt"));

        // toggle hiDPI with '/'
        if keys.just_pressed(KeyCode::Slash) || toggle_scale_factor.is_none() {
            *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(true));

            if let Some(window) = windows.get_primary() {
                let scale_factor = if toggle_scale_factor.unwrap() {
                    1.0 / window.scale_factor()
                } else {
                    1.0
                };
                egui_settings.scale_factor = scale_factor;
            }
        }

        // todo:
        // one spawner can be selected (or new spawner to-create can be selected)
        // click and drag when placing to set particle velocity
        // the selected spawner shows its elements on left
    });
}

fn main() {
    let grid_width = usize::pow(2, 7);
    let grid_zoom = 6.0;
    let window_width = grid_width as f32 * grid_zoom;

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "mlsmpm-particles-rs".to_string(),
            width: window_width,
            height: window_width,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0
                };
                grid_width * grid_width
            ],
            width: grid_width,
            dt: DEFAULT_DT,
            gravity: DEFAULT_GRAVITY,
            gravity_enabled: true,
            current_tick: 0,
        }) // add global MPM grid
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy_framepace::FramepacePlugin {
            enabled: true,
            framerate_limit: bevy_framepace::FramerateLimit::Auto,
            warn_on_frame_drop: false,
            safety_margin: std::time::Duration::from_micros(100),
            power_saver: bevy_framepace::PowerSaver::Disabled,
        })
        .add_startup_system(setup_camera)
        .add_startup_system(create_initial_spawners)
        .add_system(handle_inputs.label("handle_inputs").before("tick_spawners"))
        .add_system(
            tick_spawners
                .label("tick_spawners")
                .before("apply_cursor_effects"),
        )
        .add_system(
            apply_cursor_effects
                .label("apply_cursor_effects")
                .before("reset_grid"),
        )
        .add_system(reset_grid.label("reset_grid").before("update_cells"))
        .add_system(update_cells.label("update_cells").before("p2g"))
        .add_system(particles_to_grid.label("p2g").before("update_grid"))
        .add_system(update_grid.label("update_grid").before("g2p"))
        .add_system(
            grid_to_particles
                .label("g2p")
                .before("update_deformation_gradients"),
        )
        .add_system(
            update_deformation_gradients
                .label("update_deformation_gradients")
                .before("delete_old_entities"),
        )
        .add_system(
            delete_old_entities
                .label("delete_old_entities")
                .before("update_sprites"),
        )
        .add_system(update_sprites.label("update_sprites"))
        .run();
}

mod test;

// todo render to (animated) image output
// https://github.com/bevyengine/bevy/issues/1207
//https://github.com/rmsc/bevy/blob/render_to_file/examples/3d/render_to_file.rs
