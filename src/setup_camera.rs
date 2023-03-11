use crate::prelude::*;

pub(crate) fn setup_camera(
    mut commands: Commands,
    grid: Res<Grid>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let mut cb = Camera2dBundle::default();

    let wnd = primary_window.single();
    let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

    let scale = f32::min(size.x, size.y) / grid.width as f32; // adjust this to scale

    cb.transform = Transform::from_translation(Vec3::new(
        size.x / (scale * 2.0),
        size.y / (scale * 2.0),
        0.0,
    ));
    cb.projection.scale = 1.0 / scale;
    commands.spawn(cb);
}
