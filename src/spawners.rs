use crate::shapes::*;
use bevy::prelude::*;
use rand::Rng;
use std::arch::aarch64::vsliq_n_p8;
use std::f32::consts::PI;

use super::components::*;
use super::grid::*;
use super::world::*;

const LIQUID_PARTICLE_MASS: f32 = 1.;
const WOOD_PARTICLE_MASS: f32 = 1.;
const STEEL_PARTICLE_MASS: f32 = 1.5;

// Tags particle spawner entities
#[derive(Component)]
pub(super) struct ParticleSpawnerTag;

#[allow(dead_code)]
#[derive(Clone)]
pub(super) enum SpawnerPattern {
    SingleParticle,
    LineHorizontal {
        w: usize,
    },
    LineVertical {
        h: usize,
    },
    Rectangle {
        w: usize,
        h: usize,
    },
    Tower {
        w: usize,
        h: usize,
    },
    Triangle {
        l: usize,
    },
    FuncXY {
        f: fn(x: f32, y: f32) -> bool,
        domain: Mat2,
        particles_wide: usize,
        particles_tall: usize,
    }, // Spiral{rotationPerTick: f32, ticksPerSpawn: usize},
}

#[derive(Clone, Component)]
pub(super) struct ParticleSpawnerInfo {
    pub(super) created_at: usize,
    pub(super) pattern: SpawnerPattern,
    pub(super) spawn_frequency: usize,
    pub(super) max_particles: usize,
    pub(super) particle_duration: usize,
    pub(super) particle_origin: Vec2,
    pub(super) particle_velocity: Vec2,
    pub(super) particle_velocity_random_vec_a: Vec2,
    pub(super) particle_velocity_random_vec_b: Vec2,
    pub(super) particle_mass: f32,
}

pub(super) fn create_initial_spawners(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    grid: Res<Grid>,
) {
    // shoot arrows make of steel material
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Triangle { l: 30 },
            spawn_frequency: 1300,
            max_particles: 200000,
            particle_duration: 40000,
            particle_origin: Vec2::new(1.5 * grid.width as f32 / 4., 2.3 * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(95.3, -125.3),
            particle_velocity_random_vec_a: Vec2::new(-0.0, -1.0),
            particle_velocity_random_vec_b: Vec2::new(0.0, 0.0),
            particle_mass: STEEL_PARTICLE_MASS,
        },
        steel_properties(),
        asset_server.load::<Image, &str>("steel_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Triangle { l: 30 },
            spawn_frequency: 1200,
            max_particles: 200000,
            particle_duration: 40000,
            particle_origin: Vec2::new(1.55 * grid.width as f32 / 4., 0.8 * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(-95.3, -9.3),
            particle_velocity_random_vec_a: Vec2::new(-0.0, -0.0),
            particle_velocity_random_vec_b: Vec2::new(0.0, 0.0),
            particle_mass: STEEL_PARTICLE_MASS,
        },
        steel_properties(),
        asset_server.load::<Image, &str>("steel_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Triangle { l: 30 },
            spawn_frequency: 1100,
            max_particles: 200000,
            particle_duration: 40000,
            particle_origin: Vec2::new(0.6 * grid.width as f32 / 4., 2.6 * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(-10.3, -95.3),
            particle_velocity_random_vec_a: Vec2::new(-0.0, -0.0),
            particle_velocity_random_vec_b: Vec2::new(0.0, 0.0),
            particle_mass: STEEL_PARTICLE_MASS,
        },
        steel_properties(),
        asset_server.load::<Image, &str>("steel_particle.png"),
        ParticleSpawnerTag,
    ));

    // steel box filled with water on left
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::FuncXY {
                f: hollow_box_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 90,
                particles_tall: 90,
            },
            spawn_frequency: 25000,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                0.6 * grid.width as f32 / 4. + 12.,
                0.25 * grid.width as f32 / 4. + 16.,
            ),
            particle_velocity: Vec2::new(0., 0.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: STEEL_PARTICLE_MASS,
        },
        steel_properties(),
        asset_server.load::<Image, &str>("steel_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::FuncXY {
                f: circle_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 130,
                particles_tall: 130,
            },
            spawn_frequency: 25000,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                0.6 * grid.width as f32 / 4. + 12.,
                0.25 * grid.width as f32 / 4. + 16.,
            ),
            particle_velocity: Vec2::new(0., 0.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));

    // wood box filled with water on right
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::FuncXY {
                f: hollow_box_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 90,
                particles_tall: 90,
            },
            spawn_frequency: 25000,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                3.1 * grid.width as f32 / 4. + 12.,
                0.25 * grid.width as f32 / 4. + 16.,
            ),
            particle_velocity: Vec2::new(0., 0.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: WOOD_PARTICLE_MASS,
        },
        wood_properties(),
        asset_server.load::<Image, &str>("wood_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::FuncXY {
                f: circle_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 130,
                particles_tall: 130,
            },
            spawn_frequency: 25000,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                3.1 * grid.width as f32 / 4. + 12.,
                0.25 * grid.width as f32 / 4. + 16.,
            ),
            particle_velocity: Vec2::new(0., 0.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));

    // drop down a cool structure
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(10., 160.), Vec2::new(0., 80.)),
                particles_wide: 170,
                particles_tall: 110,
            },
            spawn_frequency: 5000,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                2. * grid.width as f32 / 4. + 12.,
                2.8 * grid.width as f32 / 4. + 64.,
            ),
            particle_velocity: Vec2::new(-0., -20.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: WOOD_PARTICLE_MASS,
        },
        wood_properties(),
        asset_server.load::<Image, &str>("wood_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(10. + PI, 160.), Vec2::new(0., 80.)),
                particles_wide: 170,
                particles_tall: 110,
            },
            spawn_frequency: 5000,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                2. * grid.width as f32 / 4. + 12. - PI,
                2.8 * grid.width as f32 / 4. + 64. - PI,
            ),
            particle_velocity: Vec2::new(0., -20.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));
    /*
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Rectangle {
                w: water_square_width,
                h: water_square_width,
            },
            spawn_frequency: 78,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                0.5 * grid.width as f32 / 4. + 12.,
                3. * grid.width as f32 / 4. + 16.,
            ),
            particle_velocity: Vec2::new(-20., -55.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Rectangle {
                w: water_square_width,
                h: water_square_width,
            },
            spawn_frequency: 478,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                0.5 * grid.width as f32 / 4. + 20.,
                3. * grid.width as f32 / 4. + 12.,
            ),
            particle_velocity: Vec2::new(-20., -35.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Rectangle {
                w: water_square_width,
                h: water_square_width,
            },
            spawn_frequency: 478,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                0.5 * grid.width as f32 / 4. - 16.,
                3. * grid.width as f32 / 4.,
            ),
            particle_velocity: Vec2::new(30., -35.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Rectangle {
                w: water_square_width,
                h: water_square_width,
            },
            spawn_frequency: 800,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                0.5 * grid.width as f32 / 4. - 8.,
                3. * grid.width as f32 / 4.,
            ),
            particle_velocity: Vec2::new(40., -45.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Rectangle {
                w: water_square_width,
                h: water_square_width,
            },
            spawn_frequency: 700,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(0.5 * grid.width as f32 / 4., 3. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(50., -45.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Rectangle {
                w: water_square_width,
                h: water_square_width,
            },
            spawn_frequency: 600,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                0.5 * grid.width as f32 / 4. + 8.,
                3. * grid.width as f32 / 4.,
            ),
            particle_velocity: Vec2::new(10., -45.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: LIQUID_PARTICLE_MASS,
        },
        water_properties(),
        asset_server.load::<Image, &str>("liquid_particle.png"),
        ParticleSpawnerTag,
    ));
    */
}

