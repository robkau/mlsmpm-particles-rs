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

const BOUNDARY_FRICTION_DAMPING: f32 = 0.001;
const DEFAULT_DT: f32 = 0.01;
const DEFAULT_GRAVITY: f32 = -3.3;
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

#[derive(Component)]
struct ConstitutiveModelFluid {
    rest_density: f32,
    dynamic_viscosity: f32,
    eos_stiffness: f32,
    eos_power: f32,
}

#[derive(Component)]
struct ConstitutiveModelNeoHookeanHyperElastic {
    deformation_gradient: Mat2,
    elastic_lambda: f32,
    elastic_mu: f32,
}

// tick the entity was created on
#[derive(Component)]
struct CreatedAt(usize);

// entity deleted after this many ticks
#[derive(Component)]
struct MaxAge(usize);

#[derive(Component)]
struct ParticleSpawnerInfo {
    created_at: usize,
    spawn_frequency: usize,
    max_particles: usize,
    particle_origin: Vec2,
    particle_velocity: Vec2,
    particle_velocity_random_vec_a: Vec2,
    particle_velocity_random_vec_b: Vec2,
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
                    cell.velocity.y *= 1.0 - BOUNDARY_FRICTION_DAMPING;
                }
                if x > self.width - 3 {
                    // can only stay in place or go left
                    if cell.velocity.x > 0.0 {
                        cell.velocity.x = 0.0;
                    }
                    cell.velocity.y *= 1.0 - BOUNDARY_FRICTION_DAMPING;
                }
                if y < 2 {
                    // can only stay in place or go up
                    if cell.velocity.y < 0.0 {
                        cell.velocity.y = 0.0;
                    }
                    cell.velocity.x *= 1.0 - BOUNDARY_FRICTION_DAMPING;
                }
                if y > self.width - 3 {
                    // can only stay in place or go down
                    if cell.velocity.y > 0.0 {
                        cell.velocity.y = 0.0;
                    }
                    cell.velocity.x *= 1.0 - BOUNDARY_FRICTION_DAMPING;
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

            // todo this is applying too early?
            // boundaries
            let position_next = position.0 + velocity.0;
            let wall_min: f32 = 3.0;
            let wall_max: f32 = (grid.width - 4) as f32;
            //if position_next.x < wall_min {
            //    velocity.0.x += wall_min - position_next.x;
            //    velocity.0.y *= 1.0 - BOUNDARY_FRICTION_DAMPING;
            //}
            //if position_next.x > wall_max {
            //    velocity.0.x += wall_max - position_next.x;
            //    velocity.0.y *= 1.0 - BOUNDARY_FRICTION_DAMPING;
            //}
            //if position_next.y < wall_min {
            //    velocity.0.y += wall_min - position_next.y;
            //    velocity.0.x *= 1.0 - BOUNDARY_FRICTION_DAMPING;
            //}
            //if position_next.y > wall_max {
            //    velocity.0.y += wall_max - position_next.y;
            //    velocity.0.x *= 1.0 - BOUNDARY_FRICTION_DAMPING;
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

fn collide_with_solid_cells(
    mut commands: Commands,
    pool: Res<ComputeTaskPool>,
    grid: Res<Grid>,
    particles: Query<(Entity, &Position, &Velocity), With<ParticleTag>>,
) {
    let mut particles_to_collide: Mutex<Vec<Entity>> = Mutex::new(Vec::new());

    particles.par_for_each(&pool, PAR_BATCH_SIZE, |(id, position, velocity)| {
        // boundaries
        let position_next = position.0 + velocity.0;
        if position_next.x < 90. && position_next.y < 50. {
            particles_to_collide.lock().unwrap().push(id);
        }
    });

    // apply the particles that collided
    for (i, particle_id) in particles_to_collide.lock().unwrap().iter().enumerate() {
        // todo i am useless.
        //commands.entity(*particle_id).despawn();
    }
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
    let momentum_changes = Mutex::new(vec![(Vec2::ZERO); grid.width * grid.width]);
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
                    let mut m = momentum_changes.lock().unwrap();
                    m[cell_at_index as usize] += momentum;
                }
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
            // todo base 2 or 10?
            let p_term_1: Mat2 = f_inv_t.mul(j.log(2.) * pp.elastic_lambda);
            let p_combined: Mat2 = p_term_0.add(p_term_1);

            let stress: Mat2 = p_combined.mul_mat2(&f_t).mul(1.0 / j);
            let eq_16_term_0 = stress * (-volume_scaled * 4.0 * grid.dt);

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

                    // fused force/momentum update from MLS-MPM
                    let mut m = momentum_changes.lock().unwrap();
                    m[cell_at_index as usize] +=
                        eq_16_term_0.mul_scalar(weight).mul_vec2(cell_dist);
                }
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
    // todo support spawn patterns - like spiral with arc per tick
    let solid_tex = asset_server.load("solid_particle.png");
    let liquid_tex = asset_server.load("liquid_particle.png");

    // todo move into own system if it works.
    if grid.current_tick % 1000 == 0 {
        spawn_square(
            &mut commands,
            solid_tex.clone(),
            liquid_tex.clone(),
            grid.current_tick,
            Some(5000),
            Vec2::new(50., 50.),
            false,
        );
    }

    let mut rng = rand::thread_rng();
    spawners.for_each(|(state)| {
        if (grid.current_tick - state.created_at) % state.spawn_frequency == 0 {
            if particles.iter().count() < state.max_particles {
                let base_vel = state.particle_velocity;
                let random_a_contrib = Vec2::new(
                    rng.gen::<f32>() * state.particle_velocity_random_vec_a.x,
                    rng.gen::<f32>() * state.particle_velocity_random_vec_a.y,
                );
                let random_b_contrib = Vec2::new(
                    rng.gen::<f32>() * state.particle_velocity_random_vec_b.x,
                    rng.gen::<f32>() * state.particle_velocity_random_vec_b.y,
                );

                //new_fluid_particle(
                //    &mut commands,
                //    tex.clone(),
                //    grid.current_tick,
                //    state.particle_origin,
                //    Some(base_vel + random_a_contrib + random_b_contrib),
                //    None,
                //    None,
                //    None,
                //);

                spawn_square(
                    &mut commands,
                    solid_tex.clone(),
                    liquid_tex.clone(),
                    grid.current_tick,
                    Some(1500),
                    state.particle_origin,
                    true,
                );
            }
        }
    });
}

