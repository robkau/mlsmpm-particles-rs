use crate::prelude::*;

#[derive(Copy, Clone, Resource)]
pub(crate) struct NeedToReset(pub(crate) bool);

#[derive(Copy, Clone, Resource)]
pub(crate) struct WorldState {
    pub(crate) dt: f32,
    pub(crate) gravity: f32,
    pub(crate) gravity_enabled: bool,
    pub(crate) current_tick: usize,
}

impl WorldState {
    pub(crate) fn toggle_gravity(&mut self) {
        self.gravity_enabled = !self.gravity_enabled;
    }

    pub(crate) fn new(dt: f32, gravity: f32, gravity_enabled: bool) -> WorldState {
        WorldState {
            dt,
            gravity,
            gravity_enabled,
            current_tick: 0,
        }
    }

    pub(crate) fn default() -> WorldState {
        WorldState {
            dt: DEFAULT_DT,
            gravity: DEFAULT_GRAVITY,
            gravity_enabled: true,
            current_tick: 0,
        }
    }

    pub(crate) fn update(&mut self) {
        self.current_tick += 1;
    }
}
