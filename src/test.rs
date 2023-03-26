use bevy::math::{Mat2, Vec2};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::step_update_grid::update_grid;
    use crate::*;
    use approx::*;

    const TEST_GRID_WIDTH: usize = 10;
    const TEST_DT: f32 = 0.1;
    const TEST_GRAVITY: f32 = -0.3;

    #[test]
    fn test_quadratic_interpolation_weights() {
        let cell_diff = Vec2::new(-0.5, -0.5);
        let weights = quadratic_interpolation_weights(cell_diff);
        assert_eq!(
            [
                Vec2::new(0.5, 0.5),
                Vec2::new(0.5, 0.5),
                Vec2::new(0.0, 0.0)
            ],
            weights
        );
    }

    #[test]
    fn test_weighted_velocity_and_cell_dist_to_term_x_zero() {
        let zm = Mat2::from_cols(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        assert_eq!(
            weighted_velocity_and_cell_dist_to_term(Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0)),
            zm
        );
        assert_eq!(0.0, zm.determinant());
    }

    #[test]
    fn test_weighted_velocity_and_cell_dist_to_term_y_zero() {
        let zm = Mat2::from_cols(Vec2::new(1.0, 1.0), Vec2::new(0.0, 0.0));
        assert_eq!(
            weighted_velocity_and_cell_dist_to_term(Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0)),
            zm
        );
        assert_eq!(0.0, zm.determinant());
    }

    #[test]
    fn test_weighted_velocity_and_cell_dist_to_term() {
        let zm = Mat2::from_cols(
            Vec2::new(0.22 * 2.0, 0.77 * 2.0),
            Vec2::new(0.22 * -1.0, 0.77 * -1.0),
        );
        assert_eq!(
            weighted_velocity_and_cell_dist_to_term(Vec2::new(0.22, 0.77), Vec2::new(2.0, -1.0)),
            zm
        );
        assert_eq!(0.0, zm.determinant());
        assert_eq!(zm.row(0).x, 0.22 * 2.0);
        assert_eq!(zm.row(1).x, 0.77 * 2.0);
    }

    #[test]
    // in update_cells system, a single particle in freefall should update mass and velocity of surrounding cells.
    fn test_update_cells_iteration() {
        let mut world = World::default();

        let mut my_schedule = Schedule::new();
        my_schedule.add_systems(
            (
                step_update_cells::update_cells,
                step_update_cells::apply_update_cell_computations,
            )
                .chain(),
        );

        world.insert_resource(Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0,
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
        });
        // add particle to world
        let particle_id = world
            .spawn((
                Position(Vec2::new(5.0, 5.0)),
                Velocity(Vec2::new(0.0, -1.0)),
                Mass(1.06),
                AffineMomentum(Mat2::from_cols(
                    Vec2::new(-0.4838, 0.01124),
                    Vec2::new(-0.0248, 0.169),
                )),
                CellMassMomentumContributions([GridMassAndMomentumChange(0, 0., Vec2::ZERO); 9]),
                ParticleTag,
            ))
            .id();
        // iterate systems
        my_schedule.run(&mut world);

        // particle position should not change
        let particle_position = world.get::<Position>(particle_id);
        assert_eq!(particle_position.unwrap().0, Vec2::new(5.0, 5.0));
        //// particle velocity should not change.
        let particle_velocity = world.get::<Velocity>(particle_id);
        assert_eq!(particle_velocity.unwrap().0, Vec2::new(0.0, -1.0));
        // particle affine momentum should not change
        let particle_momentum = world.get::<AffineMomentum>(particle_id);
        // access col major
        assert_eq!(particle_momentum.unwrap().0.x_axis.x, -0.4838);
        assert_eq!(particle_momentum.unwrap().0.x_axis.y, 0.01124);
        assert_eq!(particle_momentum.unwrap().0.y_axis.x, -0.0248);
        assert_eq!(particle_momentum.unwrap().0.y_axis.y, 0.169);
        // access row major
        assert_eq!(particle_momentum.unwrap().0.row(0).x, -0.4838);
        assert_eq!(particle_momentum.unwrap().0.row(0).y, -0.0248);
        assert_eq!(particle_momentum.unwrap().0.row(1).x, 0.01124);
        assert_eq!(particle_momentum.unwrap().0.row(1).y, 0.169);

        //// local grid cells should be updated from particle.
        let gr = world.get_resource::<Grid>().unwrap();
        assert_eq!(gr.cells[44].mass, 0.265);
        assert_eq!(gr.cells[45].mass, 0.265);
        assert_eq!(gr.cells[46].mass, 0.0);
        assert_eq!(gr.cells[54].mass, 0.265);
        assert_eq!(gr.cells[55].mass, 0.265);
        assert_eq!(gr.cells[56].mass, 0.0);
        assert_eq!(gr.cells[64].mass, 0.0);
        assert_eq!(gr.cells[65].mass, 0.0);
        assert_eq!(gr.cells[66].mass, 0.0);

        assert_abs_diff_eq!(-0.28888178, gr.cells[44].velocity.y,);

        assert_abs_diff_eq!(gr.cells[44].velocity.x, 0.0673895, epsilon = 1e-4);
        assert_abs_diff_eq!(gr.cells[44].velocity.y, -0.2888818, epsilon = 1e-4);
        assert_abs_diff_eq!(gr.cells[45].velocity.x, 0.0608175, epsilon = 1e-4);
        assert_abs_diff_eq!(gr.cells[45].velocity.y, -0.2440968, epsilon = 1e-4);
        assert_eq!(gr.cells[46].velocity, Vec2::new(0.0, 0.0));
        assert_abs_diff_eq!(gr.cells[54].velocity.x, -0.0608175, epsilon = 1e-4);
        assert_abs_diff_eq!(gr.cells[54].velocity.y, -0.2859032, epsilon = 1e-4);
        assert_abs_diff_eq!(gr.cells[55].velocity.x, -0.0673895, epsilon = 1e-4);
        assert_abs_diff_eq!(gr.cells[55].velocity.y, -0.2411182, epsilon = 1e-4);
        assert_eq!(gr.cells[56].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[64].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[65].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[66].velocity, Vec2::new(0.0, 0.0));
    }

    #[test]
    // in particles_to_grid system, a single particle in freefall should update momentum (stored as scaled velocity) of surrounding cells.
    fn test_particles_to_grid_iteration() {
        let mut world = World::default();
        let mut my_schedule = Schedule::new();

        my_schedule.add_systems((step_p2g::particles_to_grid_fluids, update_grid).chain());

        // manually put some mass in the grid at particle location since previous steps not run
        let mut gr = Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0,
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
        };
        let particle_cell_index = gr.index_at(5, 5);
        gr.cells[particle_cell_index].mass = 0.25;
        world.insert_resource(gr);

        world.insert_resource(WorldState::new(TEST_DT, TEST_GRAVITY, true));

        // add particle to world
        let particle_id = world
            .spawn((
                Position(Vec2::new(5.0, 5.0)),
                Velocity(Vec2::new(0.0, -1.0)),
                Mass(1.06),
                NewtonianFluidModel {
                    rest_density: 4.,
                    dynamic_viscosity: 0.1,
                    eos_stiffness: 10.,
                    eos_power: 4.,
                },
                AffineMomentum(Mat2::from_cols(
                    Vec2::new(-0.4838, 0.01124),
                    Vec2::new(-0.0248, 0.169),
                )),
                CellMassMomentumContributions([GridMassAndMomentumChange(0, 0., Vec2::ZERO); 9]),
                ParticleTag,
            ))
            .id();

        // iterate systems
        my_schedule.run(&mut world);

        // particle position should not change
        let particle_position = world.get::<Position>(particle_id);
        assert_eq!(particle_position.unwrap().0, Vec2::new(5.0, 5.0));
        //// particle velocity should not change.
        let particle_velocity = world.get::<Velocity>(particle_id);
        assert_eq!(particle_velocity.unwrap().0, Vec2::new(0.0, -1.0));
        // particle affine momentum should not change
        let particle_momentum = world.get::<AffineMomentum>(particle_id);
        assert_eq!(particle_momentum.unwrap().0.x_axis.x, -0.4838);
        assert_eq!(particle_momentum.unwrap().0.x_axis.y, 0.01124);
        assert_eq!(particle_momentum.unwrap().0.y_axis.x, -0.0248);
        assert_eq!(particle_momentum.unwrap().0.y_axis.y, 0.169);

        //// get grid cells.
        let gr = world.get_resource::<Grid>().unwrap();

        //// local grid cells momentum (as velocity) should be updated from particle.
        assert_abs_diff_eq!(0.042623872, gr.cells[44].velocity.x, epsilon = 1e-4);
        assert_abs_diff_eq!(0.097981312, gr.cells[44].velocity.y, epsilon = 1e-4);
        assert_abs_diff_eq!(0.044923648, gr.cells[45].velocity.x, epsilon = 1e-4);
        assert_abs_diff_eq!(-0.100281088, gr.cells[45].velocity.y, epsilon = 1e-4);
        assert_eq!(gr.cells[46].velocity, Vec2::new(0.0, 0.0));
        assert_abs_diff_eq!(-0.044923648, gr.cells[54].velocity.x, epsilon = 1e-4);
        assert_abs_diff_eq!(0.100281088, gr.cells[54].velocity.y, epsilon = 1e-4);
        assert_abs_diff_eq!(-0.1704955, gr.cells[55].velocity.x, epsilon = 1e-4);
        assert_abs_diff_eq!(-0.42192528, gr.cells[55].velocity.y, epsilon = 1e-4);
        assert_eq!(gr.cells[56].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[64].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[65].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[66].velocity, Vec2::new(0.0, 0.0));
    }

    #[test]
    // in grid_to_particles system, a couple of particles should be effected by surrounding cells.
    fn test_grid_to_particles_iteration() {
        let mut world = World::default();
        let mut my_schedule = Schedule::new();
        my_schedule.add_system(step_g2p::grid_to_particles);
        let mut gr = Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0,
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
        };

        let particle_1_id = world
            .spawn((
                Position(Vec2::new(5.2, 5.3)),
                Velocity(Vec2::new(3.3, 3.0)),
                Mass(1.06),
                AffineMomentum(Mat2::from_cols(
                    Vec2::new(-0.4838, 0.01124),
                    Vec2::new(-0.0248, 0.169),
                )),
                ParticleTag,
            ))
            .id();

        let particle_2_id = world
            .spawn((
                Position(Vec2::new(6.6, 5.9)),
                Velocity(Vec2::new(1.2, -1.0)),
                Mass(1.23),
                AffineMomentum(Mat2::from_cols(
                    Vec2::new(-0.4838, 0.01124),
                    Vec2::new(-0.0248, 0.169),
                )),
                ParticleTag,
            ))
            .id();

        let particle_1_cell_index = gr.index_at(5, 5);
        let particle_2_cell_index = gr.index_at(6, 5);

        // manually put some velocity + mass in the grid at particle location since other systems did not run
        gr.cells[particle_1_cell_index].velocity = Vec2::new(1.0, 1.0);
        gr.cells[particle_1_cell_index].mass = 0.25;

        gr.cells[particle_2_cell_index].velocity = Vec2::new(2.0, 2.0);
        gr.cells[particle_2_cell_index].mass = 0.25;

        world.insert_resource(gr);
        world.insert_resource(WorldState::new(TEST_DT, TEST_GRAVITY, true));

        // iterate systems
        my_schedule.run(&mut world);

        // check particles
        let particle_1_position = world.get::<Position>(particle_1_id);
        let particle_1_velocity = world.get::<Velocity>(particle_1_id);
        let particle_1_mass = world.get::<Mass>(particle_1_id);
        let particle_1_affine_momentum = world.get::<AffineMomentum>(particle_1_id);
        let particle_2_position = world.get::<Position>(particle_2_id);
        let particle_2_velocity = world.get::<Velocity>(particle_2_id);
        let particle_2_mass = world.get::<Mass>(particle_2_id);
        let particle_2_affine_momentum = world.get::<AffineMomentum>(particle_2_id);

        assert_abs_diff_eq!(particle_1_position.unwrap().0.x, 5.2497, epsilon = 1e-4);
        assert_abs_diff_eq!(particle_1_position.unwrap().0.y, 5.3497, epsilon = 1e-4);
        assert_abs_diff_eq!(particle_1_velocity.unwrap().0.x, 0.497, epsilon = 1e-4);
        assert_abs_diff_eq!(particle_1_velocity.unwrap().0.y, 0.497, epsilon = 1e-4);
        assert_eq!(particle_1_mass.unwrap().0, 1.06);
        assert_abs_diff_eq!(
            particle_1_affine_momentum.unwrap().0.x_axis.x,
            0.71,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(
            particle_1_affine_momentum.unwrap().0.x_axis.y,
            0.71,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(
            particle_1_affine_momentum.unwrap().0.y_axis.x,
            0.3976,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(
            particle_1_affine_momentum.unwrap().0.y_axis.y,
            0.3976,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(particle_2_position.unwrap().0.x, 6.692, epsilon = 1e-4);
        assert_abs_diff_eq!(particle_2_position.unwrap().0.y, 5.992, epsilon = 1e-4);
        assert_abs_diff_eq!(particle_2_velocity.unwrap().0.x, 0.13631988, epsilon = 1e-4);
        assert_abs_diff_eq!(particle_2_velocity.unwrap().0.y, 0.8363197, epsilon = 1e-4);
        assert_eq!(particle_2_mass.unwrap().0, 1.23);
        assert_abs_diff_eq!(
            particle_2_affine_momentum.unwrap().0.x_axis.x,
            -0.557,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(
            particle_2_affine_momentum.unwrap().0.x_axis.y,
            -0.557,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(
            particle_2_affine_momentum.unwrap().0.y_axis.x,
            -1.47264,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(
            particle_2_affine_momentum.unwrap().0.y_axis.y,
            -1.47264,
            epsilon = 1e-4
        );
    }

    #[test]
    // reset_grid should zero out the cells
    fn test_reset_grid() {
        let mut gr = Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0,
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
        };
        gr.cells[7].mass = 0.25;
        gr.cells[7].velocity = Vec2::new(1.2, 2.3);

        gr.reset();

        assert_eq!(gr.cells[7].mass, 0.0);
        assert_eq!(gr.cells[7].velocity.x, 0.0);
        assert_eq!(gr.cells[7].velocity.y, 0.0);
    }

    #[test]
    // update_grid should adjust particle velocity and apply boundary conditions
    fn test_update_grid() {
        let mut gr = Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0,
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
        };

        // add border cell with mass and velocity
        let border_cell_index = gr.index_at(3, 0);
        gr.cells[border_cell_index] = Cell {
            velocity: Vec2::new(2.2, -2.4),
            mass: 1.17171717,
        };

        // add middle cell with mass and velocity
        let middle_cell_index = gr.index_at(5, 5);
        gr.cells[middle_cell_index] = Cell {
            velocity: Vec2::new(3.7333, -1.111),
            mass: 3.333,
        };

        // apply grid update
        gr.update(TEST_DT, TEST_GRAVITY);

        // border cell should have updated velocity and -y velocity cancelled
        assert_abs_diff_eq!(
            1.8775,
            gr.cells[border_cell_index].velocity.x,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(0.0, gr.cells[border_cell_index].velocity.y, epsilon = 1e-8);
        assert_abs_diff_eq!(1.17171717, gr.cells[border_cell_index].mass, epsilon = 1e-4);

        // middle cell should have updated velocity
        assert_abs_diff_eq!(
            1.1201,
            gr.cells[middle_cell_index].velocity.x,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(
            -0.3633,
            gr.cells[middle_cell_index].velocity.y,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(3.3329, gr.cells[middle_cell_index].mass, epsilon = 1e-4);
    }
}
