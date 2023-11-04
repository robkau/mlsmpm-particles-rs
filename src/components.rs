use crate::prelude::*;

// Tags particle entities
#[derive(Component)]
pub(crate) struct ParticleTag;

// XY position
#[derive(Component, Debug)]
pub(crate) struct Position(pub(crate) Vec2);

// XY velocity
#[derive(Component, Debug)]
pub(crate) struct Velocity(pub(crate) Vec2);

// mass
#[derive(Component)]
pub(crate) struct Mass(pub(crate) f32);

// 2x2 affine momentum matrix
#[derive(Component)]
pub(crate) struct AffineMomentum(pub(crate) Mat2);

// fluid constitutive model properties
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct NewtonianFluidModel {
    pub(crate) rest_density: f32,
    pub(crate) dynamic_viscosity: f32,
    pub(crate) eos_stiffness: f32,
    pub(crate) eos_power: f32,
}

// solid constitutive model properties
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct NeoHookeanHyperElasticModel {
    pub(crate) deformation_gradient: Mat2,
    pub(crate) elastic_lambda: f32, // youngs modulus
    pub(crate) elastic_mu: f32,     // shear modulus
}

pub(crate) fn steel_properties() -> NeoHookeanHyperElasticModel {
    NeoHookeanHyperElasticModel {
        deformation_gradient: Default::default(),
        elastic_lambda: 180. * 1000.,
        elastic_mu: 78. * 1000.,
    }
}

pub(crate) fn wood_properties() -> NeoHookeanHyperElasticModel {
    NeoHookeanHyperElasticModel {
        deformation_gradient: Default::default(),
        elastic_lambda: 18. * 1000.,
        elastic_mu: 6. * 1000.,
    }
}

pub(crate) fn water_properties() -> NewtonianFluidModel {
    NewtonianFluidModel {
        rest_density: 4.,
        dynamic_viscosity: 0.1,
        eos_stiffness: 100.,
        eos_power: 4.,
    }
}

// computed changes to-be-applied to grid on next steps
#[derive(Component)]
pub(crate) struct CellMassMomentumContributions(pub(crate) [GridMassAndMomentumChange; 9]);

#[derive(Clone, Copy)]
pub(crate) struct GridMassAndMomentumChange(pub(crate) usize, pub(crate) f32, pub(crate) Vec2);

// tick the entity was created on
#[derive(Component)]
pub(crate) struct CreatedAt(pub(crate) usize);

// entity deleted after this many ticks
#[derive(Component)]
pub(crate) struct MaxAge(pub(crate) usize);

#[derive(Clone, Resource, Debug, PartialEq)]
pub(crate) struct ParticleScene {
    name: String,
    spawners: Vec<ParticleSpawnerInfo>,
    gravity: f32,
    dt: f32,
}

impl ParticleScene {
    pub(crate) fn default() -> ParticleScene {
        waterfall_scene()
    }

    pub(crate) fn new(name: String, gravity: f32, dt: f32) -> ParticleScene {
        ParticleScene {
            name,
            spawners: vec![],
            gravity,
            dt,
        }
    }

    pub(crate) fn add_spawner(&mut self, ps: ParticleSpawnerInfo) {
        self.spawners.push(ps);
    }

    pub(crate) fn name(self) -> String {
        self.name
    }

    pub(crate) fn actualize(
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
            commands.spawn((
                s.clone(),
                asset_server.load::<Image>(&s.clone().particle_texture),
                ParticleSpawnerTag,
            ));
        }
    }
}
