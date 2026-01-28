use serde::{Deserialize, Serialize};

/// Puzzle difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Tutorial = 0,
    Beginner = 1,
    Easy = 2,
    Medium = 3,
    Hard = 4,
    Expert = 5,
}

impl Difficulty {
    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Tutorial => "Tutorial",
            Difficulty::Beginner => "Beginner",
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Expert => "Expert",
        }
    }

    pub fn from_level(level: u8) -> Self {
        match level {
            0 => Difficulty::Tutorial,
            1 => Difficulty::Beginner,
            2 => Difficulty::Easy,
            3 => Difficulty::Medium,
            4 => Difficulty::Hard,
            _ => Difficulty::Expert,
        }
    }
}

/// Puzzle category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Patching,
    Crackme,
    Keygen,
    Reconstruction,
    AntiDebug,
    Custom(String),
}

impl Category {
    pub fn name(&self) -> &str {
        match self {
            Category::Patching => "Patching",
            Category::Crackme => "Crackme",
            Category::Keygen => "Keygen",
            Category::Reconstruction => "Reconstruction",
            Category::AntiDebug => "Anti-Debug",
            Category::Custom(name) => name,
        }
    }
}

/// Puzzle metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleMetadata {
    /// Unique puzzle identifier
    pub id: String,

    /// Display title
    pub title: String,

    /// Difficulty level (1-5)
    pub difficulty: u8,

    /// Category of challenge
    pub category: Category,

    /// Optional tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Estimated time to solve in minutes (optional)
    #[serde(default)]
    pub estimated_time_minutes: Option<u32>,

    /// Prerequisite puzzle IDs
    #[serde(default)]
    pub prerequisites: Vec<String>,
}

/// Puzzle description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleDescription {
    /// Brief one-liner description
    pub brief: String,

    /// Detailed description/story
    pub detailed: String,
}

/// Initial register values
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegisterSetup {
    #[serde(default)]
    pub eax: Option<u32>,
    #[serde(default)]
    pub ebx: Option<u32>,
    #[serde(default)]
    pub ecx: Option<u32>,
    #[serde(default)]
    pub edx: Option<u32>,
    #[serde(default)]
    pub esi: Option<u32>,
    #[serde(default)]
    pub edi: Option<u32>,
    #[serde(default)]
    pub ebp: Option<u32>,
    #[serde(default)]
    pub esp: Option<u32>,
}

/// Code section configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSetup {
    /// Machine code bytes as hex string
    pub bytes: String,

    /// Entry point address (relative to code_start)
    #[serde(default)]
    pub entry_point: u32,
}

/// Data section configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSetup {
    /// Data bytes as hex string
    pub bytes: String,
}

/// Puzzle setup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleSetup {
    /// Total memory size
    #[serde(default = "default_memory_size")]
    pub memory_size: usize,

    /// Where code section starts
    #[serde(default = "default_code_start")]
    pub code_start: u32,

    /// Where data section starts
    #[serde(default = "default_data_start")]
    pub data_start: u32,

    /// Where stack starts
    #[serde(default = "default_stack_start")]
    pub stack_start: u32,

    /// Initial register values
    #[serde(default)]
    pub registers: RegisterSetup,

    /// Code section
    pub code: CodeSetup,

    /// Data section (optional)
    #[serde(default)]
    pub data: Option<DataSetup>,
}

fn default_memory_size() -> usize {
    0x10000 // 64KB
}

fn default_code_start() -> u32 {
    0x1000
}

fn default_data_start() -> u32 {
    0x2000
}

fn default_stack_start() -> u32 {
    0x3000
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleValidation {
    /// Type of validation
    #[serde(rename = "type")]
    pub validation_type: String,

    /// Register to check (for register_value type)
    #[serde(default)]
    pub register: Option<String>,

    /// Expected value (for register_value type)
    #[serde(default)]
    pub expected: Option<u32>,

    /// Memory address to check (for memory_value type)
    #[serde(default)]
    pub address: Option<u32>,

    /// Expected bytes (for memory_value type)
    #[serde(default)]
    pub expected_bytes: Option<Vec<u8>>,

    /// Sub-conditions for compound validation
    #[serde(default)]
    pub conditions: Vec<PuzzleValidation>,
}

/// Hints for the puzzle
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PuzzleHints {
    #[serde(default)]
    pub level1: Option<String>,
    #[serde(default)]
    pub level2: Option<String>,
    #[serde(default)]
    pub level3: Option<String>,
}

impl PuzzleHints {
    pub fn get_hint(&self, level: usize) -> Option<&str> {
        match level {
            1 => self.level1.as_deref(),
            2 => self.level2.as_deref(),
            3 => self.level3.as_deref(),
            _ => None,
        }
    }

    pub fn hint_count(&self) -> usize {
        let mut count = 0;
        if self.level1.is_some() { count += 1; }
        if self.level2.is_some() { count += 1; }
        if self.level3.is_some() { count += 1; }
        count
    }
}

/// A complete puzzle definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    /// Puzzle metadata
    pub metadata: PuzzleMetadata,

    /// Description text
    pub description: PuzzleDescription,

    /// Initial setup
    pub setup: PuzzleSetup,

    /// Validation rules
    pub validation: PuzzleValidation,

    /// Hints
    #[serde(default)]
    pub hints: PuzzleHints,
}

impl Puzzle {
    /// Get the entry point address
    pub fn entry_point(&self) -> u32 {
        self.setup.code_start + self.setup.code.entry_point
    }

    /// Parse hex string to bytes
    pub fn parse_hex(hex: &str) -> Result<Vec<u8>, String> {
        let hex = hex.replace(' ', "").replace('\n', "").replace('\r', "");

        if hex.len() % 2 != 0 {
            return Err("Hex string must have even length".to_string());
        }

        let mut bytes = Vec::with_capacity(hex.len() / 2);
        for i in (0..hex.len()).step_by(2) {
            let byte = u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {}: {}", i, e))?;
            bytes.push(byte);
        }

        Ok(bytes)
    }

    /// Get code bytes
    pub fn code_bytes(&self) -> Result<Vec<u8>, String> {
        Self::parse_hex(&self.setup.code.bytes)
    }

    /// Get data bytes
    pub fn data_bytes(&self) -> Result<Option<Vec<u8>>, String> {
        if let Some(ref data) = self.setup.data {
            Ok(Some(Self::parse_hex(&data.bytes)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        assert_eq!(Puzzle::parse_hex("90").unwrap(), vec![0x90]);
        assert_eq!(Puzzle::parse_hex("90 90").unwrap(), vec![0x90, 0x90]);
        assert_eq!(
            Puzzle::parse_hex("DEADBEEF").unwrap(),
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        );
    }

    #[test]
    fn test_difficulty_ordering() {
        assert!(Difficulty::Tutorial < Difficulty::Beginner);
        assert!(Difficulty::Beginner < Difficulty::Expert);
    }
}
