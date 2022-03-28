// not needed until resizeable.
//fn set_zoom_from_window_size(
//    camera: Query<OrthographicCameraBundle>,
//    resize_event: Res<Events<WindowResized>>,
//) {
//    let mut reader = resize_event.get_reader();
//    // todo calculate zoom and apply it to world.
//
//    // zoom == window_width / grid_width
//    let wnd = wnds.get_primary().unwrap();
//    let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
//    let scale = wnd.width() / grid.width as f32; // todo in response to events.
//
//    cb.transform = Transform::from_translation(Vec3::new(
//        size.x / (scale * 2.0),
//        size.y / (scale * 2.0),
//        0.0,
//    ));
//    cb.orthographic_projection.scale = 1.0 / scale;
//    commands.spawn_bundle(cb);
//}
