use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, IntelFormatter};

/// A single line of disassembly
#[derive(Debug, Clone)]
pub struct DisassemblyLine {
    /// Address of the instruction
    pub address: u32,
    /// Raw bytes of the instruction
    pub bytes: Vec<u8>,
    /// Formatted instruction text (Intel syntax)
    pub text: String,
    /// Length of the instruction in bytes
    pub length: usize,
    /// The decoded instruction (for analysis)
    pub instruction: Instruction,
}

/// Disassemble code from memory
pub struct Disassembler {
    bitness: u32,
    formatter: IntelFormatter,
}

impl Disassembler {
    /// Create a new disassembler (32-bit mode)
    pub fn new() -> Self {
        Self::with_bitness(32)
    }

    /// Create a disassembler with specified bitness (16, 32, or 64)
    pub fn with_bitness(bitness: u32) -> Self {
        let mut formatter = IntelFormatter::new();
        // Configure formatter for readability
        formatter.options_mut().set_uppercase_mnemonics(false);
        formatter.options_mut().set_uppercase_registers(false);
        formatter.options_mut().set_uppercase_keywords(false);
        formatter.options_mut().set_uppercase_hex(true);
        formatter.options_mut().set_hex_prefix("0x");
        formatter.options_mut().set_hex_suffix("");
        formatter.options_mut().set_space_after_operand_separator(true);

        Self { bitness, formatter }
    }

    /// Disassemble a single instruction at the given address
    pub fn disassemble_one(&mut self, bytes: &[u8], address: u32) -> Option<DisassemblyLine> {
        if bytes.is_empty() {
            return None;
        }

        let mut decoder = Decoder::with_ip(
            self.bitness,
            bytes,
            address as u64,
            DecoderOptions::NONE,
        );

        if !decoder.can_decode() {
            return None;
        }

        let instruction = decoder.decode();
        if instruction.is_invalid() {
            return None;
        }

        let length = instruction.len();
        let mut text = String::new();
        self.formatter.format(&instruction, &mut text);

        Some(DisassemblyLine {
            address,
            bytes: bytes[..length].to_vec(),
            text,
            length,
            instruction,
        })
    }

    /// Disassemble multiple instructions starting at address
    pub fn disassemble(&mut self, bytes: &[u8], start_address: u32, count: usize) -> Vec<DisassemblyLine> {
        let mut result = Vec::with_capacity(count);
        let mut decoder = Decoder::with_ip(
            self.bitness,
            bytes,
            start_address as u64,
            DecoderOptions::NONE,
        );

        while decoder.can_decode() && result.len() < count {
            let instruction = decoder.decode();
            if instruction.is_invalid() {
                break;
            }

            let address = instruction.ip() as u32;
            let length = instruction.len();
            let offset = (address - start_address) as usize;

            let mut text = String::new();
            self.formatter.format(&instruction, &mut text);

            result.push(DisassemblyLine {
                address,
                bytes: bytes[offset..offset + length].to_vec(),
                text,
                length,
                instruction,
            });
        }

        result
    }

    /// Format bytes as hex string (e.g., "90 90 CC")
    pub fn format_bytes(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for Disassembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_nop() {
        let mut dis = Disassembler::new();
        let bytes = [0x90]; // NOP
        let line = dis.disassemble_one(&bytes, 0x1000).unwrap();

        assert_eq!(line.address, 0x1000);
        assert_eq!(line.length, 1);
        assert!(line.text.contains("nop"));
    }

    #[test]
    fn test_disassemble_mov_eax_imm() {
        let mut dis = Disassembler::new();
        // MOV EAX, 0x12345678
        let bytes = [0xB8, 0x78, 0x56, 0x34, 0x12];
        let line = dis.disassemble_one(&bytes, 0x1000).unwrap();

        assert_eq!(line.address, 0x1000);
        assert_eq!(line.length, 5);
        assert!(line.text.contains("mov"));
        assert!(line.text.contains("eax"));
    }

    #[test]
    fn test_disassemble_multiple() {
        let mut dis = Disassembler::new();
        // NOP; NOP; RET
        let bytes = [0x90, 0x90, 0xC3];
        let lines = dis.disassemble(&bytes, 0x1000, 10);

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].address, 0x1000);
        assert_eq!(lines[1].address, 0x1001);
        assert_eq!(lines[2].address, 0x1002);
        assert!(lines[2].text.contains("ret"));
    }
}
