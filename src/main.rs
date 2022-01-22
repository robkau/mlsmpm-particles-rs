use std::f32::consts::PI;
use std::ops::{Mul};
use std::sync::Mutex;
use bevy::{math::{Mat2, f32::*}, prelude::*, tasks::prelude::*};

const GRAVITY: f32 = -0.3;
const REST_DENSITY: f32 = 4.0;
const DYNAMIC_VISCOSITY: f32 = 0.1;
const EOS_STIFFNESS: f32 = 10.0;
const EOS_POWER: f32 = 4.0;
const BOUNDARY_FRICTION_DAMPING: f32 = 0.001;


// Marks particle entities
#[derive(Component)]
struct Particle;

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

// MPM grid resource
#[derive(Clone)]
struct Grid {
    cells: Vec<Cell>,
    width: usize,
    dt: f32
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

    pub fn update(&mut self) {
        for (i, cell) in self.cells.iter_mut().enumerate() {
            if cell.mass > 0.0 {
                // convert momentum to velocity, apply gravity
                cell.velocity *= (1.0 / cell.mass);
                cell.velocity.y += self.dt * GRAVITY;
                
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
     			if x > self.width-3 {
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
     			if y > self.width-3 {
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

#[derive(Debug, Clone,Copy)]
struct Cell {
    velocity: Vec2,
    mass: f32
}

fn quadratic_interpolation_weights(cell_diff: Vec2) -> [Vec2; 3] {
    [Vec2::new(0.5 * f32::powi(0.5-cell_diff.x, 2), 0.5 * f32::powi(0.5-cell_diff.y, 2)),
     Vec2::new(0.75 - f32::powi(cell_diff.x, 2), 0.75 - f32::powi(cell_diff.y, 2)),
     Vec2::new(0.5 * f32::powi(0.5+cell_diff.x, 2), 0.5 * f32::powi(0.5+cell_diff.y, 2))]
}

fn weighted_velocity_and_cell_dist_to_term(weighted_velocity: Vec2, cell_dist: Vec2) -> Mat2 {
    Mat2::from_cols(
        Vec2::new(weighted_velocity[0] * cell_dist[0], weighted_velocity[1] * cell_dist[0]),
        Vec2::new(weighted_velocity[0] * cell_dist[1], weighted_velocity[1] * cell_dist[1]),
    )
}

// G2P MPM step
fn grid_to_particles(pool: Res<ComputeTaskPool>, grid: Res<Grid>, mut particles: Query<(&mut Position, &mut Velocity, &mut AffineMomentum), With<Particle>>) {
    particles.par_for_each_mut(&pool, 32, |(mut position, mut velocity, mut affine_momentum)| {
        //// reset particle velocity. we calculate it from scratch each step using the grid
        velocity.0 = Vec2::ZERO;

        let cell_x : u32 = position.0.x as u32;
        let cell_y : u32 = position.0.y as u32;
        let cell_diff = Vec2::new(position.0.x - cell_x as f32 - 0.5, position.0.y - cell_y as f32 - 0.5);
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

                let cell_at_index= grid.index_at(cell_pos_x as usize, cell_pos_y as usize);
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
        let wall_max: f32= (grid.width - 4) as f32;
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
    });
}

fn update_sprites(pool: Res<ComputeTaskPool>, mut particles: Query<(&mut Transform, &Position), With<Particle>>) {
    particles.par_for_each_mut(&pool, 32, |(mut transform, position)| {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;
    });
}

fn particles_to_grid(pool: Res<ComputeTaskPool>, mut grid: ResMut<Grid>, particles: Query<(&Position, &Mass, &AffineMomentum), With<Particle>>) {
    let momentum_changes = Mutex::new(vec!(Vec2::ZERO; grid.width*grid.width));

    particles.par_for_each(&pool, 32, |(position, mass, affine_momentum)| {
        let cell_x : u32 = position.0.x as u32;
        let cell_y : u32 = position.0.y as u32;
        let cell_diff = Vec2::new(position.0.x - cell_x as f32 - 0.5, position.0.y - cell_y as f32 - 0.5);
        let weights = quadratic_interpolation_weights(cell_diff);

        // check surrounding 9 cells to get volume from density
       let mut density: f32  = 0.0;
        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;
                let cell_pos_x = (cell_x as i32 + gx as i32) - 1;
                let cell_pos_y = (cell_y as i32 + gy as i32) - 1;
                let cell_at_index= grid.index_at(cell_pos_x as usize, cell_pos_y as usize);
                density += grid.cells[cell_at_index].mass * weight;
            }
        }
        
       let volume = mass.0 / density;

        // fluid constitutive model
        let pressure = f32::max(-0.1, EOS_STIFFNESS*(f32::powf(density/REST_DENSITY, EOS_POWER)-1.0));
        let mut stress = Mat2::from_cols(Vec2::new(-pressure, 0.0), Vec2::new(0.0, -pressure));
        let mut strain = affine_momentum.0.clone();
        let trace = strain.y_axis.x + strain.x_axis.y; // todo review me
        strain.y_axis.x = trace;
        strain.x_axis.y = trace;
        let viscosity_term = strain * DYNAMIC_VISCOSITY;
        stress += viscosity_term;

        let eq_16_term_0 = stress * (-volume * 4.0 * grid.dt);

        // for all surrounding 9 cells
        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;
                let cell_pos_x = (cell_x as i32 + gx as i32) - 1;
                let cell_pos_y = (cell_y as i32 + gy as i32) - 1;
                let cell_dist = Vec2::new(cell_pos_x as f32  - position.0.x + 0.5, cell_pos_y as f32 - position.0.y + 0.5);
                let cell_at_index= grid.index_at(cell_pos_x as usize, cell_pos_y as usize);

                let momentum = eq_16_term_0 * weight * cell_dist; // todo review me
                // todo not assigned?
                let mut m = momentum_changes.lock().unwrap();
                m[cell_at_index as usize] += momentum;
            }
        }
    });

    // apply calculated momentum changes
    for (i, change) in momentum_changes.lock().unwrap().iter().enumerate() {
        grid.cells[i].velocity += *change;
    }
}

fn reset_grid(mut grid: ResMut<Grid>) {
    grid.reset();
}

fn update_grid(mut grid: ResMut<Grid>) {
    grid.update();
}

fn update_cells(pool: Res<ComputeTaskPool>, mut grid: ResMut<Grid>, particles: Query<(&Position, &Velocity, &Mass, &AffineMomentum), With<Particle>>) {
    let mass_contrib_changes = Mutex::new(vec!((0.0, Vec2::ZERO); grid.width*grid.width));

    particles.par_for_each(&pool, 32, |(position, velocity, mass, affine_momentum)| {
        println!("pos {:?} vel {:?} ", position, velocity);
        let cell_x : u32 = position.0.x as u32;
        let cell_y : u32 = position.0.y as u32;
        let cell_diff = Vec2::new(position.0.x - cell_x as f32 - 0.5, position.0.y - cell_y as f32 - 0.5);
        let weights = quadratic_interpolation_weights(cell_diff);


        // for all surrounding 9 cells
        for gx in 0..3 {
            for gy in 0..3 {
                let weight = weights[gx].x * weights[gy].y;
                let cell_pos_x = (cell_x as i32 + gx as i32) - 1;
                let cell_pos_y = (cell_y as i32 + gy as i32) - 1;
                let cell_dist = Vec2::new(cell_pos_x as f32  - position.0.x + 0.5, cell_pos_y as f32 - position.0.y + 0.5);
                let cell_at_index= grid.index_at(cell_pos_x as usize, cell_pos_y as usize);

                let q = affine_momentum.0 * cell_dist;
                let mass_contrib = weight * mass.0;
 				// mass and momentum update
                let mut mc = mass_contrib_changes.lock().unwrap();
                mc[cell_at_index].0 += mass_contrib;
                mc[cell_at_index].1 += (velocity.0 + q) * mass_contrib;
            }
        }
    });

    for (i, changes) in mass_contrib_changes.lock().unwrap().iter().enumerate() {
        grid.cells[i].mass += (*changes).0;
        grid.cells[i].velocity += (*changes).1;
    }
}


fn spawn_system(mut commands: Commands, asset_server: Res<AssetServer>)  {
    let texture = asset_server.load("branding/icon.png");

    let mut cb = OrthographicCameraBundle::new_2d();
    cb.orthographic_projection.scale = 0.6;  // todo this should scaled from grid_size
    cb.transform.rotate(Quat::from_rotation_z(PI));
    commands.spawn_bundle(cb);

        //.insert(Transform::from_xyz(0.0, 0.0, 1.0));

    commands.spawn_bundle(
        SpriteBundle {
            texture: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(0.1)),  // todo scale me from grid size or just to look OK
            ..Default::default()
        })
        .insert_bundle(
            (Position(Vec2::new(12.0, 12.0)),
            Velocity(Vec2::new(-1.0, -1.0)),
            Mass(1.0),
            AffineMomentum(Mat2 ::ZERO),
            Particle,
        ));
    commands.spawn_bundle(
        SpriteBundle {
            texture: texture.clone(),
            transform: Transform::from_scale(Vec3::splat(0.1)),
            ..Default::default()
        })
        .insert_bundle(
            (Position(Vec2::new(25.0, 25.0)),
             Velocity(Vec2::new(1.0, 1.0)),
             Mass(1.0),
             AffineMomentum(Mat2::ZERO),
             Particle,
            ));
}



fn main() {
    let grid_width = 64;
    let dt = 0.01;
    App::new()
        .insert_resource(Grid{cells: vec!(Cell{ velocity: Vec2::ZERO, mass: 0.0 }; grid_width * grid_width), width: grid_width, dt }) // add global MPM grid
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_system)
        .add_system(reset_grid.label("reset_grid").before("update_cells"))
        .add_system(update_cells.label("update_cells").before("p2g"))
        .add_system(particles_to_grid.label("p2g").before("update_grid"))
        .add_system(update_grid.label("update_grid").before("g2p"))
        .add_system(grid_to_particles.label("g2p").before("update_sprites"))
        .add_system(update_sprites.label("update_sprites"))
        .run();
}



mod test;