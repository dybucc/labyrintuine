//! Map data and management module.
//!
//! This module contains the `Map` struct and related functionality for handling labyrinth map data,
//! including loading, validation, and the default map.

use std::{ffi::OsString, sync::LazyLock};

use color_eyre::eyre::{OptionExt as _, Result};

/// Labyrinth map data container.
///
/// This structure represents the custom type employed for indexing into files and retrieving the
/// contents of labyrinth maps. It is used within a vector to get a kind of ordered hashmap.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
pub(crate) struct Map {
    /// Display name of the map.
    ///
    /// This field represents the key retrieved as a filename without the file extension for the
    /// map.
    pub key: String,
    /// Map content as rows of strings.
    ///
    /// This field represents the actual map stored as a vector of strings, each string representing
    /// a row in the map.
    pub data: Vec<String>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new("Default.labmap".into(), *DEFAULT_MAP).expect("failed to create default map")
    }
}

impl Map {
    /// Builds a new map from a filename and multiline string content.
    ///
    /// This function parses the provided string data into individual rows and extracts a clean
    /// filename by removing the .labmap extension. It validates the input and returns an error if
    /// the filename processing fails.
    ///
    /// # Errors
    ///
    /// This function may return errors if:
    /// - The `OsString` cannot be converted to a string slice
    /// - The filename doesn't contain the expected ".labmap" extension
    pub(crate) fn new(key: OsString, data: &str) -> Result<Self> {
        let mut vec = Vec::new();
        for line in data.lines() {
            vec.push(line.to_owned());
        }

        let mut file_name = key
            .to_str()
            .ok_or_eyre("failed to convert osstring to string slice")?
            .to_owned();
        file_name.truncate({
            file_name
                .rfind(".labmap")
                .ok_or_eyre("failed to find extension in file name")?
        });

        Ok(Self {
            key: file_name,
            data: vec,
        })
    }
}

/// Default labyrinth map used as fallback.
///
/// This static holds the default map loaded in both the main game and the map menu.
static DEFAULT_MAP: LazyLock<&str> = LazyLock::new(|| {
    "\
2222222222222222222222222222222
2133333333222223333332222223332
2232222223332223232232322223232
2233333223232223232232322223232
2232323223232223232232322222232
2232323223333333232233333333232
2232323222222222232222222222232
2232323333333332233333333332232
2232222222222232222222222232232
2232333333322233333322332232232
2232322232322222232322232232232
2232322232333332232322232232232
2232322232222232232322233332232
2232322233332232232322232232232
2232322222222232232322232232232
2232333333333232232322232232232
2232222222222232232322232232232
2233333332222232232322232232232
2222222232222232232322232232232
2333333333333332232222232233334
2222222222222222222222222222222"
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_default() {
        let map = Map::default();

        assert_eq!(map.key, "Default");
        assert_eq!(map.data.len(), 21);
        assert!(map
            .data
            .first()
            .expect("Map should have at least one row")
            .starts_with("222"));
        assert!(map
            .data
            .get(19)
            .expect("Map should have at least 20 rows")
            .ends_with('4'));
    }

    #[test]
    fn test_map_new_valid_input() {
        let filename = OsString::from("test.labmap");
        let data = "111\n222\n333";

        let map = Map::new(filename, data).expect("Failed to create map");

        assert_eq!(map.key, "test");
        assert_eq!(map.data, vec!["111", "222", "333"]);
    }

    #[test]
    fn test_map_new_single_line() {
        let filename = OsString::from("single.labmap");
        let data = "123456789";

        let map = Map::new(filename, data).expect("Failed to create map");

        assert_eq!(map.key, "single");
        assert_eq!(map.data, vec!["123456789"]);
    }

    #[test]
    fn test_map_new_empty_data() {
        let filename = OsString::from("empty.labmap");
        let data = "";

        let map = Map::new(filename, data).expect("Failed to create map");

        assert_eq!(map.key, "empty");
        assert_eq!(map.data.len(), 0);
    }

    #[test]
    fn test_map_new_missing_extension() {
        let filename = OsString::from("noextension");
        let data = "test";

        let result = Map::new(filename, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_new_wrong_extension() {
        let filename = OsString::from("test.txt");
        let data = "test";

        let result = Map::new(filename, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_new_multiple_extensions() {
        let filename = OsString::from("test.backup.labmap");
        let data = "line1\nline2";

        let map = Map::new(filename, data).expect("Failed to create map");

        assert_eq!(map.key, "test.backup");
        assert_eq!(map.data, vec!["line1", "line2"]);
    }
}
