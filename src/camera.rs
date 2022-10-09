use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Camera2dBundle, Commands, Res, Transform, Windows};

use crate::grid::Grid;

pub(super) fn setup_camera(mut commands: Commands, grid: Res<Grid>, wnds: Res<Windows>) {
    let mut cb = Camera2dBundle::default();

    let wnd = wnds.get_primary().unwrap();
    let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

    let scale = f32::min(size.x, size.y) / grid.width as f32; // todo in response to events.

    cb.transform = Transform::from_translation(Vec3::new(
        size.x / (scale * 2.0),
        size.y / (scale * 2.0),
        0.0,
    ));
    cb.projection.scale = 1.0 / scale;
    commands.spawn_bundle(cb);
}
