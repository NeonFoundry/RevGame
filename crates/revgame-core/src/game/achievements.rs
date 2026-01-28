/// Achievement tracking system
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Types of achievements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    // Tutorial & Basics
    FirstPatch,
    TutorialComplete,

    // Skill-based
    NoHintsUsed,
    SpeedRunner,      // Complete puzzle in under par time
    Minimalist,       // Minimal number of patches
    OneShot,          // Solve with single patch

    // Categories
    BasicMaster,      // Complete all basic puzzles
    FlowMaster,       // Complete all control-flow puzzles
    CrackmeMaster,    // Complete all crackmes

    // Special
    UndoMaster,       // Use undo 10 times
    Perfectionist,    // All achievements in a category
    Experimenter,     // Try 5+ different approaches (undo/redo)

    // Streaks
    WinStreak3,
    WinStreak5,
    WinStreak10,
}

impl AchievementId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FirstPatch => "First Patch",
            Self::TutorialComplete => "Tutorial Graduate",
            Self::NoHintsUsed => "Self Taught",
            Self::SpeedRunner => "Speed Runner",
            Self::Minimalist => "Minimalist",
            Self::OneShot => "One Shot, One Kill",
            Self::BasicMaster => "Basic Training Complete",
            Self::FlowMaster => "Flow Control Expert",
            Self::CrackmeMaster => "Crackme Champion",
            Self::UndoMaster => "Time Traveler",
            Self::Perfectionist => "Perfectionist",
            Self::Experimenter => "Mad Scientist",
            Self::WinStreak3 => "On a Roll",
            Self::WinStreak5 => "Unstoppable",
            Self::WinStreak10 => "Legendary",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::FirstPatch => "Patch your first instruction",
            Self::TutorialComplete => "Complete the tutorial",
            Self::NoHintsUsed => "Solve a puzzle without using hints",
            Self::SpeedRunner => "Complete a puzzle under par time",
            Self::Minimalist => "Solve with minimal patches",
            Self::OneShot => "Solve a puzzle with a single patch",
            Self::BasicMaster => "Complete all basic puzzles",
            Self::FlowMaster => "Complete all control flow puzzles",
            Self::CrackmeMaster => "Complete all crackme puzzles",
            Self::UndoMaster => "Use undo 10 times",
            Self::Perfectionist => "Earn all achievements in a category",
            Self::Experimenter => "Try 5+ different approaches on one puzzle",
            Self::WinStreak3 => "Solve 3 puzzles in a row",
            Self::WinStreak5 => "Solve 5 puzzles in a row",
            Self::WinStreak10 => "Solve 10 puzzles in a row",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::FirstPatch => "🔧",
            Self::TutorialComplete => "🎓",
            Self::NoHintsUsed => "🧠",
            Self::SpeedRunner => "⚡",
            Self::Minimalist => "✨",
            Self::OneShot => "🎯",
            Self::BasicMaster => "🥉",
            Self::FlowMaster => "🥈",
            Self::CrackmeMaster => "🥇",
            Self::UndoMaster => "⏪",
            Self::Perfectionist => "💎",
            Self::Experimenter => "🔬",
            Self::WinStreak3 => "🔥",
            Self::WinStreak5 => "🔥🔥",
            Self::WinStreak10 => "🔥🔥🔥",
        }
    }

    pub fn points(&self) -> u32 {
        match self {
            Self::FirstPatch => 10,
            Self::TutorialComplete => 50,
            Self::NoHintsUsed => 25,
            Self::SpeedRunner => 50,
            Self::Minimalist => 30,
            Self::OneShot => 100,
            Self::BasicMaster => 200,
            Self::FlowMaster => 300,
            Self::CrackmeMaster => 500,
            Self::UndoMaster => 25,
            Self::Perfectionist => 1000,
            Self::Experimenter => 50,
            Self::WinStreak3 => 50,
            Self::WinStreak5 => 150,
            Self::WinStreak10 => 500,
        }
    }
}

