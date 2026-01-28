/// x86 instruction reference manual
use std::collections::HashMap;

/// Information about a single x86 instruction
#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub mnemonic: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub syntax: Vec<&'static str>,
    pub examples: Vec<&'static str>,
    pub flags_affected: &'static str,
    pub category: InstructionCategory,
}

/// Categories of instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionCategory {
    DataMovement,
    Arithmetic,
    Logical,
    ControlFlow,
    Stack,
    Comparison,
    Special,
}

impl InstructionCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DataMovement => "Data Movement",
            Self::Arithmetic => "Arithmetic",
            Self::Logical => "Logical",
            Self::ControlFlow => "Control Flow",
            Self::Stack => "Stack Operations",
            Self::Comparison => "Comparison & Test",
            Self::Special => "Special",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::DataMovement,
            Self::Arithmetic,
            Self::Logical,
            Self::ControlFlow,
            Self::Stack,
            Self::Comparison,
            Self::Special,
        ]
    }
}

/// x86 instruction reference database
pub struct InstructionReference {
    instructions: HashMap<String, InstructionInfo>,
}

impl Default for InstructionReference {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionReference {
    pub fn new() -> Self {
        let mut reference = Self {
            instructions: HashMap::new(),
        };
        reference.populate();
        reference
    }

    /// Look up an instruction by mnemonic
    pub fn lookup(&self, mnemonic: &str) -> Option<&InstructionInfo> {
        self.instructions.get(&mnemonic.to_uppercase())
    }

    /// Get all instructions in a category
    pub fn by_category(&self, category: InstructionCategory) -> Vec<&InstructionInfo> {
        self.instructions
            .values()
            .filter(|info| info.category == category)
            .collect()
    }