pub(super) fn tick_spawners(
    mut commands: Commands,
    world: Res<WorldState>,
    grid: Res<Grid>,
    particles: Query<(), With<ParticleTag>>,
    spawners_solids: Query<
        (
            &ParticleSpawnerInfo,
            &NeoHookeanHyperElasticModel,
            &Handle<Image>,
        ),
        With<ParticleSpawnerTag>,
    >,
    spawners_fluids: Query<
        (&ParticleSpawnerInfo, &NewtonianFluidModel, &Handle<Image>),
        With<ParticleSpawnerTag>,
    >,
) {
    // todo recreate spiral spawn pattern - rate per spawn and rotation per spawn

    spawners_solids.for_each(|(spawner_info, particle_properties, texture)| {
        if (world.current_tick - spawner_info.created_at) % spawner_info.spawn_frequency == 0
            && particles.iter().count() < spawner_info.max_particles
        {
            spawn_particles(
                spawner_info,
                *particle_properties,
                &mut commands,
                texture.clone(),
                &world,
                &grid,
            );
        }
    });

    spawners_fluids.for_each(|(spawner_info, particle_properties, texture)| {
        if (world.current_tick - spawner_info.created_at) % spawner_info.spawn_frequency == 0
            && particles.iter().count() < spawner_info.max_particles
        {
            spawn_particles(
                spawner_info,
                *particle_properties,
                &mut commands,
                texture.clone(),
                &world,
                &grid,
            );
        }
    });
}

fn spawn_particle(
    commands: &mut Commands,
    grid_width: usize,
    cm: impl ConstitutiveModel + Copy,
    spawner_info: &ParticleSpawnerInfo,
    spawn_offset: Vec2,
    vel: Option<Vec2>,
    texture: Handle<Image>,
    created_at: usize,
) {
    let particle_position = spawner_info.particle_origin + spawn_offset;

    let min = 3;
    let max = grid_width - 4;
    if particle_position.x <= min as f32 || particle_position.x >= max as f32 {
        return;
    }
    if particle_position.y <= min as f32 || particle_position.y >= max as f32 {
        return;
    }

    cm.new_particle(
        commands,
        texture.clone(),
        particle_position,
        spawner_info.particle_mass,
        created_at,
        vel,
        Some(spawner_info.particle_duration),
    );
}

