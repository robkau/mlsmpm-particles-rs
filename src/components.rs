use bevy::math::{Mat2, Vec2};
use bevy::prelude::*;

// Tags particle entities
#[derive(Component)]
pub(super) struct ParticleTag;

// todo move rendering to GPU shader. over 70% of traced CPU time is inside sprite stuff.

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
        mut commands: &mut Commands,
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
                transform: Transform::from_scale(Vec3::splat(0.002 * mass)),
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
        mut commands: &mut Commands,
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
                transform: Transform::from_scale(Vec3::splat(0.005 * mass)),
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
        texture: &Handle<Image>,
        at: Vec2,
        mass: f32,
        created_at: usize,
        vel: Option<Vec2>,
        max_age: Option<usize>,
    );
}
