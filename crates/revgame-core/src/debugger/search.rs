use super::DebuggerError;
use crate::emulator::Memory;

/// Search result containing address and matched data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub address: u32,
    pub data: Vec<u8>,
}

/// Search engine for finding bytes and strings in memory
pub struct MemorySearch;

impl MemorySearch {
    /// Search for a byte pattern in memory
    ///
    /// # Arguments
    /// * `memory` - Memory to search
    /// * `pattern` - Byte pattern to find (e.g., [0x90, 0x90])
    /// * `start_address` - Start searching from this address
    /// * `end_address` - Stop searching at this address
    ///
    /// # Returns
    /// Vec of search results containing addresses where pattern was found
    pub fn search_bytes(
        memory: &Memory,
        pattern: &[u8],
        start_address: u32,
        end_address: u32,
    ) -> Result<Vec<SearchResult>, DebuggerError> {
        if pattern.is_empty() {
            return Err(DebuggerError::InvalidInput(
                "Search pattern cannot be empty".to_string(),
            ));
        }

        if start_address >= end_address {
            return Err(DebuggerError::InvalidInput(
                "Start address must be less than end address".to_string(),
            ));
        }

        let mut results = Vec::new();
        let search_len = (end_address - start_address) as usize;

        // Read the entire search range
        let data = memory.read_bytes(start_address, search_len)?;

        // Search for pattern
        for i in 0..=data.len().saturating_sub(pattern.len()) {
            if &data[i..i + pattern.len()] == pattern {
                results.push(SearchResult {
                    address: start_address + i as u32,
                    data: pattern.to_vec(),
                });
            }
        }

        Ok(results)
    }

    /// Search for ASCII/UTF-8 strings in memory
    ///
    /// # Arguments
    /// * `memory` - Memory to search
    /// * `search_string` - String to find
    /// * `start_address` - Start searching from this address
    /// * `end_address` - Stop searching at this address
    /// * `case_sensitive` - Whether search should be case-sensitive
    ///
    /// # Returns
    /// Vec of search results containing addresses where string was found
    pub fn search_string(
        memory: &Memory,
        search_string: &str,
        start_address: u32,
        end_address: u32,
        case_sensitive: bool,
    ) -> Result<Vec<SearchResult>, DebuggerError> {
        if search_string.is_empty() {
            return Err(DebuggerError::InvalidInput(
                "Search string cannot be empty".to_string(),
            ));
        }

        if start_address >= end_address {
            return Err(DebuggerError::InvalidInput(
                "Start address must be less than end address".to_string(),
            ));
        }

        let pattern_bytes = if case_sensitive {
            search_string.as_bytes()
        } else {
            &search_string.to_lowercase().as_bytes().to_vec()
        };

        let mut results = Vec::new();
        let search_len = (end_address - start_address) as usize;

        // Read the entire search range
        let data = memory.read_bytes(start_address, search_len)?;

        // Convert to lowercase if case-insensitive
        let search_data = if case_sensitive {
            data.clone()
        } else {
            data.iter()
                .map(|&b| if b.is_ascii_alphabetic() { b.to_ascii_lowercase() } else { b })
                .collect::<Vec<_>>()
        };

        // Search for pattern
        for i in 0..=search_data.len().saturating_sub(pattern_bytes.len()) {
            if &search_data[i..i + pattern_bytes.len()] == pattern_bytes {
                // Return the original bytes, not the lowercased ones
                results.push(SearchResult {
                    address: start_address + i as u32,
                    data: data[i..i + pattern_bytes.len()].to_vec(),
                });
            }
        }

        Ok(results)
    }

