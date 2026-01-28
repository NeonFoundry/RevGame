mod cpu;
mod memory;
mod flags;
mod instructions;
mod decoder;

pub use cpu::{CpuState, Registers, Register};
pub use memory::{Memory, MemoryRegion, Permissions, MemoryError};
pub use flags::Eflags;
pub use instructions::{Executor, ExecutionResult};
pub use decoder::{DisassemblyLine, Disassembler};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error("Unsupported instruction: {0}")]
    UnsupportedInstruction(String),

    #[error("Invalid operand: {0}")]
    InvalidOperand(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("CPU halted")]
    Halted,

    #[error("Execution limit exceeded")]
    ExecutionLimitExceeded,
}
