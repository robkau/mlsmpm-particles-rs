// there is a resource pointing to current scene
// the current scene can be changed
// on first tick where scene changed, despawn all old entities. spawn each particlespawner out of scene.

use crate::components::{steel_properties, ParticleTag, Scene};
use crate::grid::Grid;
use crate::world::WorldState;
use crate::{
    AssetServer, Commands, Entity, Local, ParticleSpawnerInfo, ParticleSpawnerInfoBuilder, Query,
    Res, ResMut, SceneManager, SpawnerPattern, With, DEFAULT_DT, DEFAULT_GRAVITY,
    STEEL_PARTICLE_MASS,
};
use bevy::prelude::Vec2;

pub(super) fn init_scenes() -> SceneManager {
    let mut sm = SceneManager::default();

    sm.add_scene(hollow_box_scene(0));

    sm.set_current_scene_index(1);

    return sm;
}

pub(super) fn hollow_box_scene(current_tick: usize) -> Scene {
    let mut s = Scene::new(String::from("hollow_box"), DEFAULT_GRAVITY, DEFAULT_DT);

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(current_tick)
            .pattern(SpawnerPattern::Triangle { l: 30 })
            .spawn_frequency(1300)
            .max_particles(200000)
            .particle_duration(40000)
            .particle_origin(Vec2::new(15., 23.))
            .particle_velocity(Vec2::new(95.3, -125.3))
            .particle_velocity_random_vec_a(Vec2::new(-0.0, -1.0))
            .particle_velocity_random_vec_b(Vec2::new(0.0, 0.0))
            .particle_mass(STEEL_PARTICLE_MASS)
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .pattern(SpawnerPattern::Triangle { l: 30 })
            .spawn_frequency(1200)
            .max_particles(200000)
            .particle_duration(40000)
            .particle_origin(Vec2::new(40., 20.))
            .particle_velocity(Vec2::new(-95.3, -9.3))
            .particle_velocity_random_vec_a(Vec2::new(-0.0, -0.0))
            .particle_velocity_random_vec_b(Vec2::new(0.0, 0.0))
            .particle_mass(STEEL_PARTICLE_MASS)
            .build()
            .unwrap(),
    );
    /*
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


     */
    return s;
}

pub(super) fn update_scene(
    mut commands: Commands,
    mut last_frame_scene: Local<usize>,
    scene_manager: Res<SceneManager>,
    mut world: ResMut<WorldState>,
    particles: Query<(Entity), With<ParticleTag>>,
    spawners: Query<(Entity), With<ParticleSpawnerInfo>>,
    asset_server: Res<AssetServer>,
) {
    let current_index = scene_manager.clone().get_current_scene_index();
    let current_scene = scene_manager.clone().get_current_scene();
    if world.current_tick == 0 || !current_index.eq(&*last_frame_scene) {
        // first scene or changed scene
        // remove existing entities
        particles.for_each(|(id)| {
            commands.entity(id).despawn();
        });
        spawners.for_each(|(id)| {
            commands.entity(id).despawn();
        });

        // add new entities
        current_scene.actualize(commands, asset_server, *world); // todo is ok?
    }

    *last_frame_scene = current_index;
}
