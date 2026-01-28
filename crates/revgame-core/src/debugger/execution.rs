use std::collections::{HashSet, VecDeque};

use crate::emulator::{CpuState, DisassemblyLine, ExecutionResult, Executor, Memory};

use super::{DebuggerError, History, MemoryPatch, BookmarkManager};

/// Debugger execution state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebuggerState {
    /// Ready to execute
    Ready,
    /// Running (between instructions)
    Running,
    /// Stopped at a breakpoint
    AtBreakpoint(u32),
    /// Program has halted normally
    Halted,
    /// Hit execution limit
    LimitExceeded,
    /// Encountered an error
    Error(String),
}

/// Result of stepping one instruction
#[derive(Debug)]
pub struct StepResult {
    /// The instruction that was executed
    pub instruction: Option<DisassemblyLine>,
    /// New debugger state
    pub state: DebuggerState,
    /// Registers that changed
    pub changed_registers: Vec<String>,
    /// Memory addresses that changed
    pub changed_memory: Vec<(u32, u8)>,
}

/// Result of running until stop
#[derive(Debug)]
pub enum RunResult {
    /// Stopped at breakpoint
    Breakpoint(u32),
    /// Program halted
    Halted,
    /// Hit execution limit
    LimitExceeded(u64),
    /// Error occurred
    Error(String),
}

/// Entry in execution history for reverse debugging
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub eip: u32,
    pub cpu_snapshot: CpuState,
    pub instruction_text: String,
}

/// The main debugger combining CPU, memory, and execution control
pub struct Debugger {
    /// CPU state
    pub cpu: CpuState,

    /// Memory subsystem
    pub memory: Memory,

    /// Instruction executor
    executor: Executor,

    /// Current debugger state
    pub state: DebuggerState,

    /// Breakpoint addresses
    pub breakpoints: HashSet<u32>,

    /// Maximum instructions before stopping (prevents infinite loops)
    pub max_instructions: u64,

    /// Number of instructions executed in current run
    pub instructions_executed: u64,

    /// Total instructions executed across all runs
    pub total_instructions: u64,

    /// Execution history for debugging
    history: VecDeque<HistoryEntry>,

    /// Maximum history entries to keep
    max_history: usize,

    /// Patch history for undo/redo
    patch_history: History,

    /// Bookmarks for memory addresses
    pub bookmarks: BookmarkManager,

    /// Initial state for reset
    initial_cpu: CpuState,
    initial_memory: Vec<u8>,
}

impl Debugger {
    /// Create a new debugger with the given memory size
    pub fn new(memory_size: usize) -> Self {
        let memory = Memory::new(memory_size);
        Self {
            cpu: CpuState::default(),
            memory: memory.clone(),
            executor: Executor::new(),
            state: DebuggerState::Ready,
            breakpoints: HashSet::new(),
            max_instructions: 100_000,
            instructions_executed: 0,
            total_instructions: 0,
            history: VecDeque::new(),
            max_history: 1000,
            patch_history: History::new(100),
            bookmarks: BookmarkManager::new(),
            initial_cpu: CpuState::default(),
            initial_memory: vec![0; memory_size],
        }
    }

    /// Create a debugger with puzzle layout
    pub fn with_puzzle_layout(
        entry_point: u32,
        code_start: u32,
        data_start: u32,
        stack_start: u32,
    ) -> Self {
        let memory = Memory::with_puzzle_layout(code_start, data_start, stack_start);
        let cpu = CpuState::new(entry_point, stack_start);

        Self {
            cpu: cpu.clone(),
            memory: memory.clone(),
            executor: Executor::new(),
            state: DebuggerState::Ready,
            breakpoints: HashSet::new(),
            max_instructions: 100_000,
            instructions_executed: 0,
            total_instructions: 0,
            history: VecDeque::new(),
            max_history: 1000,
            patch_history: History::new(100),
            bookmarks: BookmarkManager::new(),
            initial_cpu: cpu,
            initial_memory: memory.raw().to_vec(),
        }
    }

    /// Load code into memory
    pub fn load_code(&mut self, address: u32, code: &[u8]) -> Result<(), DebuggerError> {
        self.memory.load(address, code)?;
        // Update initial state
        self.initial_memory = self.memory.raw().to_vec();
        Ok(())
    }

    /// Load data into memory
    pub fn load_data(&mut self, address: u32, data: &[u8]) -> Result<(), DebuggerError> {
        self.memory.load(address, data)?;
        self.initial_memory = self.memory.raw().to_vec();
        Ok(())
    }

    /// Set entry point
    pub fn set_entry_point(&mut self, address: u32) {
        self.cpu.eip = address;
        self.initial_cpu.eip = address;
    }

    /// Set stack pointer
    pub fn set_stack_pointer(&mut self, address: u32) {
        self.cpu.regs.esp = address;
        self.initial_cpu.regs.esp = address;
    }

    /// Save current state as initial state (for reset)
    pub fn save_initial_state(&mut self) {
        self.initial_cpu = self.cpu.clone();
        self.initial_memory = self.memory.raw().to_vec();
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.cpu = self.initial_cpu.clone();
        self.memory.load(0, &self.initial_memory).ok();
        self.state = DebuggerState::Ready;
        self.instructions_executed = 0;
        self.history.clear();
        self.patch_history.clear();
    }

