pub(super) fn sinxy(x: f32, y: f32) -> bool {
    return x.sin() - y.sin() > 0.;
}

// todo partial application in rust???
pub(super) fn circle_20(x: f32, y: f32) -> bool {
    let radius: f32 = 20.;
    return (x.powi(2) + y.powi(2)).abs() - radius.powi(2) < 0.;
}

pub(super) fn hollow_box_20(x: f32, y: f32) -> bool {
    let hole_radius = 20.;
    return x.powi(2) + y.powi(2) > f32::powi(hole_radius, 2);
}
