use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::grid::*;
use super::particle::*;
use super::world::*;

// Tags particle spawner entities
#[derive(Component)]
pub(super) struct ParticleSpawnerTag;

// todo refactor.
#[derive(Clone)]
pub(super) enum SpawnerPattern {
    SingleParticle,
    LineHorizontal,
    LineVertical,
    Cube,
    Tower,
    TriangleLeft,
    TriangleRight,
}

#[derive(Clone, Component)]
pub(super) struct ParticleSpawnerInfo {
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
}

pub(super) fn create_initial_spawners(mut commands: Commands, grid: Res<Grid>) {
    // shoot arrows to the right
    // young's modulus and shear modulus of steel.
    // 180 Gpa young's
    // 78Gpa shear
    commands.spawn_bundle((
        // todo density option to spawners
        // todo calculate correct particle mass from material density and particle density
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::TriangleRight,
            spawn_frequency: 800,
            max_particles: 200000,
            particle_duration: 40000,
            particle_origin: Vec2::new(1.5 * grid.width as f32 / 4., 1. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(100.3, -1.3),
            particle_velocity_random_vec_a: Vec2::new(-0.0, -0.0),
            particle_velocity_random_vec_b: Vec2::new(0.0, 0.0),
            particle_mass: 2.,
        },
        ConstitutiveModelNeoHookeanHyperElastic {
            deformation_gradient: Default::default(),
            elastic_lambda: 180. * 1000.,
            elastic_mu: 78. * 1000.,
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
        },
        ConstitutiveModelNeoHookeanHyperElastic {
            deformation_gradient: Default::default(),
            elastic_lambda: 9. * 1000.,
            elastic_mu: 0.6 * 1000.,
        },
        ParticleSpawnerTag,
    ));

    // make it rain!
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Cube,
            spawn_frequency: 678,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                2.5 * grid.width as f32 / 4. + 12.,
                3. * grid.width as f32 / 4. + 16.,
            ),
            particle_velocity: Vec2::new(-20., -55.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: 0.25,
        },
        ConstitutiveModelFluid {
            rest_density: 4.,
            dynamic_viscosity: 0.1,
            eos_stiffness: 100.,
            eos_power: 4.,
        },
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Cube,
            spawn_frequency: 478,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                2.5 * grid.width as f32 / 4. + 20.,
                3. * grid.width as f32 / 4. + 12.,
            ),
            particle_velocity: Vec2::new(-20., -35.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: 0.25,
        },
        ConstitutiveModelFluid {
            rest_density: 4.,
            dynamic_viscosity: 0.1,
            eos_stiffness: 100.,
            eos_power: 4.,
        },
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Cube,
            spawn_frequency: 478,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                2.5 * grid.width as f32 / 4. - 16.,
                3. * grid.width as f32 / 4.,
            ),
            particle_velocity: Vec2::new(30., -35.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: 0.25,
        },
        ConstitutiveModelFluid {
            rest_density: 4.,
            dynamic_viscosity: 0.1,
            eos_stiffness: 100.,
            eos_power: 4.,
        },
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Cube,
            spawn_frequency: 800,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                2.5 * grid.width as f32 / 4. - 8.,
                3. * grid.width as f32 / 4.,
            ),
            particle_velocity: Vec2::new(40., -45.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: 0.25,
        },
        ConstitutiveModelFluid {
            rest_density: 4.,
            dynamic_viscosity: 0.1,
            eos_stiffness: 100.,
            eos_power: 4.,
        },
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Cube,
            spawn_frequency: 700,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(2.5 * grid.width as f32 / 4., 3. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(50., -45.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: 0.25,
        },
        ConstitutiveModelFluid {
            rest_density: 4.,
            dynamic_viscosity: 0.1,
            eos_stiffness: 100.,
            eos_power: 4.,
        },
        ParticleSpawnerTag,
    ));
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Cube,
            spawn_frequency: 600,
            max_particles: 75000,
            particle_duration: 100000,
            particle_origin: Vec2::new(
                2.5 * grid.width as f32 / 4. + 8.,
                3. * grid.width as f32 / 4.,
            ),
            particle_velocity: Vec2::new(10., -45.),
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: 0.25,
        },
        ConstitutiveModelFluid {
            rest_density: 4.,
            dynamic_viscosity: 0.1,
            eos_stiffness: 100.,
            eos_power: 4.,
        },
        ParticleSpawnerTag,
    ));
}