pub(super) fn spawn_particles(
    spawner_info: &ParticleSpawnerInfo,
    cm: impl ConstitutiveModel + Copy,
    commands: &mut Commands,
    texture: Handle<Image>,
    world: &WorldState,
    grid: &Res<Grid>,
) {
    let mut rng = rand::thread_rng();
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

    match spawner_info.pattern {
        SpawnerPattern::SingleParticle => {
            spawn_particle(
                commands,
                grid.width,
                cm,
                spawner_info,
                Vec2::ZERO,
                Some(spawn_vel),
                texture.clone(),
                world.current_tick,
            );
        }
        SpawnerPattern::LineHorizontal { w } => {
            for x in 0..w {
                spawn_particle(
                    commands,
                    grid.width,
                    cm,
                    spawner_info,
                    Vec2::new(x as f32, 0.),
                    Some(spawn_vel),
                    texture.clone(),
                    world.current_tick,
                );
            }
        }
        SpawnerPattern::LineVertical { h } => {
            for y in 0..h {
                spawn_particle(
                    commands,
                    grid.width,
                    cm,
                    spawner_info,
                    Vec2::new(0., y as f32),
                    Some(spawn_vel),
                    texture.clone(),
                    world.current_tick,
                );
            }
        }
        SpawnerPattern::Rectangle { w, h } => {
            for x in 0..w {
                for y in 0..h {
                    spawn_particle(
                        commands,
                        grid.width,
                        cm,
                        spawner_info,
                        Vec2::new(x as f32 + 0.001, y as f32 + 0.001),
                        Some(spawn_vel),
                        texture.clone(),
                        world.current_tick,
                    );
                }
            }
        }
        SpawnerPattern::Tower { w, h } => {
            for x in 0..w {
                for y in 0..h {
                    spawn_particle(
                        commands,
                        grid.width,
                        cm,
                        spawner_info,
                        Vec2::new(x as f32, y as f32),
                        Some(spawn_vel),
                        texture.clone(),
                        world.current_tick,
                    );
                }
            }
        }
        SpawnerPattern::Triangle { l } => {
            let x_axis: Vec2 = Vec2::new(1., 0.);
            let angle = match spawn_vel.length() {
                0. => 0.,
                _ => x_axis.angle_between(spawn_vel),
            };

            for x in 0..l {
                for y in 0..x {
                    // offset y by 0.5 every other time
                    let mut ya: f32 = if x % 2 == 0 {
                        y as f32 - 0.25
                    } else {
                        y as f32 + 0.25
                    };
                    ya -= x as f32 / 2.;

                    // rotate by angle about triangle tip
                    let pivot: Vec2 = Vec2::new(l as f32, 0.);
                    let pos: Vec2 = Vec2::new(
                        (0.001 + pivot.x - x as f32) / 4.,
                        (0.001 + pivot.y + ya as f32) / 4.,
                    );

                    // 1. translate point back to origin
                    let pos_rel_origin: Vec2 = Vec2::new(pos.x - pivot.x, pos.y - pivot.y);

                    // 2. rotate point
                    let pos_rel_origin_rotated: Vec2 = Vec2::new(
                        pos_rel_origin.x * angle.cos() - pos_rel_origin.y * angle.sin(),
                        pos_rel_origin.x * angle.sin() + pos_rel_origin.y * angle.cos(),
                    );

                    // 3. translate point back:
                    let pos_rotated: Vec2 = pos_rel_origin_rotated + pivot;

                    spawn_particle(
                        commands,
                        grid.width,
                        cm,
                        spawner_info,
                        pos_rotated,
                        Some(spawn_vel),
                        texture.clone(),
                        world.current_tick,
                    );
                }
            }
        }
        SpawnerPattern::FuncXY {
            f,
            domain,
            particles_wide,
            particles_tall,
        } => {
            // todo fixme with circle case
            let units_per_particle = Vec2::new(
                domain.x_axis.length() / particles_wide as f32,
                domain.y_axis.length() / particles_tall as f32,
            );

            for x in 0..particles_wide {
                for y in 0..particles_tall {
                    let xp = x as f32 * units_per_particle.x - domain.x_axis.length() / 2.;
                    let yp = y as f32 * units_per_particle.y - domain.y_axis.length() / 2.;

                    if f(xp as f32, yp as f32) {
                        spawn_particle(
                            commands,
                            grid.width,
                            cm,
                            spawner_info,
                            Vec2::new(xp as f32, yp as f32),
                            Some(spawn_vel),
                            texture.clone(),
                            world.current_tick,
                        );
                    }
                }
            }
        } //SpawnerPattern::Spiral => {
          //
          //}
    }
}
