use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Metadata about a puzzle for the selection screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleListItem {
    pub id: String,
    pub title: String,
    pub difficulty: u8,
    pub category: String,
    pub brief: String,
    pub file_path: PathBuf,
    pub is_locked: bool,
    pub prerequisites: Vec<String>,
}

/// Category grouping for puzzles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleCategory {
    pub name: String,
    pub display_name: String,
    pub puzzles: Vec<PuzzleListItem>,
}

/// Load all available puzzles from a directory
pub fn load_puzzle_list(puzzles_dir: &Path) -> Result<Vec<PuzzleCategory>, String> {
    if !puzzles_dir.exists() {
        return Err(format!("Puzzles directory not found: {:?}", puzzles_dir));
    }

    let mut categories = Vec::new();

    // Read category directories
    let entries = fs::read_dir(puzzles_dir)
        .map_err(|e| format!("Failed to read puzzles directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // Skip non-directories
        if !path.is_dir() {
            continue;
        }

        // Get category name from directory name
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid directory name".to_string())?;

        // Parse category (e.g., "01-basics" -> "01", "basics")
        let parts: Vec<&str> = dir_name.splitn(2, '-').collect();
        if parts.len() != 2 {
            continue; // Skip invalid directory names
        }

        let category_name = parts[1].to_string();
        let display_name = category_display_name(&category_name);

        // Load puzzles from this category
        let mut puzzles = Vec::new();
        let puzzle_entries = fs::read_dir(&path)
            .map_err(|e| format!("Failed to read category directory: {}", e))?;

        for puzzle_entry in puzzle_entries {
            let puzzle_entry = puzzle_entry
                .map_err(|e| format!("Failed to read puzzle entry: {}", e))?;
            let puzzle_path = puzzle_entry.path();

            // Only process .toml files
            if puzzle_path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            // Load puzzle metadata
            match load_puzzle_metadata(&puzzle_path) {
                Ok(item) => puzzles.push(item),
                Err(e) => eprintln!("Warning: Failed to load puzzle {:?}: {}", puzzle_path, e),
            }
        }

        // Sort puzzles by filename
        puzzles.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        if !puzzles.is_empty() {
            categories.push(PuzzleCategory {
                name: category_name,
                display_name,
                puzzles,
            });
        }
    }

    // Sort categories by directory name
    categories.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(categories)
}

/// Load puzzle metadata from a TOML file
fn load_puzzle_metadata(path: &Path) -> Result<PuzzleListItem, String> {
    use super::Puzzle;

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read puzzle file: {}", e))?;

    let puzzle: Puzzle = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse puzzle TOML: {}", e))?;

    Ok(PuzzleListItem {
        id: puzzle.metadata.id.clone(),
        title: puzzle.metadata.title.clone(),
        difficulty: puzzle.metadata.difficulty,
        category: puzzle.metadata.category.name().to_string(),
        brief: puzzle.description.brief.clone(),
        file_path: path.to_path_buf(),
        is_locked: false, // Will be determined by game state
        prerequisites: puzzle.metadata.prerequisites.clone(),
    })
}

/// Convert category name to display name
fn category_display_name(name: &str) -> String {
    match name {
        "basics" => "Basics".to_string(),
        "control-flow" => "Control Flow".to_string(),
        "crackmes" => "Crackmes".to_string(),
        "strings" => "Strings".to_string(),
        "arrays" => "Arrays".to_string(),
        "functions" => "Functions".to_string(),
        "advanced" => "Advanced".to_string(),
        _ => {
            // Capitalize first letter
            let mut chars = name.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_display_name() {
        assert_eq!(category_display_name("basics"), "Basics");
        assert_eq!(category_display_name("control-flow"), "Control Flow");
        assert_eq!(category_display_name("crackmes"), "Crackmes");
    }
}
