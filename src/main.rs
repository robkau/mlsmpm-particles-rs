use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy::window::WindowMode::BorderlessFullscreen;
use bevy_egui::EguiPlugin;

use camera::*;
use spawners::*;

use crate::defaults::*;

mod camera;
mod components;
mod defaults;
mod expire_old;
mod grid;
mod inputs;
mod particle;
mod particle_sprites;
mod spawners;
mod step_g2p;
mod step_p2g;
mod step_update_cells;
mod step_update_deformations;
mod step_update_grid;
mod world;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "mlsmpm-particles-rs".to_string(),
            //width: DEFAULT_WINDOW_WIDTH,
            //height: DEFAULT_WINDOW_HEIGHT, // todo mouse cursor not right when fullscrene.
            mode: BorderlessFullscreen,
            ..Default::default()
        })
        .insert_resource(grid::Grid::new(DEFAULT_GRID_WIDTH))
        .insert_resource(world::WorldState::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy_framepace::FramepacePlugin {
            enabled: true,
            framerate_limit: bevy_framepace::FramerateLimit::Auto,
            warn_on_frame_drop: false,
            safety_margin: std::time::Duration::from_micros(100),
            power_saver: bevy_framepace::PowerSaver::Disabled,
        })
        .add_startup_system(setup_camera)
        .add_startup_system(create_initial_spawners)
        //.add_system(
        //    set_zoom_from_window_size
        //        .label("set_zoom_from_window_size")
        //        .before("handle_inputs"),
        //)
        .add_system(
            inputs::handle_inputs
                .label("handle_inputs")
                .before("tick_spawners"),
        )
        .add_system(
            spawners::tick_spawners
                .label("tick_spawners")
                .before("apply_cursor_effects"),
        )
        .add_system(
            inputs::apply_cursor_effects
                .label("apply_cursor_effects")
                .before("reset_grid"),
        )
        .add_system(grid::reset_grid.label("reset_grid").before("update_cells"))
        .add_system(
            step_update_cells::update_cells
                .label("update_cells")
                .before("apply_update_cell_computations"),
        )
        .add_system(
            step_update_cells::apply_update_cell_computations
                .label("apply_update_cell_computations")
                .before("p2g_f")
                .before("p2g_s"),
        )
        .add_system(
            step_p2g::particles_to_grid_fluids
                .label("p2g_f")
                .before("update_grid"),
        )
        .add_system(
            step_p2g::particles_to_grid_solids
                .label("p2g_s")
                .before("update_grid"),
        )
        .add_system(
            step_update_grid::update_grid
                .label("update_grid")
                .before("g2p"),
        )
        .add_system(
            step_g2p::grid_to_particles
                .label("g2p")
                .before("update_deformation_gradients"),
        )
        .add_system(
            step_update_deformations::update_deformation_gradients
                .label("update_deformation_gradients")
                .before("delete_old_entities"),
        )
        .add_system(
            expire_old::delete_old_entities
                .label("delete_old_entities")
                .before("update_sprites"),
        )
        .add_system(particle_sprites::update_sprites.label("update_sprites"))
        .run();
}

// todo render to (animated) image output
// https://github.com/bevyengine/bevy/issues/1207
//https://github.com/rmsc/bevy/blob/render_to_file/examples/3d/render_to_file.rs
