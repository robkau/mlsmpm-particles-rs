#[allow(dead_code)]
pub(super) fn sinx(x: f32, _: f32) -> bool {
    return x.sin() > 0.;
}

#[allow(dead_code)]
pub(super) fn siny(_: f32, y: f32) -> bool {
    y.sin() > 0.
}

#[allow(dead_code)]
pub(super) fn sinxy(x: f32, y: f32) -> bool {
    x.sin() - y.sin() > 0.
}

// todo partial application in rust???
#[allow(dead_code)]
pub(super) fn circle_20(x: f32, y: f32) -> bool {
    let radius: f32 = 20.;
    (x.powi(2) + y.powi(2)).abs() - radius.powi(2) < 0.
}

#[allow(dead_code)]
pub(super) fn hollow_box_20(x: f32, y: f32) -> bool {
    let hole_radius = 20.;
    x.powi(2) + y.powi(2) > f32::powi(hole_radius, 2)
}
