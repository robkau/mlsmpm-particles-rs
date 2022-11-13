use crate::scene::{hollow_box_scene, waterfall_scene};
use crate::shapes::sinxy;
use crate::world::WorldState;
use crate::SpawnedParticleType::Steel;
use crate::{
    ParticleSpawnerInfo, ParticleSpawnerInfoBuilder, ParticleSpawnerTag, SpawnedParticleType,
    SpawnerPattern, DEFAULT_DT, DEFAULT_GRAVITY,
};
use bevy::ecs::system::Spawn;
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
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(super) struct NewtonianFluidModel {
    pub(super) rest_density: f32,
    pub(super) dynamic_viscosity: f32,
    pub(super) eos_stiffness: f32,
    pub(super) eos_power: f32,
}

// solid constitutive model properties
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(super) struct NeoHookeanHyperElasticModel {
    pub(super) deformation_gradient: Mat2,
    pub(super) elastic_lambda: f32, // youngs modulus
    pub(super) elastic_mu: f32,     // shear modulus
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

#[derive(Clone, Component, Debug, PartialEq)]
pub(super) struct Scene {
    name: String,
    spawners: Vec<ParticleSpawnerInfo>,
    gravity: f32,
    dt: f32,
}

impl Scene {
    pub(super) fn default() -> Scene {
        waterfall_scene()
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
        commands: &mut Commands,
        world: &mut ResMut<WorldState>,
        asset_server: &Res<AssetServer>,
    ) {
        world.gravity = self.gravity;
        world.dt = self.dt;
        world.current_tick = 0;

        for spawner in self.spawners.into_iter() {
            let mut s = spawner.clone();

            s.created_at = 0;
            commands.spawn_bundle((
                s.clone(),
                asset_server.load::<Image, &std::string::String>(&s.clone().particle_texture),
                ParticleSpawnerTag,
            ));
        }
    }
}