fn make_solid_on_click(
    pool: Res<ComputeTaskPool>,
    buttons: Res<Input<MouseButton>>, // has mouse clicks
    windows: Res<Windows>,            // has cursor position
    grid: Res<Grid>,
    mut particles: Query<(&Position, &mut Velocity, &mut Mass), With<ParticleTag>>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(win_pos) = window.cursor_position() {
        // cursor is inside the window.
        // translate window position to grid position
        let scale = window.width() / grid.width as f32;
        let grid_pos = win_pos / scale;
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

fn spawn_square(
    commands: &mut Commands,
    solid_tex: Handle<Image>,
    liquid_tex: Handle<Image>,
    tick: usize,
    max_age: Option<usize>,
    origin: Vec2,
    fluid: bool,
) {
    let mut rng = rand::thread_rng();
    let square_vel = Vec2::new(rng.gen::<f32>() * 10.0 - 5., rng.gen::<f32>() * 10.0 - 5.);
    for i in 0..25 {
        for j in 0..25 {
            if fluid {
                new_fluid_particle(
                    commands,
                    liquid_tex.clone(),
                    tick,
                    origin + Vec2::new(i as f32, j as f32),
                    Some(square_vel),
                    None,
                    None,
                    max_age,
                );
            } else {
                new_solid_particle(
                    commands,
                    solid_tex.clone(),
                    tick,
                    origin + Vec2::new(i as f32, j as f32),
                    Some(square_vel),
                    None,
                    None,
                    max_age,
                );
            }
        }
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
            transform: Transform::from_scale(Vec3::splat(0.004)), // todo scale me from mass.
            ..Default::default()
        })
        .insert_bundle((
            Position(at),
            Velocity(vel.unwrap_or(Vec2::ZERO)),
            Mass(mass.unwrap_or(3.)),
            AffineMomentum(Mat2::ZERO),
            pp.unwrap_or(ConstitutiveModelNeoHookeanHyperElastic {
                deformation_gradient: Mat2::IDENTITY,
                elastic_lambda: 1000.,
                elastic_mu: 2000.,
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
            transform: Transform::from_scale(Vec3::splat(0.002)), // todo scale me from mass.
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
                eos_stiffness: 10.,
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
                    let mut mc = mass_contrib_changes.lock().unwrap();
                    mc[cell_at_index].0 += mass_contrib;
                    mc[cell_at_index].1 += (velocity.0 + q) * mass_contrib;
                }
            }
        },
    );

    for (i, changes) in mass_contrib_changes.lock().unwrap().iter().enumerate() {
        grid.cells[i].mass += (*changes).0;
        grid.cells[i].velocity += (*changes).1;
    }
}

fn create_initial_spawners(mut commands: Commands, grid: Res<Grid>) {
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            spawn_frequency: 300,
            max_particles: 20000,
            particle_origin: Vec2::new(1. * grid.width as f32 / 4., 3. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(-9.3, -1.3),
            particle_velocity_random_vec_a: Vec2::new(-0.01, -0.01),
            particle_velocity_random_vec_b: Vec2::new(0.01, 0.01),
        },
        ParticleSpawnerTag,
    ));

    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            spawn_frequency: 63,
            max_particles: 10000,
            particle_origin: Vec2::new(1.5 * grid.width as f32 / 4., 3. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(-9.3, -1.3),
            particle_velocity_random_vec_a: Vec2::new(-0.01, -0.01),
            particle_velocity_random_vec_b: Vec2::new(0.01, 0.01),
        },
        ParticleSpawnerTag,
    ));

    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            spawn_frequency: 65,
            max_particles: 15000,
            particle_origin: Vec2::new(2. * grid.width as f32 / 4., 3. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(-9.3, -1.3),
            particle_velocity_random_vec_a: Vec2::new(-0.01, -0.01),
            particle_velocity_random_vec_b: Vec2::new(0.01, 0.01),
        },
        ParticleSpawnerTag,
    ));

    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            spawn_frequency: 66,
            max_particles: 15000,
            particle_origin: Vec2::new(2.5 * grid.width as f32 / 4., 3. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(-9.3, -1.3),
            particle_velocity_random_vec_a: Vec2::new(-0.01, -0.01),
            particle_velocity_random_vec_b: Vec2::new(0.01, 0.01),
        },
        ParticleSpawnerTag,
    ));

    // todo dissipation way before boundary??

    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            spawn_frequency: 68,
            max_particles: 15000,
            particle_origin: Vec2::new(3. * grid.width as f32 / 4., 3. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(-9.3, -1.3),
            particle_velocity_random_vec_a: Vec2::new(-0.01, -0.01),
            particle_velocity_random_vec_b: Vec2::new(0.01, 0.01),
        },
        ParticleSpawnerTag,
    ));
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
    mut currently_selected_spawner_id: Local<Option<usize>>,
    mut grid: ResMut<Grid>,
    particles: Query<(Entity), With<ParticleTag>>,
    particle_spawners: Query<(Entity, &mut ParticleSpawnerInfo), With<ParticleSpawnerTag>>,
) {
    // todo configure particle age
    // todo place spawners and drag direction

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
        // todo enabled/disabled based on gravity toggle.
        ui.add(egui::Slider::new(&mut grid.gravity, -10.0..=10.).text("gravity"));

        // slider for DT.
        ui.add(egui::Slider::new(&mut grid.dt, 0.0001..=0.05).text("dt"));

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

        // todo i need a slider for particle despawn time!
        // todo i need a slider for all particle constitutive models!
        // todo i should update relevant spawners with new properties
    });
}

fn main() {
    let grid_width = usize::pow(2, 7);
    let grid_zoom = 8.0;
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
                .before("make_solid_on_click"),
        )
        .add_system(
            make_solid_on_click
                .label("make_solid_on_click")
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
                .before("collide_with_solid_cells"),
        )
        .add_system(
            collide_with_solid_cells
                .label("collide_with_solid_cells")
                .before("update_sprites"),
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
