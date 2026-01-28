/// History tracking for undo/redo functionality
use std::collections::VecDeque;

/// A single memory modification action
#[derive(Debug, Clone)]
pub struct MemoryPatch {
    pub address: u32,
    pub old_bytes: Vec<u8>,
    pub new_bytes: Vec<u8>,
}

impl MemoryPatch {
    pub fn new(address: u32, old_bytes: Vec<u8>, new_bytes: Vec<u8>) -> Self {
        Self {
            address,
            old_bytes,
            new_bytes,
        }
    }

    /// Create the inverse patch (for undo)
    pub fn inverse(&self) -> Self {
        Self {
            address: self.address,
            old_bytes: self.new_bytes.clone(),
            new_bytes: self.old_bytes.clone(),
        }
    }
}

/// Manages undo/redo history
#[derive(Debug)]
pub struct History {
    undo_stack: VecDeque<MemoryPatch>,
    redo_stack: VecDeque<MemoryPatch>,
    max_history: usize,
}

impl History {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history,
        }
    }

    /// Record a new patch (clears redo stack)
    pub fn record(&mut self, patch: MemoryPatch) {
        self.undo_stack.push_back(patch);
        self.redo_stack.clear();

        // Limit history size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.pop_front();
        }
    }

    /// Get the next undo action
    pub fn undo(&mut self) -> Option<MemoryPatch> {
        if let Some(patch) = self.undo_stack.pop_back() {
            let inverse = patch.inverse();
            self.redo_stack.push_back(patch);
            Some(inverse)
        } else {
            None
        }
    }

    /// Get the next redo action
    pub fn redo(&mut self) -> Option<MemoryPatch> {
        if let Some(patch) = self.redo_stack.pop_back() {
            let forward = patch.clone();
            self.undo_stack.push_back(patch);
            Some(forward)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of undo actions available
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo actions available
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_undo_redo() {
        let mut history = History::new(10);

        // Record a patch
        let patch1 = MemoryPatch::new(0x1000, vec![0x90], vec![0xEB]);
        history.record(patch1);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        // Undo
        let undo = history.undo().unwrap();
        assert_eq!(undo.address, 0x1000);
        assert_eq!(undo.new_bytes, vec![0x90]); // Restore old value

        assert!(!history.can_undo());
        assert!(history.can_redo());

        // Redo
        let redo = history.redo().unwrap();
        assert_eq!(redo.address, 0x1000);
        assert_eq!(redo.new_bytes, vec![0xEB]); // Apply new value again

        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_clear_redo_on_new_action() {
        let mut history = History::new(10);

        // Record and undo
        history.record(MemoryPatch::new(0x1000, vec![0x90], vec![0xEB]));
        history.undo();

        assert!(history.can_redo());

        // New action should clear redo stack
        history.record(MemoryPatch::new(0x1001, vec![0x00], vec![0xFF]));

        assert!(!history.can_redo());
        assert!(history.can_undo());
    }

    #[test]
    fn test_max_history_limit() {
        let mut history = History::new(3);

        // Add 4 patches
        for i in 0..4 {
            history.record(MemoryPatch::new(0x1000 + i, vec![0x00], vec![0xFF]));
        }

        // Should only keep last 3
        assert_eq!(history.undo_count(), 3);
    }
}
