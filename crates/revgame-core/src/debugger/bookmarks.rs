use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A bookmark at a specific memory address with optional note
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bookmark {
    pub address: u32,
    pub note: String,
    pub created_at: u64, // Unix timestamp
}

impl Bookmark {
    pub fn new(address: u32, note: impl Into<String>) -> Self {
        Self {
            address,
            note: note.into(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Manages bookmarks for memory addresses
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkManager {
    bookmarks: BTreeMap<u32, Bookmark>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a bookmark at an address
    pub fn add(&mut self, address: u32, note: impl Into<String>) -> bool {
        let bookmark = Bookmark::new(address, note);
        self.bookmarks.insert(address, bookmark).is_none()
    }

    /// Remove a bookmark at an address
    pub fn remove(&mut self, address: u32) -> bool {
        self.bookmarks.remove(&address).is_some()
    }

    /// Check if an address has a bookmark
    pub fn has_bookmark(&self, address: u32) -> bool {
        self.bookmarks.contains_key(&address)
    }

    /// Get a bookmark at an address
    pub fn get(&self, address: u32) -> Option<&Bookmark> {
        self.bookmarks.get(&address)
    }

    /// Get a mutable bookmark at an address
    pub fn get_mut(&mut self, address: u32) -> Option<&mut Bookmark> {
        self.bookmarks.get_mut(&address)
    }

    /// Update the note for a bookmark
    pub fn update_note(&mut self, address: u32, note: impl Into<String>) -> bool {
        if let Some(bookmark) = self.bookmarks.get_mut(&address) {
            bookmark.note = note.into();
            true
        } else {
            false
        }
    }

    /// Get all bookmarks sorted by address
    pub fn list(&self) -> Vec<&Bookmark> {
        self.bookmarks.values().collect()
    }

    /// Get all bookmark addresses sorted
    pub fn addresses(&self) -> Vec<u32> {
        self.bookmarks.keys().copied().collect()
    }

    /// Get the number of bookmarks
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Clear all bookmarks
    pub fn clear(&mut self) {
        self.bookmarks.clear();
    }

    /// Get the next bookmark after an address
    pub fn next_after(&self, address: u32) -> Option<u32> {
        self.bookmarks
            .range((address + 1)..)
            .next()
            .map(|(&addr, _)| addr)
    }

    /// Get the previous bookmark before an address
    pub fn prev_before(&self, address: u32) -> Option<u32> {
        self.bookmarks
            .range(..address)
            .next_back()
            .map(|(&addr, _)| addr)
    }

    /// Toggle a bookmark (add if not exists, remove if exists)
    pub fn toggle(&mut self, address: u32, default_note: impl Into<String>) -> bool {
        if self.has_bookmark(address) {
            self.remove(address);
            false
        } else {
            self.add(address, default_note);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_bookmark() {
        let mut manager = BookmarkManager::new();

        assert!(manager.add(0x1000, "Entry point"));
        assert_eq!(manager.count(), 1);

        // Adding duplicate returns false
        assert!(!manager.add(0x1000, "Updated note"));
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_remove_bookmark() {
        let mut manager = BookmarkManager::new();

        manager.add(0x1000, "Test");
        assert!(manager.remove(0x1000));
        assert!(!manager.has_bookmark(0x1000));

        // Removing non-existent returns false
        assert!(!manager.remove(0x1000));
    }

    #[test]
    fn test_navigation() {
        let mut manager = BookmarkManager::new();

        manager.add(0x1000, "First");
        manager.add(0x2000, "Second");
        manager.add(0x3000, "Third");

        assert_eq!(manager.next_after(0x1000), Some(0x2000));
        assert_eq!(manager.next_after(0x2000), Some(0x3000));
        assert_eq!(manager.next_after(0x3000), None);

        assert_eq!(manager.prev_before(0x3000), Some(0x2000));
        assert_eq!(manager.prev_before(0x2000), Some(0x1000));
        assert_eq!(manager.prev_before(0x1000), None);
    }

    #[test]
    fn test_toggle() {
        let mut manager = BookmarkManager::new();

        // Toggle on (add)
        assert!(manager.toggle(0x1000, "Test"));
        assert!(manager.has_bookmark(0x1000));

        // Toggle off (remove)
        assert!(!manager.toggle(0x1000, "Test"));
        assert!(!manager.has_bookmark(0x1000));
    }

    #[test]
    fn test_update_note() {
        let mut manager = BookmarkManager::new();

        manager.add(0x1000, "Original");
        assert!(manager.update_note(0x1000, "Updated"));

        let bookmark = manager.get(0x1000).unwrap();
        assert_eq!(bookmark.note, "Updated");

        // Update non-existent returns false
        assert!(!manager.update_note(0x2000, "Test"));
    }
}
