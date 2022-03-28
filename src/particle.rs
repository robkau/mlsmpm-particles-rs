use bevy::math::Mat2;
use bevy::prelude::*;

use crate::components::*;

pub(super) fn new_solid_particle(
    commands: &mut Commands,
    asset_server: &AssetServer,
    tick: usize,
    at: Vec2,
    pp: ConstitutiveModelNeoHookeanHyperElastic,
    mass: f32,
    vel: Option<Vec2>,
    max_age: Option<usize>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("solid_particle.png"),
            transform: Transform::from_scale(Vec3::splat(0.005)), // todo scale me from mass.
            ..Default::default()
        })
        .insert_bundle((
            Position(at),
            pp,
            Mass(mass),
            Velocity(vel.unwrap_or(Vec2::ZERO)),
            MaxAge(max_age.unwrap_or(5000)),
            AffineMomentum(Mat2::ZERO),
            CreatedAt(tick),
            CellMassMomentumContributions([GridMassAndMomentumChange(0, 0., Vec2::ZERO); 9]),
            ParticleTag,
        ));
}

pub(super) fn new_fluid_particle(
    commands: &mut Commands,
    asset_server: &AssetServer,
    tick: usize,
    at: Vec2,
    pp: ConstitutiveModelFluid,
    mass: f32,
    vel: Option<Vec2>,
    max_age: Option<usize>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("liquid_particle.png"),
            transform: Transform::from_scale(Vec3::splat(0.002)), // todo scale me from mass.
            ..Default::default()
        })
        .insert_bundle((
            Position(at),
            pp,
            Mass(mass),
            Velocity(vel.unwrap_or(Vec2::ZERO)),
            MaxAge(max_age.unwrap_or(5000)),
            AffineMomentum(Mat2::ZERO),
            CreatedAt(tick),
            CellMassMomentumContributions([GridMassAndMomentumChange(0, 0., Vec2::ZERO); 9]),
            ParticleTag,
        ));
}
