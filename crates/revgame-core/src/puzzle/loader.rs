use super::Puzzle;

/// Load a puzzle from TOML string
pub fn load_puzzle(toml_content: &str) -> Result<Puzzle, String> {
    toml::from_str(toml_content).map_err(|e| format!("Failed to parse puzzle: {}", e))
}

/// Load a puzzle from file path
#[cfg(not(target_arch = "wasm32"))]
pub fn load_puzzle_from_file(path: &std::path::Path) -> Result<Puzzle, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read puzzle file: {}", e))?;
    load_puzzle(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_basic_puzzle() {
        let toml = r#"
[metadata]
id = "test-001"
title = "Test Puzzle"
difficulty = 1
category = "patching"

[description]
brief = "A test puzzle"
detailed = "This is a detailed description."

[setup]
memory_size = 4096
code_start = 0x1000
data_start = 0x2000
stack_start = 0x3000

[setup.code]
bytes = "B8 42 00 00 00 F4"
entry_point = 0

[validation]
type = "register_value"
register = "eax"
expected = 0x42
"#;

        let puzzle = load_puzzle(toml).unwrap();
        assert_eq!(puzzle.metadata.id, "test-001");
        assert_eq!(puzzle.metadata.title, "Test Puzzle");
        assert_eq!(puzzle.metadata.difficulty, 1);
        assert_eq!(puzzle.entry_point(), 0x1000);
    }
}