    /// Set a breakpoint at the given address
    pub fn set_breakpoint(&mut self, address: u32) {
        self.breakpoints.insert(address);
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, address: u32) {
        self.breakpoints.remove(&address);
    }

    /// Toggle a breakpoint
    pub fn toggle_breakpoint(&mut self, address: u32) -> bool {
        if self.breakpoints.contains(&address) {
            self.breakpoints.remove(&address);
            false
        } else {
            self.breakpoints.insert(address);
            true
        }
    }

    /// Check if address has a breakpoint
    pub fn has_breakpoint(&self, address: u32) -> bool {
        self.breakpoints.contains(&address)
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<StepResult, DebuggerError> {
        if matches!(self.state, DebuggerState::Halted | DebuggerState::LimitExceeded) {
            return Err(DebuggerError::AlreadyHalted);
        }

        // Save state before execution for history
        let old_cpu = self.cpu.clone();
        let old_eip = self.cpu.eip;

        // Get the instruction we're about to execute
        let disasm = self.executor.disassemble(&self.memory, self.cpu.eip, 1);
        let instruction = disasm.into_iter().next();

        // Execute the instruction
        let result = self.executor.execute_one(&mut self.cpu, &mut self.memory)?;

        // Update history
        if let Some(ref instr) = instruction {
            self.history.push_back(HistoryEntry {
                eip: old_eip,
                cpu_snapshot: old_cpu.clone(),
                instruction_text: instr.text.clone(),
            });

            if self.history.len() > self.max_history {
                self.history.pop_front();
            }
        }

        // Update counters
        self.instructions_executed += 1;
        self.total_instructions += 1;

        // Determine new state and apply result
        let new_state = match result {
            ExecutionResult::Continue { next_eip } => {
                self.cpu.eip = next_eip;

                // Check for breakpoint at new address
                if self.breakpoints.contains(&next_eip) {
                    DebuggerState::AtBreakpoint(next_eip)
                } else if self.instructions_executed >= self.max_instructions {
                    DebuggerState::LimitExceeded
                } else {
                    DebuggerState::Ready
                }
            }
            ExecutionResult::Halt => {
                self.cpu.halted = true;
                DebuggerState::Halted
            }
            ExecutionResult::Breakpoint => {
                // INT 3 hit - stop at current location
                DebuggerState::AtBreakpoint(self.cpu.eip)
            }
            ExecutionResult::Interrupt(vector) => {
                // For now, treat other interrupts as a halt
                log::debug!("Interrupt {} at 0x{:08X}", vector, self.cpu.eip);
                DebuggerState::Ready
            }
        };

        self.state = new_state.clone();

        // Detect changed registers
        let changed_registers = self.detect_register_changes(&old_cpu);

        Ok(StepResult {
            instruction,
            state: new_state,
            changed_registers,
            changed_memory: Vec::new(), // TODO: track memory changes
        })
    }

    /// Run until breakpoint, halt, or limit
    pub fn run(&mut self) -> Result<RunResult, DebuggerError> {
        self.instructions_executed = 0;
        self.state = DebuggerState::Running;

        loop {
            let result = self.step()?;

            match result.state {
                DebuggerState::AtBreakpoint(addr) => {
                    return Ok(RunResult::Breakpoint(addr));
                }
                DebuggerState::Halted => {
                    return Ok(RunResult::Halted);
                }
                DebuggerState::LimitExceeded => {
                    return Ok(RunResult::LimitExceeded(self.instructions_executed));
                }
                DebuggerState::Error(msg) => {
                    return Ok(RunResult::Error(msg));
                }
                DebuggerState::Ready | DebuggerState::Running => {
                    // Continue execution
                }
            }
        }
    }

    /// Run for N instructions
    pub fn run_n(&mut self, count: u64) -> Result<RunResult, DebuggerError> {
        let original_limit = self.max_instructions;
        self.max_instructions = count;
        self.instructions_executed = 0;

        let result = self.run();

        self.max_instructions = original_limit;
        result
    }

    /// Step backward in history (for undo)
    pub fn step_back(&mut self) -> Option<HistoryEntry> {
        if let Some(entry) = self.history.pop_back() {
            self.cpu = entry.cpu_snapshot.clone();
            self.state = DebuggerState::Ready;
            Some(entry)
        } else {
            None
        }
    }

    /// Get disassembly around current EIP
    pub fn disassemble(&mut self, count: usize) -> Vec<DisassemblyLine> {
        self.executor.disassemble(&self.memory, self.cpu.eip, count)
    }

    /// Get disassembly at specific address
    pub fn disassemble_at(&mut self, address: u32, count: usize) -> Vec<DisassemblyLine> {
        self.executor.disassemble(&self.memory, address, count)
    }

    /// Patch memory (for puzzle modifications)
    pub fn patch(&mut self, address: u32, bytes: &[u8]) -> Result<(), DebuggerError> {
        // Read old bytes before patching
        let old_bytes = self.memory.read_bytes(address, bytes.len())?;

        // Apply the patch
        self.memory.write_bytes(address, bytes)?;

        // Record in history
        let patch = MemoryPatch::new(address, old_bytes, bytes.to_vec());
        self.patch_history.record(patch);

        Ok(())
    }

    /// Undo the last patch
    pub fn undo_patch(&mut self) -> Result<(), DebuggerError> {
        if let Some(patch) = self.patch_history.undo() {
            // Apply the inverse patch without recording it
            self.memory.write_bytes(patch.address, &patch.new_bytes)?;
            Ok(())
        } else {
            Err(DebuggerError::NothingToUndo)
        }
    }

    /// Redo the last undone patch
    pub fn redo_patch(&mut self) -> Result<(), DebuggerError> {
        if let Some(patch) = self.patch_history.redo() {
            // Apply the forward patch without recording it
            self.memory.write_bytes(patch.address, &patch.new_bytes)?;
            Ok(())
        } else {
            Err(DebuggerError::NothingToRedo)
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.patch_history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.patch_history.can_redo()
    }

    /// Get number of undo actions available
    pub fn undo_count(&self) -> usize {
        self.patch_history.undo_count()
    }

    /// Get number of redo actions available
    pub fn redo_count(&self) -> usize {
        self.patch_history.redo_count()
    }

    /// Get execution history
    pub fn history(&self) -> &VecDeque<HistoryEntry> {
        &self.history
    }

    /// Detect which registers changed between two states
    fn detect_register_changes(&self, old: &CpuState) -> Vec<String> {
        let mut changes = Vec::new();

        if old.regs.eax != self.cpu.regs.eax {
            changes.push("EAX".to_string());
        }
        if old.regs.ebx != self.cpu.regs.ebx {
            changes.push("EBX".to_string());
        }
        if old.regs.ecx != self.cpu.regs.ecx {
            changes.push("ECX".to_string());
        }
        if old.regs.edx != self.cpu.regs.edx {
            changes.push("EDX".to_string());
        }
        if old.regs.esi != self.cpu.regs.esi {
            changes.push("ESI".to_string());
        }
        if old.regs.edi != self.cpu.regs.edi {
            changes.push("EDI".to_string());
        }
        if old.regs.ebp != self.cpu.regs.ebp {
            changes.push("EBP".to_string());
        }
        if old.regs.esp != self.cpu.regs.esp {
            changes.push("ESP".to_string());
        }
        if old.eip != self.cpu.eip {
            changes.push("EIP".to_string());
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execution() {
        let mut dbg = Debugger::new(0x4000);
        dbg.cpu.eip = 0x1000;
        dbg.cpu.regs.esp = 0x3000;

        // MOV EAX, 0x42; HLT
        dbg.memory.load(0x1000, &[0xB8, 0x42, 0x00, 0x00, 0x00, 0xF4]).unwrap();

        let result = dbg.run().unwrap();
        assert!(matches!(result, RunResult::Halted));
        assert_eq!(dbg.cpu.regs.eax, 0x42);
    }

    #[test]
    fn test_breakpoint() {
        let mut dbg = Debugger::new(0x4000);
        dbg.cpu.eip = 0x1000;
        dbg.cpu.regs.esp = 0x3000;

        // NOP; NOP; HLT
        dbg.memory.load(0x1000, &[0x90, 0x90, 0xF4]).unwrap();
        dbg.set_breakpoint(0x1001);

        let result = dbg.run().unwrap();
        assert!(matches!(result, RunResult::Breakpoint(0x1001)));
        assert_eq!(dbg.cpu.eip, 0x1001);
    }

    #[test]
    fn test_step() {
        let mut dbg = Debugger::new(0x4000);
        dbg.cpu.eip = 0x1000;
        dbg.cpu.regs.esp = 0x3000;

        // NOP; NOP; HLT
        dbg.memory.load(0x1000, &[0x90, 0x90, 0xF4]).unwrap();

        let result = dbg.step().unwrap();
        assert!(matches!(result.state, DebuggerState::Ready));
        assert_eq!(dbg.cpu.eip, 0x1001);

        let result = dbg.step().unwrap();
        assert!(matches!(result.state, DebuggerState::Ready));
        assert_eq!(dbg.cpu.eip, 0x1002);

        let result = dbg.step().unwrap();
        assert!(matches!(result.state, DebuggerState::Halted));
    }

    #[test]
    fn test_reset() {
        let mut dbg = Debugger::new(0x4000);
        dbg.cpu.eip = 0x1000;
        dbg.cpu.regs.eax = 0;
        dbg.cpu.regs.esp = 0x3000;

        // MOV EAX, 0x42; HLT
        dbg.memory.load(0x1000, &[0xB8, 0x42, 0x00, 0x00, 0x00, 0xF4]).unwrap();
        dbg.save_initial_state();

        dbg.run().unwrap();
        assert_eq!(dbg.cpu.regs.eax, 0x42);

        dbg.reset();
        assert_eq!(dbg.cpu.regs.eax, 0);
        assert_eq!(dbg.cpu.eip, 0x1000);
    }
}