pub(super) fn tick_spawners(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world: Res<WorldState>,
    particles: Query<(), With<ParticleTag>>,
    spawners_solids: Query<
        (
            &ParticleSpawnerInfo,
            &ConstitutiveModelNeoHookeanHyperElastic,
        ),
        With<ParticleSpawnerTag>,
    >,
    spawners_fluids: Query<
        (&ParticleSpawnerInfo, &ConstitutiveModelFluid),
        With<ParticleSpawnerTag>,
    >,
) {
    // todo recreate spiral spawn pattern - rate per spawn and rotation per spawn

    // todo refactor me to eliminate big copy paste.
    let mut rng = rand::thread_rng();
    spawners_solids.for_each(|(spawner_info, particle_properties)| {
        if (world.current_tick - spawner_info.created_at) % spawner_info.spawn_frequency == 0 {
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

                match spawner_info.pattern {
                    SpawnerPattern::SingleParticle => {
                        // todo the spawner should alrady know the ocnstitutive properties
                        new_solid_particle(
                            &mut commands,
                            &asset_server,
                            world.current_tick,
                            spawner_info.particle_origin,
                            particle_properties.clone(),
                            spawner_info.particle_mass,
                            Some(spawn_vel),
                            Some(spawner_info.particle_duration),
                        );
                    }
                    SpawnerPattern::LineHorizontal => {
                        for x in 0..100 {
                            new_solid_particle(
                                &mut commands,
                                &asset_server,
                                world.current_tick,
                                spawner_info.particle_origin + Vec2::new(x as f32, 0.),
                                particle_properties.clone(),
                                spawner_info.particle_mass,
                                Some(spawn_vel),
                                Some(spawner_info.particle_duration),
                            );
                        }
                    }
                    SpawnerPattern::LineVertical => {
                        for y in 0..15 {
                            new_solid_particle(
                                &mut commands,
                                &asset_server,
                                world.current_tick,
                                spawner_info.particle_origin + Vec2::new(0., y as f32),
                                particle_properties.clone(),
                                spawner_info.particle_mass,
                                Some(spawn_vel),
                                Some(spawner_info.particle_duration),
                            );
                        }
                    }
                    SpawnerPattern::Cube => {
                        for x in 0..15 {
                            for y in 0..15 {
                                new_solid_particle(
                                    &mut commands,
                                    &asset_server,
                                    world.current_tick,
                                    spawner_info.particle_origin + Vec2::new(x as f32, y as f32),
                                    particle_properties.clone(),
                                    spawner_info.particle_mass,
                                    Some(spawn_vel),
                                    Some(spawner_info.particle_duration),
                                );
                            }
                        }
                    }
                    SpawnerPattern::Tower => {
                        for x in 0..60 {
                            for y in 0..200 {
                                new_solid_particle(
                                    &mut commands,
                                    &asset_server,
                                    world.current_tick,
                                    spawner_info.particle_origin + Vec2::new(x as f32, y as f32),
                                    particle_properties.clone(),
                                    spawner_info.particle_mass,
                                    Some(spawn_vel),
                                    Some(spawner_info.particle_duration),
                                );
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

                                new_solid_particle(
                                    &mut commands,
                                    &asset_server,
                                    world.current_tick,
                                    spawner_info.particle_origin + Vec2::new(x as f32, ya as f32),
                                    particle_properties.clone(),
                                    spawner_info.particle_mass,
                                    Some(spawn_vel),
                                    Some(spawner_info.particle_duration),
                                );
                            }
                        }
                    }
                    SpawnerPattern::TriangleRight => {
                        for x in 0..30 {
                            for y in 0..x {
                                // offset y by 0.5 every other time
                                let mut ya: f32;
                                if (x) % 2 == 0 {
                                    ya = y as f32 - 0.25;
                                } else {
                                    ya = y as f32 + 0.25;
                                }
                                ya -= x as f32 / 2.;

                                new_solid_particle(
                                    &mut commands,
                                    &asset_server,
                                    world.current_tick,
                                    spawner_info.particle_origin
                                        + Vec2::new((15 - x) as f32 / 4., ya as f32 / 4.),
                                    particle_properties.clone(),
                                    spawner_info.particle_mass,
                                    Some(spawn_vel),
                                    Some(spawner_info.particle_duration),
                                );
                            }
                        }
                    }
                }
            }
        }
    });

    spawners_fluids.for_each(|(spawner_info, particle_properties)| {
        if (world.current_tick - spawner_info.created_at) % spawner_info.spawn_frequency == 0 {
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

                match spawner_info.pattern {
                    SpawnerPattern::SingleParticle => {
                        // todo the spawner should alrady know the ocnstitutive properties
                        new_fluid_particle(
                            &mut commands,
                            &asset_server,
                            world.current_tick,
                            spawner_info.particle_origin,
                            particle_properties.clone(),
                            spawner_info.particle_mass,
                            Some(spawn_vel),
                            Some(spawner_info.particle_duration),
                        );
                    }
                    SpawnerPattern::LineHorizontal => {
                        for x in 0..100 {
                            new_fluid_particle(
                                &mut commands,
                                &asset_server,
                                world.current_tick,
                                spawner_info.particle_origin + Vec2::new(x as f32, 0.),
                                particle_properties.clone(),
                                spawner_info.particle_mass,
                                Some(spawn_vel),
                                Some(spawner_info.particle_duration),
                            );
                        }
                    }
                    SpawnerPattern::LineVertical => {
                        for y in 0..15 {
                            new_fluid_particle(
                                &mut commands,
                                &asset_server,
                                world.current_tick,
                                spawner_info.particle_origin + Vec2::new(0., y as f32),
                                particle_properties.clone(),
                                spawner_info.particle_mass,
                                Some(spawn_vel),
                                Some(spawner_info.particle_duration),
                            );
                        }
                    }
                    SpawnerPattern::Cube => {
                        for x in 0..15 {
                            for y in 0..15 {
                                new_fluid_particle(
                                    &mut commands,
                                    &asset_server,
                                    world.current_tick,
                                    spawner_info.particle_origin + Vec2::new(x as f32, y as f32),
                                    particle_properties.clone(),
                                    spawner_info.particle_mass,
                                    Some(spawn_vel),
                                    Some(spawner_info.particle_duration),
                                );
                            }
                        }
                    }
                    SpawnerPattern::Tower => {
                        for x in 0..60 {
                            for y in 0..200 {
                                new_fluid_particle(
                                    &mut commands,
                                    &asset_server,
                                    world.current_tick,
                                    spawner_info.particle_origin + Vec2::new(x as f32, y as f32),
                                    particle_properties.clone(),
                                    spawner_info.particle_mass,
                                    Some(spawn_vel),
                                    Some(spawner_info.particle_duration),
                                );
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

                                new_fluid_particle(
                                    &mut commands,
                                    &asset_server,
                                    world.current_tick,
                                    spawner_info.particle_origin + Vec2::new(x as f32, ya as f32),
                                    particle_properties.clone(),
                                    spawner_info.particle_mass,
                                    Some(spawn_vel),
                                    Some(spawner_info.particle_duration),
                                );
                            }
                        }
                    }
                    SpawnerPattern::TriangleRight => {
                        for x in 0..30 {
                            for y in 0..x {
                                // offset y by 0.5 every other time
                                let mut ya: f32;
                                if (x) % 2 == 0 {
                                    ya = y as f32 - 0.25;
                                } else {
                                    ya = y as f32 + 0.25;
                                }
                                ya -= x as f32 / 2.;

                                new_fluid_particle(
                                    &mut commands,
                                    &asset_server,
                                    world.current_tick,
                                    spawner_info.particle_origin
                                        + Vec2::new((15 - x) as f32 / 4., ya as f32 / 4.),
                                    particle_properties.clone(),
                                    spawner_info.particle_mass,
                                    Some(spawn_vel),
                                    Some(spawner_info.particle_duration),
                                );
                            }
                        }
                    }
                }
            }
        }
    });
}
