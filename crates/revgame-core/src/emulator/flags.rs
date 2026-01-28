use serde::{Deserialize, Serialize};

/// x86 EFLAGS register (simplified for educational purposes)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Eflags {
    /// Carry Flag - set on unsigned overflow
    pub cf: bool,
    /// Zero Flag - set when result is zero
    pub zf: bool,
    /// Sign Flag - set when result is negative (high bit set)
    pub sf: bool,
    /// Overflow Flag - set on signed overflow
    pub of: bool,
    /// Parity Flag - set when low byte has even number of 1 bits
    pub pf: bool,
    /// Auxiliary Carry Flag - carry from bit 3 to bit 4
    pub af: bool,
    /// Direction Flag - controls string operation direction
    pub df: bool,
}

impl Eflags {
    /// Create a new Eflags with all flags cleared
    pub fn new() -> Self {
        Self::default()
    }

    /// Update flags based on an arithmetic result (ADD, SUB, etc.)
    pub fn update_arithmetic(&mut self, result: u32, operand1: u32, operand2: u32, is_sub: bool) {
        self.zf = result == 0;
        self.sf = (result as i32) < 0;
        self.pf = Self::compute_parity(result as u8);

        // For subtraction, we compute as if it were addition of negated operand
        let (_, carry) = if is_sub {
            operand1.overflowing_sub(operand2)
        } else {
            operand1.overflowing_add(operand2)
        };
        self.cf = carry;

        // Overflow: sign of result differs from expected
        let sign1 = (operand1 as i32) < 0;
        let sign2 = (operand2 as i32) < 0;
        let sign_result = (result as i32) < 0;

        if is_sub {
            // Overflow on subtraction: positive - negative = negative, or negative - positive = positive
            self.of = (sign1 != sign2) && (sign_result != sign1);
        } else {
            // Overflow on addition: same signs but different result sign
            self.of = (sign1 == sign2) && (sign_result != sign1);
        }

        // Auxiliary flag (BCD operations) - carry from bit 3 to bit 4
        self.af = ((operand1 ^ operand2 ^ result) & 0x10) != 0;
    }

    /// Update flags based on a logical result (AND, OR, XOR)
    pub fn update_logical(&mut self, result: u32) {
        self.zf = result == 0;
        self.sf = (result as i32) < 0;
        self.pf = Self::compute_parity(result as u8);
        self.cf = false;  // Always cleared for logical ops
        self.of = false;  // Always cleared for logical ops
        self.af = false;  // Undefined, we clear it
    }

    /// Update flags for increment operation
    pub fn update_inc(&mut self, result: u32, original: u32) {
        self.zf = result == 0;
        self.sf = (result as i32) < 0;
        self.pf = Self::compute_parity(result as u8);
        // OF set if went from 0x7FFFFFFF to 0x80000000
        self.of = original == 0x7FFFFFFF;
        self.af = (result & 0xF) == 0;
        // CF is not affected by INC
    }

    /// Update flags for decrement operation
    pub fn update_dec(&mut self, result: u32, original: u32) {
        self.zf = result == 0;
        self.sf = (result as i32) < 0;
        self.pf = Self::compute_parity(result as u8);
        // OF set if went from 0x80000000 to 0x7FFFFFFF
        self.of = original == 0x80000000;
        self.af = (original & 0xF) == 0;
        // CF is not affected by DEC
    }

    /// Compute parity of the low byte (true if even number of 1 bits)
    fn compute_parity(byte: u8) -> bool {
        byte.count_ones() % 2 == 0
    }

    /// Get flags as a formatted string for display
    pub fn display(&self) -> String {
        let mut flags = Vec::new();
        if self.cf { flags.push("CF"); }
        if self.zf { flags.push("ZF"); }
        if self.sf { flags.push("SF"); }
        if self.of { flags.push("OF"); }
        if self.pf { flags.push("PF"); }
        if self.af { flags.push("AF"); }
        if self.df { flags.push("DF"); }

        if flags.is_empty() {
            "[ ]".to_string()
        } else {
            format!("[ {} ]", flags.join(" "))
        }
    }

    /// Get flags as a compact bit representation
    pub fn as_u32(&self) -> u32 {
        let mut value = 0u32;
        if self.cf { value |= 1 << 0; }  // bit 0
        if self.pf { value |= 1 << 2; }  // bit 2
        if self.af { value |= 1 << 4; }  // bit 4
        if self.zf { value |= 1 << 6; }  // bit 6
        if self.sf { value |= 1 << 7; }  // bit 7
        if self.df { value |= 1 << 10; } // bit 10
        if self.of { value |= 1 << 11; } // bit 11
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parity() {
        assert!(Eflags::compute_parity(0b00000000)); // 0 ones = even
        assert!(!Eflags::compute_parity(0b00000001)); // 1 one = odd
        assert!(Eflags::compute_parity(0b00000011)); // 2 ones = even
        assert!(!Eflags::compute_parity(0b00000111)); // 3 ones = odd
    }

    #[test]
    fn test_arithmetic_flags() {
        let mut flags = Eflags::new();

        // Test zero flag
        flags.update_arithmetic(0, 5, 5, true); // 5 - 5 = 0
        assert!(flags.zf);

        // Test sign flag
        flags.update_arithmetic(0xFFFFFFFF, 0, 1, true); // 0 - 1 = -1
        assert!(flags.sf);

        // Test carry flag on subtraction
        flags.update_arithmetic(0xFFFFFFFF, 0, 1, true);
        assert!(flags.cf); // Borrow occurred
    }

    #[test]
    fn test_logical_flags() {
        let mut flags = Eflags::new();

        flags.update_logical(0);
        assert!(flags.zf);
        assert!(!flags.cf);
        assert!(!flags.of);

        flags.update_logical(0x80000000);
        assert!(flags.sf);
    }
}
