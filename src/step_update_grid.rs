use bevy::prelude::*;

use crate::components::*;
use crate::grid::Grid;
use crate::world::*;

pub(super) fn update_grid(
    mut grid: ResMut<Grid>,
    mut world: ResMut<WorldState>,
    particles: Query<(&CellMassMomentumContributions, ), With<ParticleTag>>,
) {
    particles.for_each(|mmc| {
        for change in mmc.0.0.iter() {
            grid.cells[change.0].velocity += change.2;
        }
    });

    world.update();
    grid.update(
        world.dt,
        if world.gravity_enabled {
            world.gravity
        } else {
            0.
        },
    );
}
