use super::components::*;
use super::grid::*;
use super::world::*;
use crate::shapes::*;

use bevy::prelude::*;
use derive_builder::Builder;
use rand::Rng;
use std::f32::consts::PI;

pub(super) const LIQUID_PARTICLE_MASS: f32 = 1.;
pub(super) const WOOD_PARTICLE_MASS: f32 = 1.;
pub(super) const STEEL_PARTICLE_MASS: f32 = 1.5;

// Tags particle spawner entities
#[derive(Component)]
pub(super) struct ParticleSpawnerTag;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(super) enum SpawnerPattern {
    SingleParticle,
    LineHorizontal {
        w: usize,
    },
    LineVertical {
        h: usize,
    },
    Rectangle {
        w: usize,
        h: usize,
    },
    Tower {
        w: usize,
        h: usize,
    },
    Triangle {
        l: usize,
    },
    FuncXY {
        f: fn(x: f32, y: f32) -> bool,
        domain: Mat2,
        particles_wide: usize,
        particles_tall: usize,
    }, // Spiral{rotationPerTick: f32, ticksPerSpawn: usize},
}

#[derive(Builder, Clone, Component, Debug, PartialEq)]
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

pub(super) fn tick_spawners(
    mut commands: Commands,
    world: Res<WorldState>,
    grid: Res<Grid>,
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
                &grid,
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
                &grid,
            );
        }
    });
}

fn spawn_particle(
    commands: &mut Commands,
    grid_width: usize,
    cm: impl ConstitutiveModel + Copy,
    spawner_info: &ParticleSpawnerInfo,
    spawn_offset: Vec2,
    vel: Option<Vec2>,
    texture: Handle<Image>,
    created_at: usize,
) {
    let particle_position = spawner_info.particle_origin + spawn_offset;

    let min = 3;
    let max = grid_width - 4;
    if particle_position.x <= min as f32 || particle_position.x >= max as f32 {
        return;
    }
    if particle_position.y <= min as f32 || particle_position.y >= max as f32 {
        return;
    }

    cm.new_particle(
        commands,
        texture.clone(),
        particle_position,
        spawner_info.particle_mass,
        created_at,
        vel,
        Some(spawner_info.particle_duration),
    );
}

pub(super) fn spawn_particles(
    spawner_info: &ParticleSpawnerInfo,
    cm: impl ConstitutiveModel + Copy,
    commands: &mut Commands,
    texture: Handle<Image>,
    world: &WorldState,
    grid: &Res<Grid>,
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
            spawn_particle(
                commands,
                grid.width,
                cm,
                spawner_info,
                Vec2::ZERO,
                Some(spawn_vel),
                texture.clone(),
                world.current_tick,
            );
        }
        SpawnerPattern::LineHorizontal { w } => {
            for x in 0..w {
                spawn_particle(
                    commands,
                    grid.width,
                    cm,
                    spawner_info,
                    Vec2::new(x as f32, 0.),
                    Some(spawn_vel),
                    texture.clone(),
                    world.current_tick,
                );
            }
        }
        SpawnerPattern::LineVertical { h } => {
            for y in 0..h {
                spawn_particle(
                    commands,
                    grid.width,
                    cm,
                    spawner_info,
                    Vec2::new(0., y as f32),
                    Some(spawn_vel),
                    texture.clone(),
                    world.current_tick,
                );
            }
        }
        SpawnerPattern::Rectangle { w, h } => {
            for x in 0..w {
                for y in 0..h {
                    spawn_particle(
                        commands,
                        grid.width,
                        cm,
                        spawner_info,
                        Vec2::new(x as f32 + 0.001, y as f32 + 0.001),
                        Some(spawn_vel),
                        texture.clone(),
                        world.current_tick,
                    );
                }
            }
        }
        SpawnerPattern::Tower { w, h } => {
            for x in 0..w {
                for y in 0..h {
                    spawn_particle(
                        commands,
                        grid.width,
                        cm,
                        spawner_info,
                        Vec2::new(x as f32, y as f32),
                        Some(spawn_vel),
                        texture.clone(),
                        world.current_tick,
                    );
                }
            }
        }
        SpawnerPattern::Triangle { l } => {
            let x_axis: Vec2 = Vec2::new(1., 0.);
            let angle = match spawn_vel.length() {
                0. => 0.,
                _ => x_axis.angle_between(spawn_vel),
            };

            for x in 0..l {
                for y in 0..x {
                    // offset y by 0.5 every other time
                    let mut ya: f32 = if x % 2 == 0 {
                        y as f32 - 0.25
                    } else {
                        y as f32 + 0.25
                    };
                    ya -= x as f32 / 2.;

                    // rotate by angle about triangle tip
                    let pivot: Vec2 = Vec2::new(l as f32, 0.);
                    let pos: Vec2 = Vec2::new(
                        (0.001 + pivot.x - x as f32) / 4.,
                        (0.001 + pivot.y + ya as f32) / 4.,
                    );

                    // 1. translate point back to origin
                    let pos_rel_origin: Vec2 = Vec2::new(pos.x - pivot.x, pos.y - pivot.y);

                    // 2. rotate point
                    let pos_rel_origin_rotated: Vec2 = Vec2::new(
                        pos_rel_origin.x * angle.cos() - pos_rel_origin.y * angle.sin(),
                        pos_rel_origin.x * angle.sin() + pos_rel_origin.y * angle.cos(),
                    );

                    // 3. translate point back:
                    let pos_rotated: Vec2 = pos_rel_origin_rotated + pivot;

                    spawn_particle(
                        commands,
                        grid.width,
                        cm,
                        spawner_info,
                        pos_rotated,
                        Some(spawn_vel),
                        texture.clone(),
                        world.current_tick,
                    );
                }
            }
        }
        SpawnerPattern::FuncXY {
            f,
            domain,
            particles_wide,
            particles_tall,
        } => {
            // todo fixme with circle case
            let units_per_particle = Vec2::new(
                domain.x_axis.length() / particles_wide as f32,
                domain.y_axis.length() / particles_tall as f32,
            );

            for x in 0..particles_wide {
                for y in 0..particles_tall {
                    let xp = x as f32 * units_per_particle.x - domain.x_axis.length() / 2.;
                    let yp = y as f32 * units_per_particle.y - domain.y_axis.length() / 2.;

                    if f(xp as f32, yp as f32) {
                        spawn_particle(
                            commands,
                            grid.width,
                            cm,
                            spawner_info,
                            Vec2::new(xp as f32, yp as f32),
                            Some(spawn_vel),
                            texture.clone(),
                            world.current_tick,
                        );
                    }
                }
            }
        } //SpawnerPattern::Spiral => {
          //
          //}
    }
}