    /// Search for null-terminated C strings in memory
    ///
    /// # Arguments
    /// * `memory` - Memory to search
    /// * `min_length` - Minimum string length to consider
    /// * `start_address` - Start searching from this address
    /// * `end_address` - Stop searching at this address
    ///
    /// # Returns
    /// Vec of search results containing addresses where strings were found
    pub fn find_strings(
        memory: &Memory,
        min_length: usize,
        start_address: u32,
        end_address: u32,
    ) -> Result<Vec<SearchResult>, DebuggerError> {
        if start_address >= end_address {
            return Err(DebuggerError::InvalidInput(
                "Start address must be less than end address".to_string(),
            ));
        }

        let mut results = Vec::new();
        let search_len = (end_address - start_address) as usize;

        // Read the entire search range
        let data = memory.read_bytes(start_address, search_len)?;

        let mut current_string = Vec::new();
        let mut string_start: Option<u32> = None;

        for (i, &byte) in data.iter().enumerate() {
            if byte == 0 {
                // Null terminator found
                if current_string.len() >= min_length {
                    if let Some(start) = string_start {
                        results.push(SearchResult {
                            address: start,
                            data: current_string.clone(),
                        });
                    }
                }
                current_string.clear();
                string_start = None;
            } else if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                // Valid ASCII character
                if string_start.is_none() {
                    string_start = Some(start_address + i as u32);
                }
                current_string.push(byte);
            } else {
                // Invalid character, reset
                current_string.clear();
                string_start = None;
            }
        }

        Ok(results)
    }

    /// Parse hex string to bytes (e.g., "90 90" -> [0x90, 0x90])
    pub fn parse_hex_pattern(hex_string: &str) -> Result<Vec<u8>, String> {
        let cleaned = hex_string
            .replace(" ", "")
            .replace("0x", "")
            .replace(",", "");

        if cleaned.len() % 2 != 0 {
            return Err("Hex string must have an even number of characters".to_string());
        }

        let mut bytes = Vec::new();
        for i in (0..cleaned.len()).step_by(2) {
            let byte_str = &cleaned[i..i + 2];
            match u8::from_str_radix(byte_str, 16) {
                Ok(byte) => bytes.push(byte),
                Err(_) => return Err(format!("Invalid hex byte: {}", byte_str)),
            }
        }

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_bytes() {
        let mut memory = Memory::new(4096);

        // Write pattern at 0x1000
        memory.write_bytes(0x1000, &[0x90, 0x90, 0x90]).unwrap();

        // Write pattern at 0x2000
        memory.write_bytes(0x2000, &[0x90, 0x90, 0x90]).unwrap();

        let results = MemorySearch::search_bytes(
            &memory,
            &[0x90, 0x90],
            0x0,
            0x3000,
        )
        .unwrap();

        // Should find 4 matches: 0x1000, 0x1001, 0x2000, 0x2001
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].address, 0x1000);
        assert_eq!(results[1].address, 0x1001);
        assert_eq!(results[2].address, 0x2000);
        assert_eq!(results[3].address, 0x2001);
    }

    #[test]
    fn test_search_string() {
        let mut memory = Memory::new(4096);

        // Write "HELLO" at 0x1000
        memory.write_bytes(0x1000, b"HELLO").unwrap();

        // Write "hello" at 0x2000
        memory.write_bytes(0x2000, b"hello").unwrap();

        // Case-sensitive search
        let results = MemorySearch::search_string(
            &memory,
            "HELLO",
            0x0,
            0x3000,
            true,
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, 0x1000);

        // Case-insensitive search
        let results = MemorySearch::search_string(
            &memory,
            "hello",
            0x0,
            0x3000,
            false,
        )
        .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].address, 0x1000);
        assert_eq!(results[1].address, 0x2000);
    }

    #[test]
    fn test_find_strings() {
        let mut memory = Memory::new(4096);

        // Write null-terminated strings
        memory.write_bytes(0x1000, b"Hello\0").unwrap();
        memory.write_bytes(0x2000, b"World!\0").unwrap();
        memory.write_bytes(0x3000, b"Hi\0").unwrap(); // Too short

        let results = MemorySearch::find_strings(
            &memory,
            4, // min length
            0x0,
            0x4000,
        )
        .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].address, 0x1000);
        assert_eq!(results[0].data, b"Hello");
        assert_eq!(results[1].address, 0x2000);
        assert_eq!(results[1].data, b"World!");
    }

    #[test]
    fn test_parse_hex_pattern() {
        assert_eq!(
            MemorySearch::parse_hex_pattern("90 90").unwrap(),
            vec![0x90, 0x90]
        );

        assert_eq!(
            MemorySearch::parse_hex_pattern("0x90 0x90").unwrap(),
            vec![0x90, 0x90]
        );

        assert_eq!(
            MemorySearch::parse_hex_pattern("9090").unwrap(),
            vec![0x90, 0x90]
        );

        assert!(MemorySearch::parse_hex_pattern("9").is_err());
        assert!(MemorySearch::parse_hex_pattern("ZZ").is_err());
    }
}
