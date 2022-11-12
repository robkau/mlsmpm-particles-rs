use crate::shapes::sinxy;
use crate::world::WorldState;
use crate::{
    ParticleSpawnerInfo, ParticleSpawnerInfoBuilder, ParticleSpawnerTag, SpawnerPattern,
    DEFAULT_DT, DEFAULT_GRAVITY,
};
use bevy::math::{Mat2, Vec2};
use bevy::prelude::*;

// Tags particle entities
#[derive(Component)]
pub(super) struct ParticleTag;

// XY position
#[derive(Component, Debug)]
pub(super) struct Position(pub(super) Vec2);

// XY velocity
#[derive(Component, Debug)]
pub(super) struct Velocity(pub(super) Vec2);

// mass
#[derive(Component)]
pub(super) struct Mass(pub(super) f32);

// 2x2 affine momentum matrix
#[derive(Component)]
pub(super) struct AffineMomentum(pub(super) Mat2);

// fluid constitutive model properties
#[derive(Clone, Copy, Component)]
pub(super) struct NewtonianFluidModel {
    pub(super) rest_density: f32,
    pub(super) dynamic_viscosity: f32,
    pub(super) eos_stiffness: f32,
    pub(super) eos_power: f32,
}

impl ConstitutiveModel for NewtonianFluidModel {
    fn new_particle(
        self,
        commands: &mut Commands,
        texture: Handle<Image>,
        at: Vec2,
        mass: f32,
        created_at: usize,
        vel: Option<Vec2>,
        max_age: Option<usize>,
    ) {
        commands
            .spawn_bundle(SpriteBundle {
                texture,
                transform: Transform::from_scale(Vec3::splat(0.002)),
                ..Default::default()
            })
            .insert_bundle((
                Position(at),
                self,
                Mass(mass),
                Velocity(vel.unwrap_or(Vec2::ZERO)),
                MaxAge(max_age.unwrap_or(5000)),
                AffineMomentum(Mat2::ZERO),
                CreatedAt(created_at),
                CellMassMomentumContributions([GridMassAndMomentumChange(0, 0., Vec2::ZERO); 9]),
                ParticleTag,
            ));
    }
}

// solid constitutive model properties
#[derive(Clone, Copy, Component)]
pub(super) struct NeoHookeanHyperElasticModel {
    pub(super) deformation_gradient: Mat2,
    pub(super) elastic_lambda: f32,
    // youngs modulus
    pub(super) elastic_mu: f32, // shear modulus
}

impl ConstitutiveModel for NeoHookeanHyperElasticModel {
    fn new_particle(
        self,
        commands: &mut Commands,
        texture: Handle<Image>,
        at: Vec2,
        mass: f32,
        created_at: usize,
        vel: Option<Vec2>,
        max_age: Option<usize>,
    ) {
        commands
            .spawn_bundle(SpriteBundle {
                texture,
                transform: Transform::from_scale(Vec3::splat(0.005)),
                ..Default::default()
            })
            .insert_bundle((
                Position(at),
                self,
                Mass(mass),
                Velocity(vel.unwrap_or(Vec2::ZERO)),
                MaxAge(max_age.unwrap_or(5000)),
                AffineMomentum(Mat2::ZERO),
                CreatedAt(created_at),
                CellMassMomentumContributions([GridMassAndMomentumChange(0, 0., Vec2::ZERO); 9]),
                ParticleTag,
            ));
    }
}

pub(super) fn steel_properties() -> NeoHookeanHyperElasticModel {
    NeoHookeanHyperElasticModel {
        deformation_gradient: Default::default(),
        elastic_lambda: 180. * 1000.,
        elastic_mu: 78. * 1000.,
    }
}

pub(super) fn wood_properties() -> NeoHookeanHyperElasticModel {
    NeoHookeanHyperElasticModel {
        deformation_gradient: Default::default(),
        elastic_lambda: 18. * 1000.,
        elastic_mu: 6. * 1000.,
    }
}

