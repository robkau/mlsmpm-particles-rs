use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::grid::*;
use super::world::*;

const LIQUID_PARTICLE_MASS: f32 = 0.5;
const WOOD_PARTICLE_MASS: f32 = 1.;
const STEEL_PARTICLE_MASS: f32 = 1.5;

// Tags particle spawner entities
#[derive(Component)]
pub(super) struct ParticleSpawnerTag;

// todo refactor.
#[allow(dead_code)]
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
            particle_origin: Vec2::new(1.1 * grid.width as f32 / 4., 1. * grid.width as f32 / 4.),
            particle_velocity: Vec2::new(100.3, -1.3),
            particle_velocity_random_vec_a: Vec2::new(-0.0, -0.0),
            particle_velocity_random_vec_b: Vec2::new(0.0, 0.0),
            particle_mass: STEEL_PARTICLE_MASS,
        },
        steel_properties(),
        asset_server.load::<Image, &str>("steel_particle.png"),
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
            spawn_frequency: 99999999,
            max_particles: 50000,
            particle_duration: 500000,
            particle_origin: Vec2::new(2.5 * grid.width as f32 / 4., 1.),
            particle_velocity: Vec2::ZERO,
            particle_velocity_random_vec_a: Vec2::ZERO,
            particle_velocity_random_vec_b: Vec2::ZERO,
            particle_mass: WOOD_PARTICLE_MASS,
        },
        NeoHookeanHyperElasticModel {
            deformation_gradient: Default::default(),
            elastic_lambda: 9. * 1000.,
            elastic_mu: 0.6 * 1000.,
        },
        asset_server.load::<Image, &str>("wood_particle.png"),
        ParticleSpawnerTag,
    ));

    // make it rain!
    commands.spawn_bundle((
        ParticleSpawnerInfo {
            created_at: 0,
            pattern: SpawnerPattern::Cube,
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
            pattern: SpawnerPattern::Cube,
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
            pattern: SpawnerPattern::Cube,
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
            pattern: SpawnerPattern::Cube,
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
            pattern: SpawnerPattern::Cube,
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
            pattern: SpawnerPattern::Cube,
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
}

pub(super) fn tick_spawners(
    mut commands: Commands,
    world: Res<WorldState>,
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
            );
        }
    });
}

pub(super) fn spawn_particles(
    spawner_info: &ParticleSpawnerInfo,
    cm: impl ConstitutiveModel + Copy,
    commands: &mut Commands,
    texture: Handle<Image>,
    world: &WorldState,
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
            cm.new_particle(
                commands,
                texture,
                spawner_info.particle_origin,
                spawner_info.particle_mass,
                world.current_tick,
                Some(spawn_vel),
                Some(spawner_info.particle_duration),
            );
        }
        SpawnerPattern::LineHorizontal => {
            for x in 0..100 {
                cm.new_particle(
                    commands,
                    texture.clone(),
                    spawner_info.particle_origin + Vec2::new(x as f32, 0.),
                    spawner_info.particle_mass,
                    world.current_tick,
                    Some(spawn_vel),
                    Some(spawner_info.particle_duration),
                );
            }
        }
        SpawnerPattern::LineVertical => {
            for y in 0..15 {
                cm.new_particle(
                    commands,
                    texture.clone(),
                    spawner_info.particle_origin + Vec2::new(0., y as f32),
                    spawner_info.particle_mass,
                    world.current_tick,
                    Some(spawn_vel),
                    Some(spawner_info.particle_duration),
                );
            }
        }
        SpawnerPattern::Cube => {
            for x in 0..15 {
                for y in 0..15 {
                    cm.new_particle(
                        commands,
                        texture.clone(),
                        spawner_info.particle_origin + Vec2::new(x as f32, y as f32),
                        spawner_info.particle_mass,
                        world.current_tick,
                        Some(spawn_vel),
                        Some(spawner_info.particle_duration),
                    );
                }
            }
        }
        SpawnerPattern::Tower => {
            for x in 0..80 {
                for y in 0..90 {
                    cm.new_particle(
                        commands,
                        texture.clone(),
                        spawner_info.particle_origin + Vec2::new(x as f32, y as f32),
                        spawner_info.particle_mass,
                        world.current_tick,
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
                    let mut ya: f32 = if x % 2 == 0 {
                        y as f32 - 0.25
                    } else {
                        y as f32 + 0.25
                    };
                    ya -= x as f32 / 2.;

                    cm.new_particle(
                        commands,
                        texture.clone(),
                        spawner_info.particle_origin + Vec2::new(x as f32, ya as f32),
                        spawner_info.particle_mass,
                        world.current_tick,
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
                    let mut ya: f32 = if x % 2 == 0 {
                        y as f32 - 0.25
                    } else {
                        y as f32 + 0.25
                    };
                    ya -= x as f32 / 2.;

                    cm.new_particle(
                        commands,
                        texture.clone(),
                        spawner_info.particle_origin
                            + Vec2::new((15 - x) as f32 / 4., ya as f32 / 4.),
                        spawner_info.particle_mass,
                        world.current_tick,
                        Some(spawn_vel),
                        Some(spawner_info.particle_duration),
                    );
                }
            }
        }
    }
}
