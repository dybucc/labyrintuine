//! Pathfinding algorithm and animation module.
//!
//! This module contains the pathfinding algorithm implementation, animation system, and coordinate
//! transformation utilities for maze solving visualization.

use std::time::{Duration, Instant};

use color_eyre::eyre::{OptionExt as _, Result};

/// Animation frame delay in milliseconds.
///
/// This constant controls the timing between animation frames in the pathfinding visualization. A
/// lower value results in faster animation, while a higher value slows down the animation to make
/// it easier to follow the algorithm's progress.
pub(crate) const ANIMATION_FRAME_DELAY_MS: u64 = 200;

/// Animation step types for pathfinding visualization.
///
/// This enumeration represents the different types of steps that can occur during the animated
/// pathfinding visualization, allowing for proper rendering of both forward exploration and
/// backtracking behavior.
#[derive(Debug, Clone)]
pub(crate) enum AnimationStep {
    /// Add a coordinate to the current path visualization.
    ///
    /// This variant represents moving forward in the pathfinding algorithm by adding a new
    /// coordinate to the currently displayed path.
    Add(usize, usize),
    /// Remove a coordinate from the current path visualization.
    ///
    /// This variant represents backtracking in the pathfinding algorithm by removing a coordinate
    /// from the currently displayed path.
    Remove(usize, usize),
}

