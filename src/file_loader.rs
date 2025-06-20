//! File loading and validation utilities for labyrinth map files.

use std::fs;

use color_eyre::eyre::{OptionExt as _, Result};

use crate::map::Map;

/// Scans the current directory for .labmap files and loads them.
///
/// This function searches for files with the .labmap extension in the current working directory,
/// validates their format, and adds them to the maps collection for user selection. It skips
/// invalid files and continues processing valid ones.
pub(crate) fn fetch_files(maps: &mut Vec<Map>) -> Result<()> {
    for file in fs::read_dir(".")? {
        match file {
            Ok(file)
                if !file.file_type()?.is_dir()
                    && file
                        .file_name()
                        .to_str()
                        .ok_or_eyre("failed to convert osstring to string slice")?
                        .ends_with(".labmap") =>
            {
                let contents = fs::read_to_string(file.path())?;

                if parse_file_contents(contents.trim()) {
                    maps.push(Map::new(file.file_name(), &contents)?);
                }
            }
            Err(err) => return Err(err.into()),
            _ => {}
        }
    }

    Ok(())
}

/// Validates the format and content of labyrinth map files.
///
/// This function performs validation to ensure the maze format follows the specification:
/// - Contains only valid characters (1-4)
/// - Has consistent row lengths
/// - Has exactly one entry point (1)
/// - Is completely surrounded by walls (2s) except for exit points on the edges
pub(crate) fn parse_file_contents(input: &str) -> bool {
    let lines: Vec<&str> = input.lines().collect();

    // Must have at least 3x3 to form a proper walled maze
    if lines.len() < 3 {
        return false;
    }

    let mut entry_point_counter = 0;
    let Some(first_line) = lines.first() else {
        return false;
    };
    let expected_width = first_line.len();

    // Must have at least 3 columns to form proper walls
    if expected_width < 3 {
        return false;
    }

    // Validate each line
    for line in &lines {
        // Check consistent row lengths
        if line.len() != expected_width {
            return false;
        }

        // Check valid characters only
        if !line
            .bytes()
            .all(|byte| matches!(byte, b'1' | b'2' | b'3' | b'4'))
        {
            return false;
        }

        // Count entry points
        for byte in line.bytes() {
            if byte == b'1' {
                entry_point_counter += 1;
            }
        }

        // Too many entry points
        if entry_point_counter > 1 {
            return false;
        }
    }

    // Must have exactly one entry point
    if entry_point_counter != 1 {
        return false;
    }

    let last_row_idx = lines.len() - 1;
    let last_col_idx = expected_width - 1;

    // Check boundary walls and validate maze structure in a single pass
    for (row_idx, line) in lines.iter().enumerate() {
        for (col_idx, byte) in line.bytes().enumerate() {
            let is_edge =
                row_idx == 0 || row_idx == last_row_idx || col_idx == 0 || col_idx == last_col_idx;

            if is_edge {
                // On edges: only walls (2) or exit points (4) allowed
                if !matches!(byte, b'2' | b'4') {
                    return false;
                }
            } else {
                // Interior: exit points (4) not allowed
                if byte == b'4' {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_contents_valid_maze() {
        let valid_maze = "2222\n2134\n2222";
        assert!(parse_file_contents(valid_maze));
    }

    #[test]
    fn test_parse_file_contents_valid_complex_maze() {
        let valid_maze = "22224\n21332\n23332\n22222";
        assert!(parse_file_contents(valid_maze));
    }

    #[test]
    fn test_parse_file_contents_too_small_height() {
        let invalid_maze = "222\n213";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_too_small_width() {
        let invalid_maze = "22\n21\n22";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_inconsistent_row_lengths() {
        let invalid_maze = "2222\n213\n2222";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_invalid_characters() {
        let invalid_maze = "2222\n21x4\n2222";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_no_entry_point() {
        let invalid_maze = "2222\n2334\n2222";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_multiple_entry_points() {
        let invalid_maze = "2222\n2114\n2222";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_entry_point_on_edge() {
        let invalid_maze = "2122\n2334\n2222";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_exit_point_in_interior() {
        let invalid_maze = "2222\n2143\n2222";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_non_wall_on_edge() {
        let invalid_maze = "2322\n2134\n2222";
        assert!(!parse_file_contents(invalid_maze));
    }

    #[test]
    fn test_parse_file_contents_empty_input() {
        assert!(!parse_file_contents(""));
    }

    #[test]
    fn test_parse_file_contents_single_line() {
        assert!(!parse_file_contents("222"));
    }
}
