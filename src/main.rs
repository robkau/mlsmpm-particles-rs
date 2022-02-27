use std::ops::Mul;
use std::sync::Mutex;

use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
};
use bevy::{
    math::{f32::*, Mat2},
    prelude::*,
    tasks::prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use rand::Rng;

const BOUNDARY_FRICTION_DAMPING: f32 = 0.001;
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

// particle constitutive model
#[derive(Component)]
struct RestDensity(f32);

// particle constitutive model
#[derive(Component)]
struct DynamicViscosity(f32);

// particle constitutive model
#[derive(Component)]
struct EosStiffness(f32);

// particle constitutive model
#[derive(Component)]
struct EosPower(f32);

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

            // cursor effects
            // 		dist := mgl64.Vec2{
            // 			p.p[0] - cx, // x distance
            // 			p.p[1] - cy, // y distance
            // 		}
            // 		if dist.Dot(dist) < mouseRadius*mouseRadius {
            // 			normFactor := dist.Len() / mouseRadius
            // 			normFactor = math.Pow(math.Sqrt(normFactor), 8)
            // 			force := dist.Normalize().Mul(normFactor / 2)
            // 			p.v = p.v.Add(force)
            // 		}
            //

            // boundaries
            let position_next = position.0 + velocity.0;
            let wall_min: f32 = 3.0;
            let wall_max: f32 = (grid.width - 4) as f32;
            if position_next.x < wall_min {
                velocity.0.x += wall_min - position_next.x;
                velocity.0.y *= 1.0 - BOUNDARY_FRICTION_DAMPING;
            }
            if position_next.x > wall_max {
                velocity.0.x += wall_max - position_next.x;
                velocity.0.y *= 1.0 - BOUNDARY_FRICTION_DAMPING;
            }
            if position_next.y < wall_min {
                velocity.0.y += wall_min - position_next.y;
                velocity.0.x *= 1.0 - BOUNDARY_FRICTION_DAMPING;
            }
            if position_next.y > wall_max {
                velocity.0.y += wall_max - position_next.y;
                velocity.0.x *= 1.0 - BOUNDARY_FRICTION_DAMPING;
            }
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
    particles.par_for_each_mut(&pool, PAR_BATCH_SIZE, |(mut transform, position)| {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;
    });
}

