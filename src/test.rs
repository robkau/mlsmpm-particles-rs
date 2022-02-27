use bevy::math::{Mat2, Vec2};

#[cfg(test)]
mod tests {
    use crate::*;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

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
        let mut update_stage = SystemStage::parallel();
        world.insert_resource(Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
            dt: TEST_DT,
            current_tick: 0,
            gravity_enabled: true,
            gravity: TEST_GRAVITY,
        });
        update_stage.add_system(update_cells);
        // add particle to world
        let particle_id = world
            .spawn()
            .insert_bundle((
                Position(Vec2::new(5.0, 5.0)),
                Velocity(Vec2::new(0.0, -1.0)),
                Mass(1.06),
                AffineMomentum(Mat2::from_cols(
                    Vec2::new(-0.4838, 0.01124),
                    Vec2::new(-0.0248, 0.169),
                )),
                ParticleTag,
            ))
            .id();
        // iterate systems
        update_stage.run(&mut world);

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
        assert_eq!(true, approx_equal(gr.cells[44].velocity.x, 0.0673895, 5));
        assert_eq!(true, approx_equal(gr.cells[44].velocity.y, -0.2888818, 5));
        assert_eq!(true, approx_equal(gr.cells[45].velocity.x, 0.0608175, 5));
        assert_eq!(true, approx_equal(gr.cells[45].velocity.y, -0.2440968, 5));
        assert_eq!(gr.cells[46].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(true, approx_equal(gr.cells[54].velocity.x, -0.0608175, 5));
        assert_eq!(true, approx_equal(gr.cells[54].velocity.y, -0.2859032, 5));
        assert_eq!(true, approx_equal(gr.cells[55].velocity.x, -0.0673895, 5));
        assert_eq!(true, approx_equal(gr.cells[55].velocity.y, -0.2411182, 5));
        assert_eq!(gr.cells[56].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[64].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[65].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[66].velocity, Vec2::new(0.0, 0.0));
    }

    #[test]
    // in particles_to_grid system, a single particle in freefall should update momentum (stored as scaled velocity) of surrounding cells.
    fn test_particles_to_grid_iteration() {
        let mut world = World::default();
        let mut update_stage = SystemStage::parallel();
        // manually put some mass in the grid at particle location since previous steps not run
        let mut gr = Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
            dt: TEST_DT,
            current_tick: 0,
            gravity_enabled: true,
            gravity: TEST_GRAVITY,
        };
        let particle_cell_index = gr.index_at(5, 5);
        gr.cells[particle_cell_index].mass = 0.25;
        world.insert_resource(gr);
        update_stage.add_system(particles_to_grid);
        // add particle to world
        let particle_id = world
            .spawn()
            .insert_bundle((
                Position(Vec2::new(5.0, 5.0)),
                Velocity(Vec2::new(0.0, -1.0)),
                Mass(1.06),
                RestDensity(4.),
                DynamicViscosity(0.1),
                EosStiffness(10.),
                EosPower(4.),
                AffineMomentum(Mat2::from_cols(
                    Vec2::new(-0.4838, 0.01124),
                    Vec2::new(-0.0248, 0.169),
                )),
                ParticleTag,
            ))
            .id();

        // iterate systems
        update_stage.run(&mut world);

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
        assert_eq!(true, approx_equal(gr.cells[44].velocity.x, 0.042623872, 5));
        assert_eq!(true, approx_equal(gr.cells[44].velocity.y, 0.097981312, 5));
        assert_eq!(true, approx_equal(gr.cells[45].velocity.x, 0.044923648, 5));
        assert_eq!(true, approx_equal(gr.cells[45].velocity.y, -0.100281088, 5));
        assert_eq!(gr.cells[46].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(true, approx_equal(gr.cells[54].velocity.x, -0.044923648, 5));
        assert_eq!(true, approx_equal(gr.cells[54].velocity.y, 0.100281088, 5));
        assert_eq!(true, approx_equal(gr.cells[55].velocity.x, -0.042623872, 5));
        assert_eq!(true, approx_equal(gr.cells[55].velocity.y, -0.097981312, 5));
        assert_eq!(gr.cells[56].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[64].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[65].velocity, Vec2::new(0.0, 0.0));
        assert_eq!(gr.cells[66].velocity, Vec2::new(0.0, 0.0));
    }

    #[test]
    // in grid_to_particles system, a couple of particles should be effected by surrounding cells.
    fn test_grid_to_particles_iteration() {
        let mut world = World::default();
        let mut update_stage = SystemStage::parallel();
        let mut gr = Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
            dt: TEST_DT,
            current_tick: 0,
            gravity_enabled: true,
            gravity: TEST_GRAVITY,
        };

        let particle_1_id = world
            .spawn()
            .insert_bundle((
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
            .spawn()
            .insert_bundle((
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
        update_stage.add_system(grid_to_particles);

        // iterate systems
        update_stage.run(&mut world);

        // check particles
        let particle_1_position = world.get::<Position>(particle_1_id);
        let particle_1_velocity = world.get::<Velocity>(particle_1_id);
        let particle_1_mass = world.get::<Mass>(particle_1_id);
        let particle_1_affine_momentum = world.get::<AffineMomentum>(particle_1_id);
        let particle_2_position = world.get::<Position>(particle_2_id);
        let particle_2_velocity = world.get::<Velocity>(particle_2_id);
        let particle_2_mass = world.get::<Mass>(particle_2_id);
        let particle_2_affine_momentum = world.get::<AffineMomentum>(particle_2_id);

        assert_eq!(
            true,
            approx_equal(particle_1_position.unwrap().0.x, 5.2497, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_1_position.unwrap().0.y, 5.3496, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_1_velocity.unwrap().0.x, 0.497, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_1_velocity.unwrap().0.y, 0.497, 3)
        );
        assert_eq!(particle_1_mass.unwrap().0, 1.06);
        assert_eq!(
            true,
            approx_equal(particle_1_affine_momentum.unwrap().0.x_axis.x, 0.71, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_1_affine_momentum.unwrap().0.x_axis.y, 0.71, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_1_affine_momentum.unwrap().0.y_axis.x, 0.3976, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_1_affine_momentum.unwrap().0.y_axis.y, 0.3976, 3)
        );

        assert_eq!(
            true,
            approx_equal(particle_2_position.unwrap().0.x, 6.692, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_2_position.unwrap().0.y, 5.992, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_2_velocity.unwrap().0.x, -0.69135, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_2_velocity.unwrap().0.y, 0.00704, 3)
        );
        assert_eq!(particle_2_mass.unwrap().0, 1.23);
        assert_eq!(
            true,
            approx_equal(particle_2_affine_momentum.unwrap().0.x_axis.x, -0.557, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_2_affine_momentum.unwrap().0.x_axis.y, -0.557, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_2_affine_momentum.unwrap().0.y_axis.x, -1.47264, 3)
        );
        assert_eq!(
            true,
            approx_equal(particle_2_affine_momentum.unwrap().0.y_axis.y, -1.47264, 3)
        );
    }

    #[test]
    // reset_grid should zero out the cells
    fn test_reset_grid() {
        let mut gr = Grid {
            cells: vec![
                Cell {
                    velocity: Vec2::ZERO,
                    mass: 0.0
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
            dt: TEST_DT,
            current_tick: 0,
            gravity_enabled: true,
            gravity: TEST_GRAVITY,
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
                    mass: 0.0
                };
                TEST_GRID_WIDTH * TEST_GRID_WIDTH
            ],
            width: TEST_GRID_WIDTH,
            dt: TEST_DT,
            current_tick: 0,
            gravity_enabled: true,
            gravity: TEST_GRAVITY,
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
        gr.update();

        // border cell should have updated velocity and -y velocity cancelled
        assert_eq!(
            true,
            approx_equal(gr.cells[border_cell_index].velocity.x, 1.8757, 5)
        );
        assert_eq!(
            true,
            approx_equal(gr.cells[border_cell_index].velocity.y, 0.0, 5)
        );
        assert_eq!(gr.cells[border_cell_index].mass, 1.17171717);

        // middle cell should have updated velocity
        assert_eq!(
            true,
            approx_equal(gr.cells[middle_cell_index].velocity.x, 1.120102, 5)
        );
        assert_eq!(
            true,
            approx_equal(gr.cells[middle_cell_index].velocity.y, -0.36333, 5)
        );
        assert_eq!(gr.cells[middle_cell_index].mass, 3.333);
    }
}

fn mat2_equal(a: Mat2, b: Mat2, dp: u8) -> bool {
    vec2_equal(a.x_axis, b.x_axis, dp) && vec2_equal(a.y_axis, b.y_axis, dp)
}

fn vec2_equal(a: Vec2, b: Vec2, dp: u8) -> bool {
    approx_equal(a.x, b.x, dp) && approx_equal(a.y, b.y, dp)
}

fn approx_equal(a: f32, b: f32, dp: u8) -> bool {
    let p = 10f32.powi(-(dp as i32));
    (a - b).abs() < p
}