/// Statistics for a puzzle completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleStats {
    pub puzzle_id: String,
    pub completed: bool,
    pub hints_used: usize,
    pub patches_made: usize,
    pub time_seconds: Option<u64>,
    pub attempts: usize,
}

impl PuzzleStats {
    pub fn new(puzzle_id: String) -> Self {
        Self {
            puzzle_id,
            completed: false,
            hints_used: 0,
            patches_made: 0,
            time_seconds: None,
            attempts: 0,
        }
    }
}

/// Achievement tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementTracker {
    /// Unlocked achievements
    pub unlocked: HashSet<AchievementId>,

    /// Total points earned
    pub total_points: u32,

    /// Statistics per puzzle
    pub puzzle_stats: HashMap<String, PuzzleStats>,

    /// Current win streak
    pub current_streak: u32,

    /// Best win streak
    pub best_streak: u32,

    /// Total puzzles completed
    pub total_completed: u32,

    /// Total patches made across all puzzles
    pub total_patches: u32,

    /// Total undos used
    pub total_undos: u32,
}

impl Default for AchievementTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl AchievementTracker {
    pub fn new() -> Self {
        Self {
            unlocked: HashSet::new(),
            total_points: 0,
            puzzle_stats: HashMap::new(),
            current_streak: 0,
            best_streak: 0,
            total_completed: 0,
            total_patches: 0,
            total_undos: 0,
        }
    }

    /// Unlock an achievement
    pub fn unlock(&mut self, achievement: AchievementId) -> bool {
        if self.unlocked.insert(achievement) {
            self.total_points += achievement.points();
            true
        } else {
            false
        }
    }

    /// Check if achievement is unlocked
    pub fn is_unlocked(&self, achievement: AchievementId) -> bool {
        self.unlocked.contains(&achievement)
    }

    /// Record a puzzle completion
    pub fn record_completion(
        &mut self,
        puzzle_id: &str,
        hints_used: usize,
        patches_made: usize,
        time_seconds: Option<u64>,
    ) -> Vec<AchievementId> {
        let mut newly_unlocked = Vec::new();

        // Update stats
        let stats = self.puzzle_stats
            .entry(puzzle_id.to_string())
            .or_insert_with(|| PuzzleStats::new(puzzle_id.to_string()));

        stats.completed = true;
        stats.hints_used += hints_used;
        stats.patches_made = patches_made;
        stats.time_seconds = time_seconds;
        stats.attempts += 1;

        // Update global stats
        self.total_completed += 1;
        self.total_patches += patches_made as u32;
        self.current_streak += 1;
        self.best_streak = self.best_streak.max(self.current_streak);

        // Check for new achievements

        // First patch ever
        if self.total_patches > 0 && self.unlock(AchievementId::FirstPatch) {
            newly_unlocked.push(AchievementId::FirstPatch);
        }

        // No hints used
        if hints_used == 0 && self.unlock(AchievementId::NoHintsUsed) {
            newly_unlocked.push(AchievementId::NoHintsUsed);
        }

        // One shot (single patch)
        if patches_made == 1 && self.unlock(AchievementId::OneShot) {
            newly_unlocked.push(AchievementId::OneShot);
        }

        // Win streaks
        if self.current_streak >= 3 && self.unlock(AchievementId::WinStreak3) {
            newly_unlocked.push(AchievementId::WinStreak3);
        }
        if self.current_streak >= 5 && self.unlock(AchievementId::WinStreak5) {
            newly_unlocked.push(AchievementId::WinStreak5);
        }
        if self.current_streak >= 10 && self.unlock(AchievementId::WinStreak10) {
            newly_unlocked.push(AchievementId::WinStreak10);
        }

        // Category completion checks
        self.check_category_completions(&mut newly_unlocked);

        newly_unlocked
    }

    /// Record a puzzle failure (breaks streak)
    pub fn record_failure(&mut self) {
        self.current_streak = 0;
    }