fn particles_to_grid(
    pool: Res<ComputeTaskPool>,
    mut grid: ResMut<Grid>,
    particles: Query<
        (
            &Position,
            &Mass,
            &AffineMomentum,
            &RestDensity,
            &DynamicViscosity,
            &EosStiffness,
            &EosPower,
        ),
        With<ParticleTag>,
    >,
) {
    let momentum_changes = Mutex::new(vec![Vec2::ZERO; grid.width * grid.width]);

    particles.par_for_each(
        &pool,
        PAR_BATCH_SIZE,
        |(
            position,
            mass,
            affine_momentum,
            rest_density,
            dynamic_viscosity,
            eos_stiffness,
            eos_power,
        )| {
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
                eos_stiffness.0 * (f32::powf(density / rest_density.0, eos_power.0) - 1.0),
            );
            let mut stress = Mat2::from_cols(Vec2::new(-pressure, 0.0), Vec2::new(0.0, -pressure));
            let mut strain = affine_momentum.0.clone();
            let trace = strain.y_axis.x + strain.x_axis.y;
            strain.y_axis.x = trace;
            strain.x_axis.y = trace;
            let viscosity_term = strain * dynamic_viscosity.0;
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
    let tex = asset_server.load("branding/icon.png");
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

                new_particle(
                    &mut commands,
                    tex.clone(),
                    grid.current_tick,
                    state.particle_origin,
                    base_vel + random_a_contrib + random_b_contrib,
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
    mut particles: Query<
        (
            &Position,
            &mut Velocity,
            &mut Mass,
            &mut RestDensity,
            &mut DynamicViscosity,
            &mut EosStiffness,
            &mut EosPower,
        ),
        With<ParticleTag>,
    >,
) {
    if buttons.just_pressed(MouseButton::Left) {
        // Left button was pressed

        let window = windows.get_primary().unwrap();
        if let Some(win_pos) = window.cursor_position() {
            // cursor is inside the window.
            // translate window position to grid position
            let scale = window.width() / grid.width as f32;
            let grid_pos = win_pos / scale;
            particles.par_for_each_mut(
                &pool,
                PAR_BATCH_SIZE,
                |(
                    position,
                    mut velocity,
                    mut mass,
                    mut rest_density,
                    mut dynamic_viscosity,
                    mut eos_stiffness,
                    mut eos_power,
                )| {
                    if (grid_pos.x - position.0.x).abs() < 4.0
                        && (grid_pos.y - position.0.y).abs() < 4.0
                    {
                        // todo i dont do anything right now!
                    }
                },
            );
        }
    }
}

fn spawn_square(commands: &mut Commands, tex: Handle<Image>, tick: usize, origin: Vec2) {
    let mut rng = rand::thread_rng();
    let square_vel = Vec2::new(rng.gen::<f32>() * 10.0 - 5., rng.gen::<f32>() * 10.0 - 5.);
    for i in 0..5 {
        for j in 0..5 {
            new_particle(
                commands,
                tex.clone(),
                tick,
                origin + Vec2::new(i as f32, j as f32),
                square_vel,
            );
        }
    }
}

fn new_particle(commands: &mut Commands, tex: Handle<Image>, tick: usize, at: Vec2, vel: Vec2) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: tex.clone(),
            transform: Transform::from_scale(Vec3::splat(0.002)), // todo scale me from grid size or just to look OK
            ..Default::default()
        })
        .insert_bundle((
            Position(at),
            Velocity(vel),
            Mass(1.0),
            AffineMomentum(Mat2::ZERO),
            RestDensity(4.),
            DynamicViscosity(0.1),
            EosStiffness(10.),
            EosPower(4.),
            MaxAge(1000),
            CreatedAt(tick),
            ParticleTag,
        ));
}

fn update_grid(mut grid: ResMut<Grid>) {
    grid.update();
}

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
            // todo created_at and max_age as components.
            created_at: 0,
            spawn_frequency: 1,
            max_particles: 15000,
            particle_origin: Vec2::new(grid.width as f32 / 4., grid.width as f32 / 4.),
            particle_velocity: Vec2::new(0.3, 0.3),
            particle_velocity_random_vec_a: Vec2::new(-0.01, -0.01),
            particle_velocity_random_vec_b: Vec2::new(0.01, 0.01),
        },
        ParticleSpawnerTag,
    ));

    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            spawn_frequency: 1,
            max_particles: 15000,
            particle_origin: Vec2::new(3. * grid.width as f32 / 4., 3. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(-0.3, -0.3),
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
    mut egui_context: ResMut<EguiContext>,
    keys: Res<Input<KeyCode>>,
    mut grid: ResMut<Grid>,
    particles: Query<(Entity), With<ParticleTag>>,
) {
    // todo configure particle age
    // todo place spawners and drag direction

    egui::Window::new("Controls").show(egui_context.ctx_mut(), |ui| {
        if ui.button("(R)eset").clicked() || keys.just_pressed(KeyCode::R) {
            particles.for_each(|(id)| {
                commands.entity(id).despawn();
            });
            return;
        };
        if ui.button("(G)ravity toggle").clicked() || keys.just_pressed(KeyCode::G) {
            grid.toggle_gravity();
            return;
        };
    });
}

fn main() {
    let grid_width = usize::pow(2, 7);
    let grid_zoom = 9.0;
    let window_width = grid_width as f32 * grid_zoom;
    let dt = 0.02;
    let gravity = -0.3;
    App::new()
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
            dt,
            gravity,
            gravity_enabled: false,
            current_tick: 0,
        }) // add global MPM grid
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
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
