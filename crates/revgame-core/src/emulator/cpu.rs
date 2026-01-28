use serde::{Deserialize, Serialize};
use super::Eflags;

/// Represents which register to access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Register {
    // 32-bit general purpose
    Eax,
    Ebx,
    Ecx,
    Edx,
    Esi,
    Edi,
    Ebp,
    Esp,
    // Instruction pointer
    Eip,
}

impl Register {
    /// Parse register name from string (case-insensitive)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "eax" => Some(Register::Eax),
            "ebx" => Some(Register::Ebx),
            "ecx" => Some(Register::Ecx),
            "edx" => Some(Register::Edx),
            "esi" => Some(Register::Esi),
            "edi" => Some(Register::Edi),
            "ebp" => Some(Register::Ebp),
            "esp" => Some(Register::Esp),
            "eip" => Some(Register::Eip),
            _ => None,
        }
    }

    /// Get the display name of the register
    pub fn name(&self) -> &'static str {
        match self {
            Register::Eax => "EAX",
            Register::Ebx => "EBX",
            Register::Ecx => "ECX",
            Register::Edx => "EDX",
            Register::Esi => "ESI",
            Register::Edi => "EDI",
            Register::Ebp => "EBP",
            Register::Esp => "ESP",
            Register::Eip => "EIP",
        }
    }
}

/// General purpose registers (32-bit mode)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Registers {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
    pub esp: u32,
}

/// Complete CPU state for the emulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuState {
    /// General-purpose registers
    pub regs: Registers,

    /// Instruction pointer
    pub eip: u32,

    /// Flags register
    pub eflags: Eflags,

    /// Whether the CPU has halted
    pub halted: bool,

    /// Optional fault information
    pub fault: Option<CpuFault>,
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            regs: Registers::default(),
            eip: 0,
            eflags: Eflags::default(),
            halted: false,
            fault: None,
        }
    }
}

impl CpuState {
    /// Create a new CPU state with specified entry point and stack pointer
    pub fn new(entry_point: u32, stack_pointer: u32) -> Self {
        let mut state = Self::default();
        state.eip = entry_point;
        state.regs.esp = stack_pointer;
        state
    }

    /// Get the value of a register
    pub fn get_register(&self, reg: Register) -> u32 {
        match reg {
            Register::Eax => self.regs.eax,
            Register::Ebx => self.regs.ebx,
            Register::Ecx => self.regs.ecx,
            Register::Edx => self.regs.edx,
            Register::Esi => self.regs.esi,
            Register::Edi => self.regs.edi,
            Register::Ebp => self.regs.ebp,
            Register::Esp => self.regs.esp,
            Register::Eip => self.eip,
        }
    }

    /// Set the value of a register
    pub fn set_register(&mut self, reg: Register, value: u32) {
        match reg {
            Register::Eax => self.regs.eax = value,
            Register::Ebx => self.regs.ebx = value,
            Register::Ecx => self.regs.ecx = value,
            Register::Edx => self.regs.edx = value,
            Register::Esi => self.regs.esi = value,
            Register::Edi => self.regs.edi = value,
            Register::Ebp => self.regs.ebp = value,
            Register::Esp => self.regs.esp = value,
            Register::Eip => self.eip = value,
        }
    }

    /// Get register value by name (case-insensitive)
    pub fn get_register_by_name(&self, name: &str) -> Option<u32> {
        Register::from_name(name).map(|r| self.get_register(r))
    }

    /// Set register value by name (case-insensitive)
    pub fn set_register_by_name(&mut self, name: &str, value: u32) -> bool {
        if let Some(reg) = Register::from_name(name) {
            self.set_register(reg, value);
            true
        } else {
            false
        }
    }
}

/// CPU fault information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CpuFault {
    /// Invalid opcode
    InvalidOpcode(u32),
    /// Memory access violation
    MemoryFault { address: u32, write: bool },
    /// Division by zero
    DivideError,
    /// Stack overflow/underflow
    StackFault,
    /// General protection fault
    GeneralProtection(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_from_name() {
        assert_eq!(Register::from_name("eax"), Some(Register::Eax));
        assert_eq!(Register::from_name("EAX"), Some(Register::Eax));
        assert_eq!(Register::from_name("Eax"), Some(Register::Eax));
        assert_eq!(Register::from_name("invalid"), None);
    }

    #[test]
    fn test_cpu_state_registers() {
        let mut cpu = CpuState::new(0x1000, 0x3000);

        assert_eq!(cpu.get_register(Register::Eip), 0x1000);
        assert_eq!(cpu.get_register(Register::Esp), 0x3000);

        cpu.set_register(Register::Eax, 0xDEADBEEF);
        assert_eq!(cpu.get_register(Register::Eax), 0xDEADBEEF);

        assert!(cpu.set_register_by_name("ebx", 0x12345678));
        assert_eq!(cpu.get_register_by_name("ebx"), Some(0x12345678));
    }
}