    /// Record an undo action
    pub fn record_undo(&mut self) -> Option<AchievementId> {
        self.total_undos += 1;

        if self.total_undos >= 10 && self.unlock(AchievementId::UndoMaster) {
            Some(AchievementId::UndoMaster)
        } else {
            None
        }
    }

    /// Check for category completion achievements
    fn check_category_completions(&mut self, newly_unlocked: &mut Vec<AchievementId>) {
        // Check if all puzzles in a category are completed
        let basic_puzzles: Vec<_> = self.puzzle_stats.keys()
            .filter(|id| id.starts_with("basic-"))
            .collect();

        if !basic_puzzles.is_empty() &&
           basic_puzzles.iter().all(|id| self.puzzle_stats[*id].completed) &&
           self.unlock(AchievementId::BasicMaster) {
            newly_unlocked.push(AchievementId::BasicMaster);
        }

        let flow_puzzles: Vec<_> = self.puzzle_stats.keys()
            .filter(|id| id.starts_with("flow-"))
            .collect();

        if !flow_puzzles.is_empty() &&
           flow_puzzles.iter().all(|id| self.puzzle_stats[*id].completed) &&
           self.unlock(AchievementId::FlowMaster) {
            newly_unlocked.push(AchievementId::FlowMaster);
        }

        let crackme_puzzles: Vec<_> = self.puzzle_stats.keys()
            .filter(|id| id.starts_with("crackme-"))
            .collect();

        if !crackme_puzzles.is_empty() &&
           crackme_puzzles.iter().all(|id| self.puzzle_stats[*id].completed) &&
           self.unlock(AchievementId::CrackmeMaster) {
            newly_unlocked.push(AchievementId::CrackmeMaster);
        }
    }

    /// Get progress percentage
    pub fn progress_percentage(&self) -> f32 {
        let total_achievements = 15; // Total number of achievements
        (self.unlocked.len() as f32 / total_achievements as f32) * 100.0
    }

    /// Get all unlocked achievements sorted by points
    pub fn unlocked_sorted(&self) -> Vec<AchievementId> {
        let mut achievements: Vec<_> = self.unlocked.iter().copied().collect();
        achievements.sort_by_key(|a| std::cmp::Reverse(a.points()));
        achievements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlock_achievement() {
        let mut tracker = AchievementTracker::new();

        assert!(!tracker.is_unlocked(AchievementId::FirstPatch));
        assert!(tracker.unlock(AchievementId::FirstPatch));
        assert!(tracker.is_unlocked(AchievementId::FirstPatch));
        assert!(!tracker.unlock(AchievementId::FirstPatch)); // Already unlocked

        assert_eq!(tracker.total_points, AchievementId::FirstPatch.points());
    }

    #[test]
    fn test_puzzle_completion() {
        let mut tracker = AchievementTracker::new();

        let unlocked = tracker.record_completion("basic-001", 0, 1, Some(60));

        assert!(unlocked.contains(&AchievementId::FirstPatch));
        assert!(unlocked.contains(&AchievementId::NoHintsUsed));
        assert!(unlocked.contains(&AchievementId::OneShot));
        assert_eq!(tracker.total_completed, 1);
        assert_eq!(tracker.current_streak, 1);
    }

    #[test]
    fn test_streak_tracking() {
        let mut tracker = AchievementTracker::new();

        tracker.record_completion("basic-001", 1, 2, None);
        tracker.record_completion("basic-002", 1, 2, None);
        tracker.record_completion("basic-003", 1, 2, None);

        assert_eq!(tracker.current_streak, 3);
        assert!(tracker.is_unlocked(AchievementId::WinStreak3));

        tracker.record_failure();
        assert_eq!(tracker.current_streak, 0);
        assert_eq!(tracker.best_streak, 3);
    }

    #[test]
    fn test_undo_tracking() {
        let mut tracker = AchievementTracker::new();

        for _ in 0..9 {
            assert!(tracker.record_undo().is_none());
        }

        let achievement = tracker.record_undo();
        assert_eq!(achievement, Some(AchievementId::UndoMaster));
        assert_eq!(tracker.total_undos, 10);
    }
}