/// Animation state manager for pathfinding visualization.
///
/// This structure manages the animation state including timing, current step tracking, and the
/// coordinate path being displayed during the animated maze solving.
pub(crate) struct AnimationManager {
    /// Animation steps recorded during pathfinding.
    ///
    /// This field stores a sequence of animation steps that represent the pathfinding algorithm's
    /// traversal process, including forward moves and backtracking. Each step contains the
    /// coordinates to be drawn or removed from the visualization.
    pub steps: Vec<AnimationStep>,
    /// Current step in the animation sequence.
    ///
    /// This field tracks the current position in the [`steps`](AnimationManager::steps) vector to
    /// determine which steps have been rendered and which are still pending.
    pub current_index: usize,
    /// Timestamp of the last animation frame update.
    ///
    /// This field stores the time when the animation was last updated, used to control the timing
    /// between animation frames for smooth visualization.
    pub last_update_time: Instant,
    /// Current set of coordinates being displayed in the animation.
    ///
    /// This field maintains the currently visible path coordinates during animation, allowing for
    /// proper backtracking visualization by removing coordinates when needed.
    pub current_path: Vec<(usize, usize)>,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationManager {
    /// Creates a new animation manager with default values.
    pub(crate) fn new() -> Self {
        Self {
            steps: Vec::new(),
            current_index: 0,
            last_update_time: Instant::now(),
            current_path: Vec::new(),
        }
    }

    /// Resets the animation state to the beginning.
    pub(crate) fn reset(&mut self) {
        self.current_index = 0;
        self.current_path.clear();
        self.last_update_time = Instant::now();
    }

    /// Clears all animation data and resets state.
    pub(crate) fn clear(&mut self) {
        self.steps.clear();
        self.reset();
    }

    /// Updates the animation state based on timing and current progress.
    ///
    /// This method advances the animation by processing the next step in the animation sequence
    /// when enough time has passed. It handles both adding and removing coordinates from the
    /// current animation path to show the pathfinding exploration and backtracking.
    pub(crate) fn update(&mut self) {
        // Check if enough time has passed for the next animation frame
        if self.last_update_time.elapsed() >= Duration::from_millis(ANIMATION_FRAME_DELAY_MS) {
            self.last_update_time = Instant::now();

            if self.current_index < self.steps.len() {
                // Process the next animation step
                if let Some(step) = self.steps.get(self.current_index) {
                    match step {
                        AnimationStep::Add(x, y) => {
                            self.current_path.push((*x, *y));
                        }
                        AnimationStep::Remove(x, y) => {
                            // Remove the coordinate from current path (backtracking)
                            if let Some(pos) = self
                                .current_path
                                .iter()
                                .position(|&coord| coord == (*x, *y))
                            {
                                let _ = self.current_path.remove(pos);
                            }
                        }
                    }
                }

                self.current_index += 1;
            } else {
                // Animation complete, restart from beginning
                self.reset();
            }
        }
    }
}

/// Records animation steps during pathfinding for later visualization.
///
/// This function performs depth-first search to explore the maze and records each step of the
/// algorithm (forward moves and backtracking) for animated playback. It captures the exact
/// sequence of the pathfinding algorithm's exploration from the entry point through the maze.
pub(crate) fn record_animation_steps(
    map_data: &[String],
    start: (usize, usize),
    current_path: &mut Vec<(usize, usize)>,
    animation_steps: &mut Vec<AnimationStep>,
) {
    // Record adding current position to path
    current_path.push(start);
    animation_steps.push(AnimationStep::Add(start.0, start.1));

    // Check if we've reached an exit point ('4')
    if let Some(row) = map_data.get(start.1) {
        if let Some(cell) = row.as_bytes().get(start.0) {
            if *cell == b'4' {
                // Found exit - record removing position during backtrack
                let _ = current_path.pop();
                animation_steps.push(AnimationStep::Remove(start.0, start.1));
                return;
            }
        }
    }

    // Explore all four directions (north, south, east, west)
    let directions = [(0_i32, -1_i32), (0, 1), (1, 0), (-1, 0)];

    for (dx, dy) in directions {
        // Calculate neighbor coordinates with proper bounds checking
        let Some(new_x) = start.0.checked_add_signed(dx as isize) else {
            continue;
        };
        let Some(new_y) = start.1.checked_add_signed(dy as isize) else {
            continue;
        };

        let new_pos = (new_x, new_y);

        // Skip if already visited in current path
        if current_path.contains(&new_pos) {
            continue;
        }

        // Check if position is valid and walkable
        if let Some(row) = map_data.get(new_pos.1) {
            if let Some(cell) = row.chars().nth(new_pos.0) {
                // Only explore walkable cells ('3') or exit ('4')
                if matches!(cell, '3' | '4') {
                    // Recursively explore from this position
                    record_animation_steps(map_data, new_pos, current_path, animation_steps);
                }
            }
        }
    }

    // Record removing position during backtrack
    let _ = current_path.pop();
    animation_steps.push(AnimationStep::Remove(start.0, start.1));
}

/// Transforms maze coordinates to screen coordinates for canvas rendering.
///
/// This function converts maze coordinates (col, row) to screen coordinates (x, y) using the
/// standard transformation formulas: coordinate[i] = (n - 1) / 2 - i for rows (ascending order) and
/// coordinate[i] = i - (n - 1) / 2 for columns (descending order).
///
/// # Errors
///
/// This function may return errors from coordinate conversion operations.
pub(crate) fn transform_maze_to_screen_coords(
    maze_coords: &[(usize, usize)],
    map_data: &[String],
) -> Result<Vec<(f64, f64)>> {
    let rows_n = f64::from(u16::try_from(map_data.len())?);
    let cols_n = f64::from(u16::try_from(
        map_data
            .first()
            .ok_or_eyre("failed to retrieve first element of map data")?
            .len(),
    )?);

    maze_coords
        .iter()
        .map(|&(col, row)| {
            // Row transformation: coordinate[i] = (n - 1) / 2 - i
            let screen_y = (rows_n - 1.) / 2. - f64::from(u16::try_from(row)?);

            // Column transformation: coordinate[i] = i - (n - 1) / 2
            let screen_x = f64::from(u16::try_from(col)?) - (cols_n - 1.) / 2.;

            Ok((screen_x, screen_y))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_step_creation() {
        let add_step = AnimationStep::Add(5, 10);
        let remove_step = AnimationStep::Remove(3, 7);

        match add_step {
            AnimationStep::Add(x, y) => {
                assert_eq!(x, 5);
                assert_eq!(y, 10);
            }
            _ => panic!("Expected Add variant"),
        }

        match remove_step {
            AnimationStep::Remove(x, y) => {
                assert_eq!(x, 3);
                assert_eq!(y, 7);
            }
            _ => panic!("Expected Remove variant"),
        }
    }

    #[test]
    fn test_animation_manager_new() {
        let manager = AnimationManager::new();

        assert!(manager.steps.is_empty());
        assert_eq!(manager.current_index, 0);
        assert!(manager.current_path.is_empty());
    }

    #[test]
    fn test_animation_manager_default() {
        let manager = AnimationManager::default();

        assert!(manager.steps.is_empty());
        assert_eq!(manager.current_index, 0);
        assert!(manager.current_path.is_empty());
    }

    #[test]
    fn test_animation_manager_reset() {
        let mut manager = AnimationManager::new();

        // Add some data
        manager.steps.push(AnimationStep::Add(1, 2));
        manager.current_index = 5;
        manager.current_path.push((3, 4));

        manager.reset();

        // Steps should remain, but index and path should be reset
        assert_eq!(manager.steps.len(), 1);
        assert_eq!(manager.current_index, 0);
        assert!(manager.current_path.is_empty());
    }

    #[test]
    fn test_animation_manager_clear() {
        let mut manager = AnimationManager::new();

        // Add some data
        manager.steps.push(AnimationStep::Add(1, 2));
        manager.current_index = 5;
        manager.current_path.push((3, 4));

        manager.clear();

        // Everything should be cleared
        assert!(manager.steps.is_empty());
        assert_eq!(manager.current_index, 0);
        assert!(manager.current_path.is_empty());
    }

    #[test]
    fn test_animation_manager_update_add_step() {
        let mut manager = AnimationManager::new();
        manager.steps.push(AnimationStep::Add(5, 10));

        // Force timing to be ready for update
        manager.last_update_time = Instant::now()
            .checked_sub(Duration::from_millis(ANIMATION_FRAME_DELAY_MS + 10))
            .expect("Duration subtraction should not underflow in test");

        manager.update();

        assert_eq!(manager.current_index, 1);
        assert_eq!(manager.current_path.len(), 1);
        assert_eq!(
            manager
                .current_path
                .first()
                .expect("Path should have at least one element"),
            &(5, 10)
        );
    }

    #[test]
    fn test_animation_manager_update_remove_step() {
        let mut manager = AnimationManager::new();
        manager.current_path.push((5, 10));
        manager.steps.push(AnimationStep::Remove(5, 10));

        // Force timing to be ready for update
        manager.last_update_time = Instant::now()
            .checked_sub(Duration::from_millis(ANIMATION_FRAME_DELAY_MS + 10))
            .expect("Duration subtraction should not underflow in test");

        manager.update();

        assert_eq!(manager.current_index, 1);
        assert!(manager.current_path.is_empty());
    }

    #[test]
    fn test_animation_manager_update_timing() {
        let mut manager = AnimationManager::new();
        manager.steps.push(AnimationStep::Add(1, 2));

        // Update immediately after creation - shouldn't advance
        manager.update();

        assert_eq!(manager.current_index, 0);
        assert!(manager.current_path.is_empty());
    }

    #[test]
    fn test_animation_manager_update_loop_restart() {
        let mut manager = AnimationManager::new();
        manager.steps.push(AnimationStep::Add(1, 2));
        manager.current_index = 1; // At end of steps
        manager.current_path.push((1, 2));

        // Force timing to be ready for update
        manager.last_update_time = Instant::now()
            .checked_sub(Duration::from_millis(ANIMATION_FRAME_DELAY_MS + 10))
            .expect("Duration subtraction should not underflow in test");

        manager.update();

        // Should reset to beginning
        assert_eq!(manager.current_index, 0);
        assert!(manager.current_path.is_empty());
    }

    #[test]
    fn test_record_animation_steps_with_exit() {
        let map_data = vec!["11111".to_owned(), "13341".to_owned(), "11111".to_owned()];

        let mut current_path = Vec::new();
        let mut animation_steps = Vec::new();

        record_animation_steps(&map_data, (1, 1), &mut current_path, &mut animation_steps);

        // Should have recorded some steps
        assert!(!animation_steps.is_empty());
        // First step should be adding the start position
        match animation_steps
            .first()
            .expect("Animation steps should not be empty")
        {
            AnimationStep::Add(x, y) => {
                assert_eq!(*x, 1);
                assert_eq!(*y, 1);
            }
            _ => panic!("Expected first step to be Add"),
        }
    }

    #[test]
    fn test_record_animation_steps_direct_exit() {
        let map_data = vec!["111".to_owned(), "141".to_owned(), "111".to_owned()];

        let mut current_path = Vec::new();
        let mut animation_steps = Vec::new();

        record_animation_steps(&map_data, (1, 1), &mut current_path, &mut animation_steps);

        // Should add position then immediately remove it upon finding exit
        assert_eq!(animation_steps.len(), 2);
        match animation_steps
            .first()
            .expect("First animation step should exist")
        {
            AnimationStep::Add(1, 1) => {}
            _ => panic!("Expected Add step"),
        }
        match animation_steps
            .get(1)
            .expect("Second animation step should exist")
        {
            AnimationStep::Remove(1, 1) => {}
            _ => panic!("Expected Remove step"),
        }
    }

    #[test]
    fn test_transform_maze_to_screen_coords_basic() {
        let map_data = vec!["111".to_owned(), "131".to_owned(), "111".to_owned()];

        let maze_coords = vec![(1, 1)];
        let result = transform_maze_to_screen_coords(&maze_coords, &map_data)
            .expect("Transform should work with valid data");

        assert_eq!(result.len(), 1);
        let (screen_x, screen_y) = result
            .first()
            .expect("Result should have at least one element")
            .to_owned();

        // For 3x3 grid, center (1,1) should map to (0.0, 0.0)
        assert!((screen_x - 0.0).abs() < f64::EPSILON);
        assert!((screen_y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transform_maze_to_screen_coords_multiple_points() {
        let map_data = vec![
            "11111".to_owned(),
            "13331".to_owned(),
            "13331".to_owned(),
            "13331".to_owned(),
            "11111".to_owned(),
        ];

        let maze_coords = vec![(0, 0), (2, 2), (4, 4)];
        let result = transform_maze_to_screen_coords(&maze_coords, &map_data)
            .expect("Transform should work with valid data");

        assert_eq!(result.len(), 3);

        // For 5x5 grid:
        // (0,0) -> x = 0 - 2 = -2.0, y = 2 - 0 = 2.0
        let (x0, y0) = result
            .first()
            .expect("Result should have first element")
            .to_owned();
        assert!((x0 - (-2.0)).abs() < f64::EPSILON);
        assert!((y0 - 2.0).abs() < f64::EPSILON);
        // (2,2) -> x = 2 - 2 = 0.0, y = 2 - 2 = 0.0
        let (x1, y1) = result
            .get(1)
            .expect("Result should have second element")
            .to_owned();
        assert!((x1 - 0.0).abs() < f64::EPSILON);
        assert!((y1 - 0.0).abs() < f64::EPSILON);
        // (4,4) -> x = 4 - 2 = 2.0, y = 2 - 4 = -2.0
        let (x2, y2) = result
            .get(2)
            .expect("Result should have third element")
            .to_owned();
        assert!((x2 - 2.0).abs() < f64::EPSILON);
        assert!((y2 - (-2.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transform_maze_to_screen_coords_empty_input() {
        let map_data = vec!["111".to_owned(), "131".to_owned(), "111".to_owned()];

        let maze_coords = vec![];
        let result = transform_maze_to_screen_coords(&maze_coords, &map_data)
            .expect("Transform should work with empty input");

        assert!(result.is_empty());
    }

    #[test]
    fn test_transform_maze_to_screen_coords_error_empty_map() {
        let map_data: Vec<String> = vec![];
        let maze_coords = vec![(0, 0)];

        let result = transform_maze_to_screen_coords(&maze_coords, &map_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_animation_frame_delay_constant() {
        assert_eq!(ANIMATION_FRAME_DELAY_MS, 200);
    }
}
