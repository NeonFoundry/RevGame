use crate::emulator::{CpuState, Memory, Register};

use super::{Puzzle, PuzzleValidation};

/// Result of puzzle validation
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Puzzle solved successfully
    Success,
    /// Puzzle not yet solved
    Failure(String),
    /// Validation error
    Error(String),
}

impl ValidationResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ValidationResult::Success)
    }
}

/// Validation rule types
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Check register equals value
    RegisterValue { register: Register, expected: u32 },

    /// Check memory location equals bytes
    MemoryValue { address: u32, expected: Vec<u8> },

    /// Program must halt normally
    NormalHalt,

    /// All conditions must pass
    All(Vec<ValidationRule>),

    /// Any condition passes
    Any(Vec<ValidationRule>),
}

/// Puzzle validator
pub struct Validator;

impl Validator {
    /// Validate puzzle completion
    pub fn validate(puzzle: &Puzzle, cpu: &CpuState, memory: &Memory) -> ValidationResult {
        Self::validate_config(&puzzle.validation, cpu, memory)
    }

    /// Validate based on configuration
    fn validate_config(
        config: &PuzzleValidation,
        cpu: &CpuState,
        memory: &Memory,
    ) -> ValidationResult {
        match config.validation_type.as_str() {
            "register_value" => {
                let reg_name = match &config.register {
                    Some(r) => r,
                    None => return ValidationResult::Error("Missing register name".to_string()),
                };

                let expected = match config.expected {
                    Some(v) => v,
                    None => return ValidationResult::Error("Missing expected value".to_string()),
                };

                let register = match Register::from_name(reg_name) {
                    Some(r) => r,
                    None => {
                        return ValidationResult::Error(format!(
                            "Unknown register: {}",
                            reg_name
                        ))
                    }
                };

                let actual = cpu.get_register(register);
                if actual == expected {
                    ValidationResult::Success
                } else {
                    ValidationResult::Failure(format!(
                        "{} = 0x{:08X}, expected 0x{:08X}",
                        register.name(),
                        actual,
                        expected
                    ))
                }
            }

            "memory_value" => {
                let address = match config.address {
                    Some(a) => a,
                    None => return ValidationResult::Error("Missing address".to_string()),
                };

                let expected: &Vec<u8> = match &config.expected_bytes {
                    Some(b) => b,
                    None => return ValidationResult::Error("Missing expected bytes".to_string()),
                };

                match memory.read_bytes(address, expected.len()) {
                    Ok(actual) => {
                        if actual == *expected {
                            ValidationResult::Success
                        } else {
                            ValidationResult::Failure(format!(
                                "Memory at 0x{:08X} = {:02X?}, expected {:02X?}",
                                address, actual, expected
                            ))
                        }
                    }
                    Err(e) => ValidationResult::Error(format!("Memory read error: {}", e)),
                }
            }

            "normal_halt" => {
                if cpu.halted {
                    ValidationResult::Success
                } else {
                    ValidationResult::Failure("Program did not halt".to_string())
                }
            }

            "all" => {
                for condition in &config.conditions {
                    match Self::validate_config(condition, cpu, memory) {
                        ValidationResult::Success => continue,
                        result => return result,
                    }
                }
                ValidationResult::Success
            }

            "any" => {
                let mut last_failure = None;
                for condition in &config.conditions {
                    match Self::validate_config(condition, cpu, memory) {
                        ValidationResult::Success => return ValidationResult::Success,
                        ValidationResult::Failure(msg) => {
                            last_failure = Some(msg);
                        }
                        ValidationResult::Error(e) => return ValidationResult::Error(e),
                    }
                }
                ValidationResult::Failure(
                    last_failure.unwrap_or_else(|| "No conditions matched".to_string()),
                )
            }

            other => ValidationResult::Error(format!("Unknown validation type: {}", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_validation() {
        let mut cpu = CpuState::default();
        cpu.regs.eax = 0x42;

        let memory = Memory::new(0x1000);

        let config = PuzzleValidation {
            validation_type: "register_value".to_string(),
            register: Some("eax".to_string()),
            expected: Some(0x42),
            address: None,
            expected_bytes: None,
            conditions: Vec::new(),
        };

        let result = Validator::validate_config(&config, &cpu, &memory);
        assert!(result.is_success());

        // Test failure
        cpu.regs.eax = 0x41;
        let result = Validator::validate_config(&config, &cpu, &memory);
        assert!(!result.is_success());
    }

    #[test]
    fn test_halt_validation() {
        let mut cpu = CpuState::default();
        let memory = Memory::new(0x1000);

        let config = PuzzleValidation {
            validation_type: "normal_halt".to_string(),
            register: None,
            expected: None,
            address: None,
            expected_bytes: None,
            conditions: Vec::new(),
        };

        // Not halted
        let result = Validator::validate_config(&config, &cpu, &memory);
        assert!(!result.is_success());

        // Halted
        cpu.halted = true;
        let result = Validator::validate_config(&config, &cpu, &memory);
        assert!(result.is_success());
    }
}
