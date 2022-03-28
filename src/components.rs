use bevy::math::{Mat2, Vec2};
use bevy::prelude::Component;

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
#[derive(Clone, Component)]
// todo these fields and this struct doesnt need to be public?
pub(super) struct ConstitutiveModelFluid {
    pub(super) rest_density: f32,
    pub(super) dynamic_viscosity: f32,
    pub(super) eos_stiffness: f32,
    pub(super) eos_power: f32,
}

// solid constitutive model properties
#[derive(Clone, Component)]
pub(super) struct ConstitutiveModelNeoHookeanHyperElastic {
    pub(super) deformation_gradient: Mat2,
    pub(super) elastic_lambda: f32,
    // youngs modulus
    pub(super) elastic_mu: f32,     // shear modulus
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
