use crate::prelude::*;
use bevy::window::WindowResized;

pub(crate) fn setup_camera(
    mut commands: Commands,
    grid: Res<Grid>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let mut cb = Camera2dBundle::default();
    let wnd = primary_window.single();

    let (t, s) = transform_and_scale_from(wnd, grid);

    cb.transform = t;
    cb.projection.scale = s;
    commands.spawn(cb);
}

pub(crate) fn on_window_resize(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<(&mut Transform, &mut OrthographicProjection, &Camera2d)>,
    grid: Res<Grid>,
    mut resize_events: EventReader<WindowResized>,
) {
    let (mut transform, mut projection, _) = camera.single_mut();

    for _ in resize_events.iter() {
        let wnd = primary_window.single();

        let (t, s) = transform_and_scale_from(wnd, grid);
        *transform = t;
        projection.scale = s;

        return; // only need to match current size once
    }
}

fn transform_and_scale_from(wnd: &Window, grid: Res<Grid>) -> (Transform, f32) {
    let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
    let scale = f32::min(size.x, size.y) / grid.width as f32; // adjust this to scale

    let t = Transform::from_translation(Vec3::new(
        (size.x) / (scale * 2.0),
        (size.y) / (scale * 2.0),
        0.0,
    ));

    let s = 1.0 / scale;

    (t, s)
}
