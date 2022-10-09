use bevy::math::Vec2;
use bevy::prelude::{
    AssetServer, Commands, Entity, Input, KeyCode, Local, MouseButton, Query, Res, ResMut, Windows,
    With,
};
use bevy_egui::{egui, EguiContext, EguiSettings};
use std::thread::spawn;

use crate::{grid, spawn_particles, ParticleSpawnerInfo, ParticleSpawnerTag, SpawnerPattern};

use super::components::*;
use super::defaults::*;
use super::world::*;

#[derive(Default)]
pub(super) struct ClickAndDragState {
    dragging: bool,
    source_pos: Vec2,
    current_pos: Vec2,
}

pub(super) fn handle_inputs(
    mut commands: Commands,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    btn: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut egui_context: ResMut<EguiContext>,
    mut egui_settings: ResMut<EguiSettings>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut world: ResMut<WorldState>,
    mut spawner_drag: Local<ClickAndDragState>,
    mut particles: Query<(Entity, &Position, &mut Velocity, &Mass), With<ParticleTag>>,
    grid: Res<grid::Grid>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(win_pos) = window.cursor_position() {
        // cursor is inside the window.
        // translate window position to grid position
        let size = Vec2::new(window.width() as f32, window.height() as f32);
        let scale = f32::min(size.x, size.y) / grid.width as f32;
        let grid_pos = win_pos / scale;

        // if particle is near cursor, push it away.
        particles.par_for_each_mut(PAR_BATCH_SIZE, |(_, position, mut velocity, _)| {
            let dist = Vec2::new(position.0.x - grid_pos.x, position.0.y - grid_pos.y);

            let mouse_radius = 6.;

            if dist.dot(dist) < mouse_radius * mouse_radius {
                let norm_factor = dist.length() / mouse_radius;
                let force = dist.normalize() * (norm_factor / 2.);
                velocity.0 += force;
            }
        });

        // can left click and drag to spawn new solid steel arrowhead with velocity.
        if btn.just_pressed(MouseButton::Left) && !spawner_drag.dragging {
            // start dragging
            spawner_drag.dragging = true;
            spawner_drag.source_pos = grid_pos;
        } else if btn.just_released(MouseButton::Left) && spawner_drag.dragging {
            // end dragging
            spawner_drag.dragging = false;

            // and spawn some particles with velocity based on drag distance
            let si = &ParticleSpawnerInfo {
                created_at: world.current_tick,
                pattern: SpawnerPattern::TriangleRight,
                spawn_frequency: 999999999, // todo special value to spawn once only.
                max_particles: 500000,
                particle_duration: 20000,
                particle_origin: spawner_drag.source_pos,
                particle_velocity: grid_pos - spawner_drag.source_pos,
                particle_velocity_random_vec_a: Default::default(),
                particle_velocity_random_vec_b: Default::default(),
                particle_mass: 1.0,
            };

            spawn_particles(
                si,
                steel_properties(),
                &mut commands,
                asset_server.load("steel_particle.png"),
                &world,
            );
        }

        // can right click to dump a bucket of water.
        if btn.just_pressed(MouseButton::Right) {
            let si = &ParticleSpawnerInfo {
                created_at: world.current_tick,
                pattern: SpawnerPattern::Cube,
                spawn_frequency: 999999999, // todo special value to spawn once only.
                max_particles: 500000,
                particle_duration: 20000,
                particle_origin: grid_pos,
                particle_velocity: Vec2::new(0., -40.),
                particle_velocity_random_vec_a: Default::default(),
                particle_velocity_random_vec_b: Default::default(),
                particle_mass: 0.75,
            };

            // todo value to set size/width of spawned objects
            spawn_particles(
                si,
                water_properties(),
                &mut commands,
                asset_server.load("liquid_particle.png"),
                &world,
            );
        }

        egui::Window::new("Controls").show(egui_context.ctx_mut(), |ui| {
            if ui.button("(R)eset").clicked() || keys.just_pressed(KeyCode::R) {
                particles.for_each(|(id, _, _, _)| {
                    commands.entity(id).despawn();
                });
                world.dt = DEFAULT_DT;
                world.gravity = DEFAULT_GRAVITY;
                return;
            };
            if ui.button("(G)ravity toggle").clicked() || keys.just_pressed(KeyCode::G) {
                world.toggle_gravity();
                return;
            };

            // slider for gravity
            ui.add(egui::Slider::new(&mut world.gravity, -10.0..=10.).text("gravity"));

            // slider for DT.
            ui.add(egui::Slider::new(&mut world.dt, 0.0001..=0.01).text("dt"));

            // toggle hiDPI with '/'
            if keys.just_pressed(KeyCode::Slash) || toggle_scale_factor.is_none() {
                *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(true));

                if let Some(window) = windows.get_primary() {
                    let scale_factor = if toggle_scale_factor.unwrap() {
                        1.0 / window.scale_factor()
                    } else {
                        1.0
                    };
                    egui_settings.scale_factor = scale_factor;
                }
            }

            // todo:
            // one spawner can be selected (or new spawner to-create can be selected)
            // click and drag when placing to set particle velocity
            // the selected spawner shows its elements on left
        });
    };
}
