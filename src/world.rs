use super::defaults::*;

#[derive(Copy, Clone)]
pub(super) struct WorldState {
    pub(super) dt: f32,
    pub(super) gravity: f32,
    pub(super) gravity_enabled: bool,
    pub(super) current_tick: usize,
}

impl WorldState {
    pub(super) fn toggle_gravity(&mut self) {
        self.gravity_enabled = !self.gravity_enabled;
    }

    pub(super) fn default() -> WorldState {
        WorldState {
            dt: DEFAULT_DT,
            gravity: DEFAULT_GRAVITY,
            gravity_enabled: true,
            current_tick: 0,
        }
    }

    pub(super) fn update(&mut self) {
        self.current_tick += 1;
    }
}
