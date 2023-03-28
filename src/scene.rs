// there is a resource pointing to current scene
// the current scene can be changed
// on first tick where scene changed, despawn all old entities. spawn each particlespawner out of scene.
use std::f32::consts::PI;

use crate::prelude::*;

pub(crate) fn update_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_scene: Res<ParticleScene>,
    mut last_frame_scene: Local<String>,
    mut world: ResMut<WorldState>,
    mut need_to_reset: ResMut<NeedToReset>,
    particles: Query<Entity, With<ParticleTag>>,
    spawners: Query<Entity, With<ParticleSpawnerInfo>>,
) {
    if world.current_tick == 0  // first scene
        || !current_scene.clone().name().eq(&*last_frame_scene) // changed scene
        || need_to_reset.0
    // reset scene
    {
        // remove existing entities
        particles.for_each(|id| {
            commands.entity(id).despawn();
        });
        spawners.for_each(|id| {
            commands.entity(id).despawn();
        });
        // add new entities
        current_scene
            .clone()
            .actualize(&mut commands, &mut world, &asset_server);

        need_to_reset.0 = false;
    }

    *last_frame_scene = current_scene.clone().name();
}

pub(crate) fn hollow_box_scene() -> ParticleScene {
    let mut s = ParticleScene::new(String::from("hollow_box"), DEFAULT_GRAVITY, DEFAULT_DT);

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::Triangle { l: 25 })
            .spawn_on_creation(true)
            .spawn_frequency(1300)
            .max_particles(200000)
            .particle_duration(40000)
            .particle_origin(Vec2::new(15., 115.))
            .particle_velocity(Vec2::new(25.3, -125.3))
            .particle_velocity_random_vec_a(Vec2::new(-0.0, -1.0))
            .particle_velocity_random_vec_b(Vec2::new(0.0, 0.0))
            .particle_type(SpawnedParticleType::steel())
            .particle_texture("steel_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::Triangle { l: 25 })
            .spawn_on_creation(true)
            .spawn_frequency(1100)
            .max_particles(200000)
            .particle_duration(40000)
            .particle_origin(Vec2::new(15., 95.))
            .particle_velocity(Vec2::new(-10.3, -95.3))
            .particle_velocity_random_vec_a(Vec2::new(-0.0, -0.0))
            .particle_velocity_random_vec_b(Vec2::new(0.0, 0.0))
            .particle_type(SpawnedParticleType::steel())
            .particle_texture("steel_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: hollow_box_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 100,
                particles_tall: 100,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(45., 35.))
            .particle_velocity(Vec2::new(0., 0.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::steel())
            .particle_texture("steel_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: circle_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 100,
                particles_tall: 100,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(45., 35.))
            .particle_velocity(Vec2::new(0., 0.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::water())
            .particle_texture("liquid_particle.png".to_string())
            .build()
            .unwrap(),
    );

    // wood box filled with water on right
    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: hollow_box_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 100,
                particles_tall: 100,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(165., 35.))
            .particle_velocity(Vec2::new(0., 0.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::wood())
            .particle_texture("wood_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: circle_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 100,
                particles_tall: 100,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(165., 35.))
            .particle_velocity(Vec2::new(0., 0.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::water())
            .particle_texture("liquid_particle.png".to_string())
            .build()
            .unwrap(),
    );

    // drop down a cool structure
    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(10., 160.), Vec2::new(0., 80.)),
                particles_wide: 170,
                particles_tall: 110,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(95., 155.))
            .particle_velocity(Vec2::new(-5., -5.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::water())
            .particle_texture("liquid_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(10., 160.), Vec2::new(0., 80.)),
                particles_wide: 170,
                particles_tall: 110,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(95. + PI, 155. + PI))
            .particle_velocity(Vec2::new(-5., -5.))
            .particle_velocity_random_vec_a(Vec2::new(-10., 10.))
            .particle_velocity_random_vec_b(Vec2::new(-20., 00.))
            .particle_type(SpawnedParticleType::wood())
            .particle_texture("wood_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s
}

pub(crate) fn waterfall_scene() -> ParticleScene {
    let mut s = ParticleScene::new(String::from("waterfall"), DEFAULT_GRAVITY, DEFAULT_DT);

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: hollow_box_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 100,
                particles_tall: 100,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(85., 35.))
            .particle_velocity(Vec2::new(0., 0.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::steel())
            .particle_texture("steel_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: circle_20,
                domain: Mat2::from_cols(Vec2::new(-50., 50.), Vec2::new(-50., 50.)),
                particles_wide: 100,
                particles_tall: 100,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(85., 35.))
            .particle_velocity(Vec2::new(0., 0.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::water())
            .particle_texture("liquid_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::Rectangle { w: 25, h: 25 })
            .spawn_on_creation(true)
            .spawn_frequency(300)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(35., 185.))
            .particle_velocity(Vec2::new(10., -47.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::water())
            .particle_texture("liquid_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(-25., 25.), Vec2::new(-25., 25.)),
                particles_wide: 25,
                particles_tall: 25,
            })
            .spawn_on_creation(true)
            .spawn_frequency(425)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(15., 115.))
            .particle_velocity(Vec2::new(40., 17.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::water())
            .particle_texture("liquid_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(-25., 25.), Vec2::new(-25., 25.)),
                particles_wide: 25,
                particles_tall: 25,
            })
            .spawn_on_creation(true)
            .spawn_frequency(800)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(125., 115.))
            .particle_velocity(Vec2::new(-20., -37.))
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::water())
            .particle_texture("liquid_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::Triangle { l: 25 })
            .spawn_on_creation(false)
            .spawn_frequency(1800)
            .max_particles(200000)
            .particle_duration(40000)
            .particle_origin(Vec2::new(85., 215.))
            .particle_velocity(Vec2::new(1., -125.3))
            .particle_velocity_random_vec_a(Vec2::new(-0.0, -0.0))
            .particle_velocity_random_vec_b(Vec2::new(0.0, 0.0))
            .particle_type(SpawnedParticleType::steel())
            .particle_texture("steel_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(25., 40.), Vec2::new(0., 80.)),
                particles_wide: 110,
                particles_tall: 110,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(15., 5.))
            .particle_velocity(Vec2::ZERO)
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::steel())
            .particle_texture("steel_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(25., 40.), Vec2::new(0., 80.)),
                particles_wide: 110,
                particles_tall: 110,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(15. + PI, 5. + PI))
            .particle_velocity(Vec2::ZERO)
            .particle_velocity_random_vec_a(Vec2::new(10., 10.))
            .particle_velocity_random_vec_b(Vec2::new(20., 00.))
            .particle_type(SpawnedParticleType::wood())
            .particle_texture("wood_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(10., 40.), Vec2::new(0., 80.)),
                particles_wide: 90,
                particles_tall: 90,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(155., 5.))
            .particle_velocity(Vec2::ZERO)
            .particle_velocity_random_vec_a(Vec2::ZERO)
            .particle_velocity_random_vec_b(Vec2::ZERO)
            .particle_type(SpawnedParticleType::steel())
            .particle_texture("steel_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s.add_spawner(
        ParticleSpawnerInfoBuilder::default()
            .created_at(0)
            .pattern(SpawnerPattern::FuncXY {
                f: sinxy,
                domain: Mat2::from_cols(Vec2::new(10., 40.), Vec2::new(0., 80.)),
                particles_wide: 90,
                particles_tall: 90,
            })
            .spawn_on_creation(true)
            .spawn_frequency(0)
            .max_particles(75000)
            .particle_duration(100000)
            .particle_origin(Vec2::new(155. + PI, 5. + PI))
            .particle_velocity(Vec2::ZERO)
            .particle_velocity_random_vec_a(Vec2::new(-10., 10.))
            .particle_velocity_random_vec_b(Vec2::new(-20., 00.))
            .particle_type(SpawnedParticleType::wood())
            .particle_texture("wood_particle.png".to_string())
            .build()
            .unwrap(),
    );

    s
}
