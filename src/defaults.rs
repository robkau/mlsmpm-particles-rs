pub(super) const DEFAULT_WINDOW_WIDTH: f32 = 1000.;
pub(super) const DEFAULT_WINDOW_HEIGHT: f32 = 1000.;
// todo proper scaling.

pub(super) const DEFAULT_GRID_WIDTH: usize = usize::pow(2, 8);

// todo use me when spawning.
pub(super) const PARTICLE_ACROSS_GRID: usize = 1024;
pub(super) const PARTICLES_ACROSS_CELL: usize = PARTICLE_ACROSS_GRID / DEFAULT_GRID_WIDTH;

pub(super) const DEFAULT_DT: f32 = 0.0010;
pub(super) const DEFAULT_GRAVITY: f32 = -1.;

pub(super) const PAR_BATCH_SIZE: usize = usize::pow(2, 12);
