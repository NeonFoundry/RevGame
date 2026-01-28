mod execution;
mod history;
mod search;
mod bookmarks;

pub use execution::{Debugger, DebuggerState, StepResult, RunResult};
pub use history::{History, MemoryPatch};
pub use search::{MemorySearch, SearchResult};
pub use bookmarks::{Bookmark, BookmarkManager};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DebuggerError {
    #[error("Emulator error: {0}")]
    Emulator(#[from] crate::emulator::EmulatorError),

    #[error("Memory error: {0}")]
    Memory(#[from] crate::emulator::MemoryError),

    #[error("Already halted")]
    AlreadyHalted,

    #[error("Execution limit exceeded ({0} instructions)")]
    ExecutionLimit(u64),

    #[error("Invalid breakpoint address: 0x{0:08X}")]
    InvalidBreakpoint(u32),

    #[error("No actions to undo")]
    NothingToUndo,

    #[error("No actions to redo")]
    NothingToRedo,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
