use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Cell {
    pub(crate) velocity: Vec2,
    pub(crate) mass: f32,
}

// MPM grid resource
#[derive(Clone, Resource)]
pub(crate) struct Grid {
    pub(crate) cells: Vec<Cell>,
    pub(crate) width: usize,
}

impl Grid {
    pub(crate) fn new(width: usize) -> Grid {
        Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0,
                };
                width * width
            ],
            width,
        }
    }

    pub(crate) fn index_at(&self, x: usize, y: usize) -> usize {
        x * self.width + y
    }

    pub(crate) fn reset(&mut self) {
        for mut cell in self.cells.iter_mut() {
            cell.velocity = Vec2::ZERO;
            cell.mass = 0.0;
        }
    }

    pub(crate) fn update(&mut self, dt: f32, gravity: f32) {
        for (i, cell) in self.cells.iter_mut().enumerate() {
            if cell.mass > 0.0 {
                // convert momentum to velocity, apply gravity
                cell.velocity *= 1.0 / cell.mass;
                cell.velocity.y += dt * gravity;

                // boundary conditions
                let x = i / self.width;
                let y = i % self.width;
                if x < 2 {
                    // can only stay in place or go right
                    if cell.velocity.x < 0.0 {
                        cell.velocity.x = 0.0;
                    }
                }
                if x > self.width - 3 {
                    // can only stay in place or go left
                    if cell.velocity.x > 0.0 {
                        cell.velocity.x = 0.0;
                    }
                }
                if y < 2 {
                    // can only stay in place or go up
                    if cell.velocity.y < 0.0 {
                        cell.velocity.y = 0.0;
                    }
                }
                if y > self.width - 3 {
                    // can only stay in place or go down
                    if cell.velocity.y > 0.0 {
                        cell.velocity.y = 0.0;
                    }
                }
            }
        }
    }
}

pub(crate) fn reset_grid(mut grid: ResMut<Grid>) {
    grid.reset();
}

pub(crate) fn quadratic_interpolation_weights(cell_diff: Vec2) -> [Vec2; 3] {
    [
        Vec2::new(
            0.5 * f32::powi(0.5 - cell_diff.x, 2),
            0.5 * f32::powi(0.5 - cell_diff.y, 2),
        ),
        Vec2::new(
            0.75 - f32::powi(cell_diff.x, 2),
            0.75 - f32::powi(cell_diff.y, 2),
        ),
        Vec2::new(
            0.5 * f32::powi(0.5 + cell_diff.x, 2),
            0.5 * f32::powi(0.5 + cell_diff.y, 2),
        ),
    ]
}

pub(crate) fn weighted_velocity_and_cell_dist_to_term(
    weighted_velocity: Vec2,
    cell_dist: Vec2,
) -> Mat2 {
    Mat2::from_cols(
        Vec2::new(
            weighted_velocity[0] * cell_dist[0],
            weighted_velocity[1] * cell_dist[0],
        ),
        Vec2::new(
            weighted_velocity[0] * cell_dist[1],
            weighted_velocity[1] * cell_dist[1],
        ),
    )
}
