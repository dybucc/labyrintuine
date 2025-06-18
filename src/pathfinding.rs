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
