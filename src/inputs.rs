use bevy::math::Vec2;
use bevy::prelude::{
    Commands, Entity, Input, KeyCode, Local, MouseButton, Query, Res, ResMut, Windows, With,
};
use bevy::tasks::ComputeTaskPool;
use bevy_egui::{egui, EguiContext, EguiSettings};

use crate::grid;

use super::components::*;
use super::defaults::*;
use super::world::*;

pub(super) fn handle_inputs(
    mut commands: Commands,
    windows: Res<Windows>,
    keys: Res<Input<KeyCode>>,
    mut egui_context: ResMut<EguiContext>,
    mut egui_settings: ResMut<EguiSettings>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut world: ResMut<WorldState>,
    particles: Query<Entity, With<ParticleTag>>,
) {
    // todo place spawners and drag direction and click to configure

    egui::Window::new("Controls").show(egui_context.ctx_mut(), |ui| {
        if ui.button("(R)eset").clicked() || keys.just_pressed(KeyCode::R) {
            particles.for_each(|id| {
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
}

pub(super) fn apply_cursor_effects(
    pool: Res<ComputeTaskPool>,
    clicks: Res<Input<MouseButton>>, // has mouse clicks
    windows: Res<Windows>,           // has cursor position
    world: Res<WorldState>,
    grid: Res<grid::Grid>,
    mut particles: Query<(&Position, &mut Velocity, &Mass), With<ParticleTag>>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(win_pos) = window.cursor_position() {
        // cursor is inside the window.
        // translate window position to grid position
        let scale = window.width() / grid.width as f32;
        let grid_pos = win_pos / scale;
        // if particle is near cursor, push it away.
        particles.par_for_each_mut(&pool, PAR_BATCH_SIZE, |(position, mut velocity, mass)| {
            let dist = Vec2::new(position.0.x - grid_pos.x, position.0.y - grid_pos.y);

            let mouse_radius = 6.;

            if dist.dot(dist) < mouse_radius * mouse_radius {
                let norm_factor = dist.length() / mouse_radius;
                let force = dist.normalize() * (norm_factor / 2.);
                velocity.0 += force;
            }
        });
    }
}
