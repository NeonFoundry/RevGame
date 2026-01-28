use ratatui::style::Color;

/// Instruction category for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionCategory {
    DataMovement,   // MOV, LEA, XCHG, etc.
    Arithmetic,     // ADD, SUB, MUL, DIV, INC, DEC
    Logic,          // AND, OR, XOR, NOT, TEST
    ControlFlow,    // JMP, Jcc, CALL, RET, LOOP
    Comparison,     // CMP
    Stack,          // PUSH, POP
    Shift,          // SHL, SHR, ROL, ROR, SAL, SAR
    Special,        // NOP, HLT, INT, SYSCALL
    Conditional,    // CMOVcc, SETcc
    Unknown,
}

impl InstructionCategory {
    /// Get the category of an instruction by mnemonic
    pub fn from_mnemonic(mnemonic: &str) -> Self {
        let mnemonic_upper = mnemonic.to_uppercase();
        let base = mnemonic_upper.trim();

        // Data Movement
        if matches!(
            base,
            "MOV" | "MOVZX" | "MOVSX" | "LEA" | "XCHG" | "MOVS" | "MOVSB" | "MOVSW" | "MOVSD"
        ) {
            return Self::DataMovement;
        }

        // Stack operations
        if matches!(base, "PUSH" | "POP" | "PUSHA" | "POPA" | "PUSHF" | "POPF") {
            return Self::Stack;
        }

        // Arithmetic
        if matches!(
            base,
            "ADD"
                | "SUB"
                | "MUL"
                | "IMUL"
                | "DIV"
                | "IDIV"
                | "INC"
                | "DEC"
                | "NEG"
                | "ADC"
                | "SBB"
        ) {
            return Self::Arithmetic;
        }

        // Logic
        if matches!(base, "AND" | "OR" | "XOR" | "NOT" | "TEST") {
            return Self::Logic;
        }

        // Comparison
        if base == "CMP" {
            return Self::Comparison;
        }

        // Shift and Rotate
        if matches!(
            base,
            "SHL" | "SHR" | "SAL" | "SAR" | "ROL" | "ROR" | "RCL" | "RCR"
        ) {
            return Self::Shift;
        }

        // Control Flow (unconditional jumps and calls)
        if matches!(
            base,
            "JMP" | "CALL" | "RET" | "RETN" | "RETF" | "LOOP" | "LOOPE" | "LOOPNE"
        ) {
            return Self::ControlFlow;
        }

        // Conditional jumps and moves
        if base.starts_with("J")
            || base.starts_with("CMOV")
            || base.starts_with("SET")
        {
            return Self::Conditional;
        }

        // Special instructions
        if matches!(
            base,
            "NOP" | "HLT" | "INT" | "INT3" | "SYSCALL" | "SYSENTER" | "SYSEXIT" | "CPUID"
        ) {
            return Self::Special;
        }

        Self::Unknown
    }

    /// Get the color for this instruction category
    pub fn color(&self) -> Color {
        match self {
            Self::DataMovement => Color::Cyan,
            Self::Arithmetic => Color::Green,
            Self::Logic => Color::Yellow,
            Self::ControlFlow => Color::Red,
            Self::Comparison => Color::LightYellow,
            Self::Stack => Color::Magenta,
            Self::Shift => Color::Blue,
            Self::Special => Color::Gray,
            Self::Conditional => Color::LightRed,
            Self::Unknown => Color::White,
        }
    }

    /// Get a lighter version of the color for less important elements
    pub fn light_color(&self) -> Color {
        match self {
            Self::DataMovement => Color::LightCyan,
            Self::Arithmetic => Color::LightGreen,
            Self::Logic => Color::LightYellow,
            Self::ControlFlow => Color::LightRed,
            Self::Comparison => Color::Yellow,
            Self::Stack => Color::LightMagenta,
            Self::Shift => Color::LightBlue,
            Self::Special => Color::DarkGray,
            Self::Conditional => Color::Red,
            Self::Unknown => Color::Gray,
        }
    }

    /// Get the display name for this category
    pub fn name(&self) -> &'static str {
        match self {
            Self::DataMovement => "Data Movement",
            Self::Arithmetic => "Arithmetic",
            Self::Logic => "Logic",
            Self::ControlFlow => "Control Flow",
            Self::Comparison => "Comparison",
            Self::Stack => "Stack",
            Self::Shift => "Shift/Rotate",
            Self::Special => "Special",
            Self::Conditional => "Conditional",
            Self::Unknown => "Unknown",
        }
    }
}

/// Syntax highlighter for disassembly
pub struct SyntaxHighlighter {
    pub enabled: bool,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the color for an instruction mnemonic
    pub fn mnemonic_color(&self, mnemonic: &str) -> Color {
        if !self.enabled {
            return Color::White;
        }

        InstructionCategory::from_mnemonic(mnemonic).color()
    }

    /// Get the color for operands (lighter than mnemonic)
    pub fn operand_color(&self, mnemonic: &str) -> Color {
        if !self.enabled {
            return Color::Gray;
        }

        InstructionCategory::from_mnemonic(mnemonic).light_color()
    }

    /// Get the color for addresses
    pub fn address_color(&self) -> Color {
        if !self.enabled {
            return Color::DarkGray;
        }
        Color::DarkGray
    }

    /// Get the color for bytes (hex representation)
    pub fn bytes_color(&self) -> Color {
        if !self.enabled {
            return Color::Gray;
        }
        Color::Gray
    }

    /// Get the color for comments
    pub fn comment_color(&self) -> Color {
        if !self.enabled {
            return Color::DarkGray;
        }
        Color::DarkGray
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_instructions() {
        assert_eq!(
            InstructionCategory::from_mnemonic("MOV"),
            InstructionCategory::DataMovement
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("ADD"),
            InstructionCategory::Arithmetic
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("JMP"),
            InstructionCategory::ControlFlow
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("JE"),
            InstructionCategory::Conditional
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("PUSH"),
            InstructionCategory::Stack
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("CMP"),
            InstructionCategory::Comparison
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("XOR"),
            InstructionCategory::Logic
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("NOP"),
            InstructionCategory::Special
        );
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(
            InstructionCategory::from_mnemonic("mov"),
            InstructionCategory::DataMovement
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("MOV"),
            InstructionCategory::DataMovement
        );
        assert_eq!(
            InstructionCategory::from_mnemonic("Mov"),
            InstructionCategory::DataMovement
        );
    }
}
