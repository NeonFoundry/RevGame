/// Save/load game progress
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use super::GameState;

/// Manages save/load operations
pub struct SaveManager {
    save_dir: PathBuf,
}

impl SaveManager {
    /// Create a new save manager
    pub fn new() -> Result<Self, String> {
        let save_dir = Self::get_save_directory()?;

        // Create save directory if it doesn't exist
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir)
                .map_err(|e| format!("Failed to create save directory: {}", e))?;
        }

        Ok(Self { save_dir })
    }

    /// Get the save directory path
    fn get_save_directory() -> Result<PathBuf, String> {
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "HOME environment variable not set".to_string())?;
            Ok(PathBuf::from(home).join(".local/share/revgame"))
        }

        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| "APPDATA environment variable not set".to_string())?;
            Ok(PathBuf::from(appdata).join("RevGame"))
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "HOME environment variable not set".to_string())?;
            Ok(PathBuf::from(home).join("Library/Application Support/RevGame"))
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Ok(PathBuf::from(".revgame"))
        }
    }

    /// Save game state
    pub fn save(&self, game_state: &GameState, slot: &str) -> Result<(), String> {
        let file_path = self.save_dir.join(format!("save_{}.json", slot));

        let json = serde_json::to_string_pretty(game_state)
            .map_err(|e| format!("Failed to serialize game state: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(())
    }

    /// Load game state
    pub fn load(&self, slot: &str) -> Result<GameState, String> {
        let file_path = self.save_dir.join(format!("save_{}.json", slot));

        if !file_path.exists() {
            return Err("Save file not found".to_string());
        }

        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read save file: {}", e))?;

        let game_state: GameState = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize save file: {}", e))?;

        Ok(game_state)
    }

    /// Check if a save exists
    pub fn save_exists(&self, slot: &str) -> bool {
        self.save_dir.join(format!("save_{}.json", slot)).exists()
    }

    /// List all save slots
    pub fn list_saves(&self) -> Result<Vec<String>, String> {
        let mut saves = Vec::new();

        let entries = fs::read_dir(&self.save_dir)
            .map_err(|e| format!("Failed to read save directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if let Some(filename) = path.file_name() {
                if let Some(name) = filename.to_str() {
                    if name.starts_with("save_") && name.ends_with(".json") {
                        let slot = name.trim_start_matches("save_").trim_end_matches(".json");
                        saves.push(slot.to_string());
                    }
                }
            }
        }

        Ok(saves)
    }

    /// Delete a save
    pub fn delete_save(&self, slot: &str) -> Result<(), String> {
        let file_path = self.save_dir.join(format!("save_{}.json", slot));

        if !file_path.exists() {
            return Err("Save file not found".to_string());
        }

        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete save file: {}", e))?;

        Ok(())
    }

    /// Get save file metadata
    pub fn get_save_info(&self, slot: &str) -> Result<SaveInfo, String> {
        let file_path = self.save_dir.join(format!("save_{}.json", slot));

        if !file_path.exists() {
            return Err("Save file not found".to_string());
        }

        let metadata = fs::metadata(&file_path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;

        let modified = metadata.modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))?;

        let game_state = self.load(slot)?;

        Ok(SaveInfo {
            slot: slot.to_string(),
            modified_time: modified,
            puzzles_completed: game_state.completed_puzzles.len(),
            total_achievements: game_state.achievements.unlocked.len(),
            total_points: game_state.achievements.total_points,
        })
    }
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            save_dir: PathBuf::from(".revgame"),
        })
    }
}

/// Information about a save file
#[derive(Debug, Clone)]
pub struct SaveInfo {
    pub slot: String,
    pub modified_time: std::time::SystemTime,
    pub puzzles_completed: usize,
    pub total_achievements: usize,
    pub total_points: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load() {
        let manager = SaveManager::new().unwrap();
        let game_state = GameState::new();

        // Save
        manager.save(&game_state, "test").unwrap();

        // Load
        let loaded = manager.load("test").unwrap();
        assert_eq!(loaded.completion_count(), 0);

        // Cleanup
        manager.delete_save("test").ok();
    }

    #[test]
    fn test_save_exists() {
        let manager = SaveManager::new().unwrap();
        let game_state = GameState::new();

        assert!(!manager.save_exists("test2"));

        manager.save(&game_state, "test2").unwrap();
        assert!(manager.save_exists("test2"));

        manager.delete_save("test2").ok();
    }
}