    /// Search for instructions by keyword
    pub fn search(&self, query: &str) -> Vec<&InstructionInfo> {
        let query_lower = query.to_lowercase();
        self.instructions
            .values()
            .filter(|info| {
                info.mnemonic.to_lowercase().contains(&query_lower)
                    || info.name.to_lowercase().contains(&query_lower)
                    || info.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get all mnemonics (sorted)
    pub fn all_mnemonics(&self) -> Vec<String> {
        let mut mnemonics: Vec<String> = self.instructions.keys().cloned().collect();
        mnemonics.sort();
        mnemonics
    }

    /// Populate the reference with instruction data
    fn populate(&mut self) {
        // Data Movement
        self.add(InstructionInfo {
            mnemonic: "MOV",
            name: "Move",
            description: "Copy data from source to destination",
            syntax: vec!["MOV dest, src"],
            examples: vec![
                "MOV EAX, EBX     ; Copy EBX to EAX",
                "MOV EAX, 0x42    ; Load immediate value",
                "MOV [0x1000], EAX ; Store to memory",
            ],
            flags_affected: "None",
            category: InstructionCategory::DataMovement,
        });

        // Arithmetic
        self.add(InstructionInfo {
            mnemonic: "ADD",
            name: "Add",
            description: "Add source to destination",
            syntax: vec!["ADD dest, src"],
            examples: vec![
                "ADD EAX, EBX     ; EAX = EAX + EBX",
                "ADD EAX, 10      ; EAX = EAX + 10",
            ],
            flags_affected: "CF, OF, SF, ZF, AF, PF",
            category: InstructionCategory::Arithmetic,
        });

        self.add(InstructionInfo {
            mnemonic: "SUB",
            name: "Subtract",
            description: "Subtract source from destination",
            syntax: vec!["SUB dest, src"],
            examples: vec![
                "SUB EAX, EBX     ; EAX = EAX - EBX",
                "SUB EAX, 5       ; EAX = EAX - 5",
            ],
            flags_affected: "CF, OF, SF, ZF, AF, PF",
            category: InstructionCategory::Arithmetic,
        });

        self.add(InstructionInfo {
            mnemonic: "INC",
            name: "Increment",
            description: "Increase value by 1",
            syntax: vec!["INC dest"],
            examples: vec!["INC EAX          ; EAX = EAX + 1"],
            flags_affected: "OF, SF, ZF, AF, PF (not CF)",
            category: InstructionCategory::Arithmetic,
        });

        self.add(InstructionInfo {
            mnemonic: "DEC",
            name: "Decrement",
            description: "Decrease value by 1",
            syntax: vec!["DEC dest"],
            examples: vec!["DEC ECX          ; ECX = ECX - 1"],
            flags_affected: "OF, SF, ZF, AF, PF (not CF)",
            category: InstructionCategory::Arithmetic,
        });

        self.add(InstructionInfo {
            mnemonic: "NEG",
            name: "Negate",
            description: "Two's complement negation",
            syntax: vec!["NEG dest"],
            examples: vec!["NEG EAX          ; EAX = -EAX"],
            flags_affected: "CF, OF, SF, ZF, AF, PF",
            category: InstructionCategory::Arithmetic,
        });

        self.add(InstructionInfo {
            mnemonic: "IMUL",
            name: "Signed Multiply",
            description: "Signed integer multiplication",
            syntax: vec!["IMUL dest, src"],
            examples: vec!["IMUL EAX, EBX    ; EAX = EAX * EBX"],
            flags_affected: "CF, OF (SF, ZF, AF, PF undefined)",
            category: InstructionCategory::Arithmetic,
        });

        // Logical
        self.add(InstructionInfo {
            mnemonic: "AND",
            name: "Logical AND",
            description: "Bitwise AND operation",
            syntax: vec!["AND dest, src"],
            examples: vec![
                "AND EAX, 0xFF    ; Mask low byte",
                "AND EAX, EBX     ; EAX = EAX & EBX",
            ],
            flags_affected: "CF=0, OF=0, SF, ZF, PF",
            category: InstructionCategory::Logical,
        });

        self.add(InstructionInfo {
            mnemonic: "OR",
            name: "Logical OR",
            description: "Bitwise OR operation",
            syntax: vec!["OR dest, src"],
            examples: vec![
                "OR EAX, EBX      ; EAX = EAX | EBX",
                "OR EAX, EAX      ; Test if zero (sets ZF)",
            ],
            flags_affected: "CF=0, OF=0, SF, ZF, PF",
            category: InstructionCategory::Logical,
        });

        self.add(InstructionInfo {
            mnemonic: "XOR",
            name: "Logical XOR",
            description: "Bitwise exclusive OR",
            syntax: vec!["XOR dest, src"],
            examples: vec![
                "XOR EAX, EBX     ; EAX = EAX ^ EBX",
                "XOR EAX, EAX     ; Fast way to zero EAX",
            ],
            flags_affected: "CF=0, OF=0, SF, ZF, PF",
            category: InstructionCategory::Logical,
        });

        self.add(InstructionInfo {
            mnemonic: "NOT",
            name: "Logical NOT",
            description: "One's complement (bitwise inversion)",
            syntax: vec!["NOT dest"],
            examples: vec!["NOT EAX          ; EAX = ~EAX"],
            flags_affected: "None",
            category: InstructionCategory::Logical,
        });

        // Comparison
        self.add(InstructionInfo {
            mnemonic: "CMP",
            name: "Compare",
            description: "Compare by subtraction (sets flags only)",
            syntax: vec!["CMP op1, op2"],
            examples: vec![
                "CMP EAX, EBX     ; Compare EAX with EBX",
                "CMP EAX, 0       ; Check if zero",
            ],
            flags_affected: "CF, OF, SF, ZF, AF, PF",
            category: InstructionCategory::Comparison,
        });

        self.add(InstructionInfo {
            mnemonic: "TEST",
            name: "Logical Compare",
            description: "Bitwise AND (sets flags only)",
            syntax: vec!["TEST op1, op2"],
            examples: vec![
                "TEST EAX, EAX    ; Check if zero",
                "TEST EAX, 1      ; Check if odd",
            ],
            flags_affected: "CF=0, OF=0, SF, ZF, PF",
            category: InstructionCategory::Comparison,
        });

        // Control Flow - Jumps
        self.add(InstructionInfo {
            mnemonic: "JMP",
            name: "Unconditional Jump",
            description: "Jump to address unconditionally",
            syntax: vec!["JMP target"],
            examples: vec!["JMP 0x1000       ; Jump to 0x1000"],
            flags_affected: "None",
            category: InstructionCategory::ControlFlow,
        });

        self.add(InstructionInfo {
            mnemonic: "JE",
            name: "Jump if Equal",
            description: "Jump if ZF=1 (result was zero/equal)",
            syntax: vec!["JE target"],
            examples: vec![
                "CMP EAX, EBX",
                "JE equal         ; Jump if EAX == EBX",
            ],
            flags_affected: "None",
            category: InstructionCategory::ControlFlow,
        });

        self.add(InstructionInfo {
            mnemonic: "JNE",
            name: "Jump if Not Equal",
            description: "Jump if ZF=0 (result was not zero/equal)",
            syntax: vec!["JNE target"],
            examples: vec![
                "CMP EAX, 0",
                "JNE not_zero     ; Jump if EAX != 0",
            ],
            flags_affected: "None",
            category: InstructionCategory::ControlFlow,
        });

        self.add(InstructionInfo {
            mnemonic: "JG",
            name: "Jump if Greater",
            description: "Jump if greater (signed: ZF=0 and SF=OF)",
            syntax: vec!["JG target"],
            examples: vec![
                "CMP EAX, EBX",
                "JG greater       ; Jump if EAX > EBX",
            ],
            flags_affected: "None",
            category: InstructionCategory::ControlFlow,
        });

        self.add(InstructionInfo {
            mnemonic: "JL",
            name: "Jump if Less",
            description: "Jump if less (signed: SF != OF)",
            syntax: vec!["JL target"],
            examples: vec![
                "CMP EAX, EBX",
                "JL less          ; Jump if EAX < EBX",
            ],
            flags_affected: "None",
            category: InstructionCategory::ControlFlow,
        });

        self.add(InstructionInfo {
            mnemonic: "CALL",
            name: "Call Procedure",
            description: "Push return address and jump to function",
            syntax: vec!["CALL target"],
            examples: vec!["CALL 0x1000      ; Call function at 0x1000"],
            flags_affected: "None",
            category: InstructionCategory::ControlFlow,
        });

        self.add(InstructionInfo {
            mnemonic: "RET",
            name: "Return",
            description: "Pop return address and jump to it",
            syntax: vec!["RET", "RET imm16"],
            examples: vec![
                "RET              ; Return from function",
                "RET 8            ; Return and pop 8 bytes",
            ],
            flags_affected: "None",
            category: InstructionCategory::ControlFlow,
        });

        // Stack
        self.add(InstructionInfo {
            mnemonic: "PUSH",
            name: "Push",
            description: "Push value onto stack (ESP decreases)",
            syntax: vec!["PUSH src"],
            examples: vec![
                "PUSH EAX         ; Push EAX onto stack",
                "PUSH 0x42        ; Push immediate",
            ],
            flags_affected: "None",
            category: InstructionCategory::Stack,
        });

        self.add(InstructionInfo {
            mnemonic: "POP",
            name: "Pop",
            description: "Pop value from stack (ESP increases)",
            syntax: vec!["POP dest"],
            examples: vec!["POP EAX          ; Pop top of stack into EAX"],
            flags_affected: "None",
            category: InstructionCategory::Stack,
        });

        self.add(InstructionInfo {
            mnemonic: "PUSHAD",
            name: "Push All",
            description: "Push all general-purpose registers",
            syntax: vec!["PUSHAD"],
            examples: vec!["PUSHAD           ; Push EAX,ECX,EDX,EBX,ESP,EBP,ESI,EDI"],
            flags_affected: "None",
            category: InstructionCategory::Stack,
        });

        self.add(InstructionInfo {
            mnemonic: "POPAD",
            name: "Pop All",
            description: "Pop all general-purpose registers",
            syntax: vec!["POPAD"],
            examples: vec!["POPAD            ; Pop EDI,ESI,EBP,ESP,EBX,EDX,ECX,EAX"],
            flags_affected: "None",
            category: InstructionCategory::Stack,
        });

        // Special
        self.add(InstructionInfo {
            mnemonic: "NOP",
            name: "No Operation",
            description: "Do nothing (one byte: 0x90)",
            syntax: vec!["NOP"],
            examples: vec!["NOP              ; Placeholder/alignment"],
            flags_affected: "None",
            category: InstructionCategory::Special,
        });

        self.add(InstructionInfo {
            mnemonic: "INT",
            name: "Interrupt",
            description: "Software interrupt",
            syntax: vec!["INT imm8"],
            examples: vec![
                "INT 0x80         ; Linux system call",
                "INT 3            ; Debugger breakpoint",
            ],
            flags_affected: "Depends on interrupt",
            category: InstructionCategory::Special,
        });

        self.add(InstructionInfo {
            mnemonic: "HLT",
            name: "Halt",
            description: "Halt the processor",
            syntax: vec!["HLT"],
            examples: vec!["HLT              ; Stop execution"],
            flags_affected: "None",
            category: InstructionCategory::Special,
        });
    }

    fn add(&mut self, info: InstructionInfo) {
        self.instructions.insert(info.mnemonic.to_string(), info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup() {
        let reference = InstructionReference::new();

        let mov = reference.lookup("MOV").unwrap();
        assert_eq!(mov.mnemonic, "MOV");
        assert_eq!(mov.name, "Move");

        let add = reference.lookup("add").unwrap(); // Case insensitive
        assert_eq!(add.mnemonic, "ADD");
    }

    #[test]
    fn test_by_category() {
        let reference = InstructionReference::new();

        let arithmetic = reference.by_category(InstructionCategory::Arithmetic);
        assert!(!arithmetic.is_empty());
        assert!(arithmetic.iter().any(|i| i.mnemonic == "ADD"));
        assert!(arithmetic.iter().any(|i| i.mnemonic == "SUB"));
    }

    #[test]
    fn test_search() {
        let reference = InstructionReference::new();

        let results = reference.search("jump");
        assert!(!results.is_empty());
        assert!(results.iter().any(|i| i.mnemonic == "JMP"));

        let results = reference.search("stack");
        assert!(!results.is_empty());
        assert!(results.iter().any(|i| i.mnemonic == "PUSH"));
    }
}
