#![allow(clippy::too_many_arguments)]

mod components;
mod defaults;
mod expire_old;
mod grid;
mod inputs;
mod particle_sprites;
mod scene;
mod setup_camera;
mod shapes;
mod spawners;
mod step_g2p;
mod step_p2g;
mod step_update_cells;
mod step_update_deformations;
mod step_update_grid;
mod world;

#[cfg(test)]
mod test;

mod prelude {
    pub(crate) use bevy::diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
    };
    pub(crate) use bevy::math::{Mat2, Vec2};
    pub(crate) use bevy::prelude::*;
    pub(crate) use bevy::window::{PrimaryWindow, Window};
    pub(crate) use bevy_egui::egui;
    pub(crate) use bevy_egui::*;

    pub(crate) use crate::components::*;
    pub(crate) use crate::defaults::*;
    pub(crate) use crate::grid::*;
    pub(crate) use crate::inputs::*;
    pub(crate) use crate::scene::*;
    pub(crate) use crate::setup_camera::*;
    pub(crate) use crate::shapes::*;
    pub(crate) use crate::spawners::*;
    pub(crate) use crate::world::*;
}

use prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum Sets {
    Input,
    P2g,
    G2p,
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(grid::Grid::new(DEFAULT_GRID_WIDTH))
        .insert_resource(world::WorldState::default())
        .insert_resource(ParticleScene::default())
        .insert_resource(NeedToReset(false))
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(EguiPlugin)
        .add_plugins(EntityCountDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            (
                bevy::window::close_on_esc,
                on_window_resize,
                handle_inputs,
                tick_spawners,
                reset_grid,
                step_update_cells::update_cells,
                step_update_cells::apply_update_cell_computations,
            )
                .chain()
                .in_set(Sets::Input),
        )
        .add_systems(
            Update,
            (
                step_p2g::particles_to_grid_fluids,
                step_p2g::particles_to_grid_solids,
            )
                .chain()
                .in_set(Sets::P2g),
        )
        .add_systems(
            Update,
            (
                step_update_grid::update_grid,
                step_g2p::grid_to_particles,
                step_update_deformations::update_deformation_gradients,
                expire_old::delete_old_entities,
                particle_sprites::update_sprites,
                update_scene,
            )
                .chain()
                .in_set(Sets::G2p),
        )
        .configure_sets(Update, Sets::Input.before(Sets::P2g))
        .configure_sets(Update, Sets::P2g.before(Sets::G2p))
        .run()
}

// todo render to (animated) image output
// https://github.com/bevyengine/bevy/issues/1207
//https://github.com/rmsc/bevy/blob/render_to_file/examples/3d/render_to_file.rs