pub(super) fn water_properties() -> NewtonianFluidModel {
    NewtonianFluidModel {
        rest_density: 4.,
        dynamic_viscosity: 0.1,
        eos_stiffness: 100.,
        eos_power: 4.,
    }
}

// computed changes to-be-applied to grid on next steps
#[derive(Component)]
pub(super) struct CellMassMomentumContributions(pub(super) [GridMassAndMomentumChange; 9]);

#[derive(Clone, Copy)]
pub(super) struct GridMassAndMomentumChange(pub(super) usize, pub(super) f32, pub(super) Vec2);

// tick the entity was created on
#[derive(Component)]
pub(super) struct CreatedAt(pub(super) usize);

// entity deleted after this many ticks
#[derive(Component)]
pub(super) struct MaxAge(pub(super) usize);

pub trait ConstitutiveModel {
    fn new_particle(
        self,
        commands: &mut Commands,
        texture: Handle<Image>,
        at: Vec2,
        mass: f32,
        created_at: usize,
        vel: Option<Vec2>,
        max_age: Option<usize>,
    );
}

#[derive(Clone, Component)]
pub(super) struct SceneManager {
    scenes: Vec<Scene>,
    current: usize,
}

impl SceneManager {
    pub(super) fn default() -> SceneManager {
        SceneManager {
            scenes: vec![Scene::default()],
            current: 0,
        }
    }

    pub(super) fn get_current_scene(self) -> Scene {
        self.scenes.get(self.current).unwrap().clone()
    }
    pub(super) fn get_scene(self, i: usize) -> Scene {
        self.scenes.get(i).unwrap().clone()
    }

    pub(super) fn scenes(self) -> Vec<Scene> {
        self.scenes
    }

    pub(super) fn add_scene(&mut self, s: Scene) {
        self.scenes.push(s);
    }

    pub(super) fn get_current_scene_index(self) -> usize {
        self.current
    }

    pub(super) fn set_current_scene_index(&mut self, i: usize) {
        self.current = i;
    }

    pub(super) fn len(self) -> usize {
        self.scenes.len()
    }
}

#[derive(Clone, Component, Debug, PartialEq)]
pub(super) struct Scene {
    name: String,
    spawners: Vec<ParticleSpawnerInfo>,
    gravity: f32,
    dt: f32,
}

impl Scene {
    pub(super) fn default() -> Scene {
        Scene {
            name: "default scene".parse().unwrap(),
            spawners: vec![ParticleSpawnerInfoBuilder::default()
                .created_at(0)
                .pattern(SpawnerPattern::FuncXY {
                    f: sinxy,
                    domain: Mat2::from_cols(Vec2::new(0., 50.), Vec2::new(0., 50.)),
                    particles_wide: 50,
                    particles_tall: 50,
                })
                .spawn_frequency(1000) // todo special value to spawn once only)
                .max_particles(500000)
                .particle_duration(20000)
                .particle_origin(Vec2::new(20., 20.))
                .particle_velocity(Vec2::new(40., 100.))
                .particle_velocity_random_vec_a(Default::default())
                .particle_velocity_random_vec_b(Default::default())
                .particle_mass(1.0)
                .build()
                .unwrap()],
            gravity: DEFAULT_GRAVITY,
            dt: DEFAULT_DT,
        }
    }

    pub(super) fn new(name: String, gravity: f32, dt: f32) -> Scene {
        Scene {
            name,
            spawners: vec![],
            gravity,
            dt,
        }
    }

    pub(super) fn add_spawner(&mut self, ps: ParticleSpawnerInfo) {
        self.spawners.push(ps);
    }

    pub(super) fn name(self) -> String {
        self.name
    }

    pub(super) fn actualize(
        self,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut world: WorldState,
    ) {
        world.gravity = self.gravity;
        world.dt = self.dt;

        for spawner in self.spawners.into_iter() {
            commands.spawn_bundle((
                spawner,
                steel_properties(), // todo the actual properties stored with each spawner.
                asset_server.load::<Image, &str>("steel_particle.png"),
                ParticleSpawnerTag,
            ));
        }
    }
}
