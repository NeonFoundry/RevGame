use std::collections::HashSet;
use serde::{Deserialize, Serialize};

use crate::puzzle::Puzzle;
use super::achievements::{AchievementId, AchievementTracker};

/// Game state tracking progress and current puzzle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// IDs of completed puzzles
    pub completed_puzzles: HashSet<String>,

    /// ID of current puzzle (if any)
    pub current_puzzle_id: Option<String>,

    /// Number of hints used for current puzzle
    pub hints_used: usize,

    /// Total hints used across all puzzles
    pub total_hints_used: usize,

    /// Maximum difficulty completed
    pub max_difficulty_completed: u8,

    /// Achievement tracker
    pub achievements: AchievementTracker,

    /// Number of patches made in current puzzle
    pub patches_made: usize,

    /// Start time of current puzzle (unix timestamp)
    pub puzzle_start_time: Option<u64>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            completed_puzzles: HashSet::new(),
            current_puzzle_id: None,
            hints_used: 0,
            total_hints_used: 0,
            max_difficulty_completed: 0,
            achievements: AchievementTracker::new(),
            patches_made: 0,
            puzzle_start_time: None,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a puzzle as completed
    pub fn complete_puzzle(&mut self, puzzle_id: &str, difficulty: u8) -> Vec<AchievementId> {
        self.completed_puzzles.insert(puzzle_id.to_string());
        self.total_hints_used += self.hints_used;

        // Calculate elapsed time
        let elapsed = self.puzzle_start_time
            .and_then(|start| {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs().saturating_sub(start))
            });

        // Record in achievement tracker
        let achievements = self.achievements.record_completion(
            puzzle_id,
            self.hints_used,
            self.patches_made,
            elapsed,
        );

        // Reset puzzle-specific state
        self.hints_used = 0;
        self.patches_made = 0;
        self.current_puzzle_id = None;
        self.puzzle_start_time = None;

        if difficulty > self.max_difficulty_completed {
            self.max_difficulty_completed = difficulty;
        }

        achievements
    }

    /// Start a new puzzle
    pub fn start_puzzle(&mut self, puzzle_id: &str) {
        self.current_puzzle_id = Some(puzzle_id.to_string());
        self.hints_used = 0;
        self.patches_made = 0;

        // Record start time
        use std::time::{SystemTime, UNIX_EPOCH};
        self.puzzle_start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());
    }

    /// Use a hint
    pub fn use_hint(&mut self) {
        self.hints_used += 1;
    }

    /// Record a patch
    pub fn record_patch(&mut self) {
        self.patches_made += 1;
    }

    /// Record an undo
    pub fn record_undo(&mut self) -> Option<AchievementId> {
        self.achievements.record_undo()
    }

    /// Get newly unlocked achievements
    pub fn get_new_achievements(&self) -> Vec<AchievementId> {
        // This would return recently unlocked achievements
        // For now, just return empty vec (achievement notifications are handled in complete_puzzle)
        Vec::new()
    }

    /// Check if a puzzle is completed
    pub fn is_completed(&self, puzzle_id: &str) -> bool {
        self.completed_puzzles.contains(puzzle_id)
    }

    /// Get completion count
    pub fn completion_count(&self) -> usize {
        self.completed_puzzles.len()
    }

    /// Check if puzzle prerequisites are met
    pub fn prerequisites_met(&self, puzzle: &Puzzle) -> bool {
        puzzle
            .metadata
            .prerequisites
            .iter()
            .all(|prereq| self.completed_puzzles.contains(prereq))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state() {
        let mut state = GameState::new();

        assert!(!state.is_completed("test-001"));

        state.start_puzzle("test-001");
        assert_eq!(state.current_puzzle_id, Some("test-001".to_string()));

        state.use_hint();
        state.use_hint();
        assert_eq!(state.hints_used, 2);

        state.complete_puzzle("test-001", 1);
        assert!(state.is_completed("test-001"));
        assert_eq!(state.total_hints_used, 2);
        assert_eq!(state.hints_used, 0);
        assert_eq!(state.current_puzzle_id, None);
    }
}
