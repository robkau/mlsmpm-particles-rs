use bevy::math::Vec2;
use bevy::prelude::{
    AssetServer, Commands, Entity, Image, Input, KeyCode, Local, MouseButton, Query, Res, ResMut,
    Windows, With,
};
use bevy_egui::{egui, EguiContext, EguiSettings};

use crate::components::Scene;
use crate::scene::hollow_box_scene;
use crate::{
    grid, ParticleSpawnerInfoBuilder, ParticleSpawnerTag, SpawnedParticleType, SpawnerPattern,
};

use super::components::*;
use super::defaults::*;
use super::world::*;

#[derive(Default)]
pub(super) struct ClickAndDragState {
    dragging: bool,
    source_pos: Vec2,
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
    mut current_scene: ResMut<Scene>,
    mut need_to_reset: ResMut<NeedToReset>,
    grid: Res<grid::Grid>,
    mut spawner_drag: Local<ClickAndDragState>,
    mut particles: Query<(Entity, &Position, &mut Velocity, &Mass), With<ParticleTag>>,
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
            let si = ParticleSpawnerInfoBuilder::default()
                .created_at(world.current_tick)
                .pattern(SpawnerPattern::Triangle { l: 30 })
                .spawn_on_creation(true)
                .spawn_frequency(0)
                .max_particles(500000)
                .particle_duration(20000)
                .particle_origin(spawner_drag.source_pos)
                .particle_velocity(grid_pos - spawner_drag.source_pos)
                .particle_velocity_random_vec_a(Default::default())
                .particle_velocity_random_vec_b(Default::default())
                .particle_type(SpawnedParticleType::steel())
                .particle_texture("steel_particle.png".to_string())
                .build()
                .unwrap();
            commands.spawn_bundle((
                si.clone(),
                asset_server.load::<Image, &std::string::String>(&si.clone().particle_texture),
                ParticleSpawnerTag,
            ));
        }

        // can right click to dump a bucket of water.
        if btn.just_pressed(MouseButton::Right) {
            let si = &ParticleSpawnerInfoBuilder::default()
                .created_at(world.current_tick)
                .pattern(SpawnerPattern::Rectangle { w: 30, h: 30 })
                .spawn_on_creation(true)
                .spawn_frequency(0)
                .max_particles(500000)
                .particle_duration(20000)
                .particle_origin(grid_pos)
                .particle_velocity(Vec2::new(0., -40.))
                .particle_velocity_random_vec_a(Default::default())
                .particle_velocity_random_vec_b(Default::default())
                .particle_type(SpawnedParticleType::water())
                .particle_texture("liquid_particle.png".to_string())
                .build()
                .unwrap();
            commands.spawn_bundle((
                si.clone(),
                asset_server.load::<Image, &std::string::String>(&si.clone().particle_texture),
                ParticleSpawnerTag,
            ));
        }

        egui::Window::new("Controls").show(egui_context.ctx_mut(), |ui| {
            if ui.button("(R)eset").clicked() || keys.just_pressed(KeyCode::R) {
                need_to_reset.0 = true;
                return;
            };
            if ui.button("(G)ravity toggle").clicked() || keys.just_pressed(KeyCode::G) {
                world.toggle_gravity();
                return;
            };

            egui::ComboBox::from_label(format!(
                "Currently selected enum: {}",
                current_scene.clone().name(),
            )) // When created from a label the text will b shown on the side of the combobox
            .selected_text(current_scene.clone().name()) // This is the currently selected option (in text form)
            .show_ui(ui, |ui| {
                // The first parameter is a mutable reference to allow the choice to be modified when the user selects
                // something else. The second parameter is the actual value of the option (to be compared with the currently)
                // selected one to allow egui to highlight the correct label. The third parameter is the string to show.
                ui.selectable_value(
                    &mut *current_scene,
                    Scene::default(),
                    Scene::default().name(),
                );
                let hbs = hollow_box_scene();
                let hbs_name = hbs.clone().name();
                ui.selectable_value(&mut *current_scene, hbs, hbs_name);
            });

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
