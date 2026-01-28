use iced_x86::{
    Decoder, DecoderOptions, Instruction, Mnemonic, OpKind, Register as IcedRegister,
};

use super::{CpuState, DisassemblyLine, Disassembler, EmulatorError, Memory, Register};

/// Result of executing a single instruction
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Continue execution at the given address
    Continue { next_eip: u32 },
    /// CPU has halted (HLT instruction)
    Halt,
    /// Hit a breakpoint (INT 3)
    Breakpoint,
    /// Syscall/interrupt (for future I/O)
    Interrupt(u8),
}

/// The instruction executor
pub struct Executor {
    bitness: u32,
    disassembler: Disassembler,
}

impl Executor {
    /// Create a new executor for 32-bit mode
    pub fn new() -> Self {
        Self {
            bitness: 32,
            disassembler: Disassembler::new(),
        }
    }

    /// Execute a single instruction and return the result
    pub fn execute_one(
        &mut self,
        cpu: &mut CpuState,
        memory: &mut Memory,
    ) -> Result<ExecutionResult, EmulatorError> {
        // Fetch bytes from current EIP
        let bytes = memory.read_bytes(cpu.eip, 15)?; // Max x86 instruction length

        // Decode instruction
        let mut decoder = Decoder::with_ip(
            self.bitness,
            &bytes,
            cpu.eip as u64,
            DecoderOptions::NONE,
        );

        let instr = decoder.decode();
        if instr.is_invalid() {
            return Err(EmulatorError::UnsupportedInstruction(format!(
                "Invalid instruction at 0x{:08X}",
                cpu.eip
            )));
        }

        let instr_len = instr.len() as u32;
        let next_eip = cpu.eip + instr_len;

        // Execute based on mnemonic
        let result = match instr.mnemonic() {
            // Data movement
            Mnemonic::Mov => self.exec_mov(cpu, memory, &instr, next_eip),
            Mnemonic::Push => self.exec_push(cpu, memory, &instr, next_eip),
            Mnemonic::Pop => self.exec_pop(cpu, memory, &instr, next_eip),
            Mnemonic::Xchg => self.exec_xchg(cpu, memory, &instr, next_eip),
            Mnemonic::Lea => self.exec_lea(cpu, &instr, next_eip),

            // Arithmetic
            Mnemonic::Add => self.exec_add(cpu, memory, &instr, next_eip),
            Mnemonic::Sub => self.exec_sub(cpu, memory, &instr, next_eip),
            Mnemonic::Inc => self.exec_inc(cpu, memory, &instr, next_eip),
            Mnemonic::Dec => self.exec_dec(cpu, memory, &instr, next_eip),
            Mnemonic::Neg => self.exec_neg(cpu, memory, &instr, next_eip),
            Mnemonic::Imul => self.exec_imul(cpu, memory, &instr, next_eip),
            Mnemonic::Mul => self.exec_mul(cpu, memory, &instr, next_eip),

            // Logic
            Mnemonic::And => self.exec_and(cpu, memory, &instr, next_eip),
            Mnemonic::Or => self.exec_or(cpu, memory, &instr, next_eip),
            Mnemonic::Xor => self.exec_xor(cpu, memory, &instr, next_eip),
            Mnemonic::Not => self.exec_not(cpu, memory, &instr, next_eip),
            Mnemonic::Shl | Mnemonic::Sal => self.exec_shl(cpu, memory, &instr, next_eip),
            Mnemonic::Shr => self.exec_shr(cpu, memory, &instr, next_eip),
            Mnemonic::Sar => self.exec_sar(cpu, memory, &instr, next_eip),

            // Comparison
            Mnemonic::Cmp => self.exec_cmp(cpu, memory, &instr, next_eip),
            Mnemonic::Test => self.exec_test(cpu, memory, &instr, next_eip),

            // Unconditional jump
            Mnemonic::Jmp => self.exec_jmp(cpu, &instr),

            // Conditional jumps (using primary mnemonic names from iced-x86)
            Mnemonic::Je => self.exec_jcc(cpu, &instr, next_eip, cpu.eflags.zf),
            Mnemonic::Jne => self.exec_jcc(cpu, &instr, next_eip, !cpu.eflags.zf),
            Mnemonic::Jg => {
                let cond = !cpu.eflags.zf && (cpu.eflags.sf == cpu.eflags.of);
                self.exec_jcc(cpu, &instr, next_eip, cond)
            }
            Mnemonic::Jge => {
                let cond = cpu.eflags.sf == cpu.eflags.of;
                self.exec_jcc(cpu, &instr, next_eip, cond)
            }
            Mnemonic::Jl => {
                let cond = cpu.eflags.sf != cpu.eflags.of;
                self.exec_jcc(cpu, &instr, next_eip, cond)
            }
            Mnemonic::Jle => {
                let cond = cpu.eflags.zf || (cpu.eflags.sf != cpu.eflags.of);
                self.exec_jcc(cpu, &instr, next_eip, cond)
            }
            Mnemonic::Ja => {
                let cond = !cpu.eflags.cf && !cpu.eflags.zf;
                self.exec_jcc(cpu, &instr, next_eip, cond)
            }
            Mnemonic::Jae => {
                self.exec_jcc(cpu, &instr, next_eip, !cpu.eflags.cf)
            }
            Mnemonic::Jb => {
                self.exec_jcc(cpu, &instr, next_eip, cpu.eflags.cf)
            }
            Mnemonic::Jbe => {
                let cond = cpu.eflags.cf || cpu.eflags.zf;
                self.exec_jcc(cpu, &instr, next_eip, cond)
            }
            Mnemonic::Js => self.exec_jcc(cpu, &instr, next_eip, cpu.eflags.sf),
            Mnemonic::Jns => self.exec_jcc(cpu, &instr, next_eip, !cpu.eflags.sf),
            Mnemonic::Jo => self.exec_jcc(cpu, &instr, next_eip, cpu.eflags.of),
            Mnemonic::Jno => self.exec_jcc(cpu, &instr, next_eip, !cpu.eflags.of),

            // Call/Return
            Mnemonic::Call => self.exec_call(cpu, memory, &instr, next_eip),
            Mnemonic::Ret => self.exec_ret(cpu, memory, &instr),

            // Misc
            Mnemonic::Nop => Ok(ExecutionResult::Continue { next_eip }),
            Mnemonic::Hlt => {
                cpu.halted = true;
                Ok(ExecutionResult::Halt)
            }
            Mnemonic::Int => self.exec_int(cpu, &instr, next_eip),
            Mnemonic::Int3 => Ok(ExecutionResult::Breakpoint),

            _ => Err(EmulatorError::UnsupportedInstruction(format!(
                "{:?} at 0x{:08X}",
                instr.mnemonic(),
                cpu.eip
            ))),
        };

        result
    }

    /// Get disassembly around an address
    pub fn disassemble(&mut self, memory: &Memory, address: u32, count: usize) -> Vec<DisassemblyLine> {
        if let Ok(bytes) = memory.read_bytes(address, count * 15) {
            self.disassembler.disassemble(&bytes, address, count)
        } else {
            Vec::new()
        }
    }

    // ==================== Helper functions ====================

    /// Convert iced register to our Register enum
    fn iced_to_register(reg: IcedRegister) -> Option<Register> {
        match reg {
            IcedRegister::EAX | IcedRegister::AX | IcedRegister::AL | IcedRegister::AH => {
                Some(Register::Eax)
            }
            IcedRegister::EBX | IcedRegister::BX | IcedRegister::BL | IcedRegister::BH => {
                Some(Register::Ebx)
            }
            IcedRegister::ECX | IcedRegister::CX | IcedRegister::CL | IcedRegister::CH => {
                Some(Register::Ecx)
            }
            IcedRegister::EDX | IcedRegister::DX | IcedRegister::DL | IcedRegister::DH => {
                Some(Register::Edx)
            }
            IcedRegister::ESI | IcedRegister::SI => Some(Register::Esi),
            IcedRegister::EDI | IcedRegister::DI => Some(Register::Edi),
            IcedRegister::EBP | IcedRegister::BP => Some(Register::Ebp),
            IcedRegister::ESP | IcedRegister::SP => Some(Register::Esp),
            _ => None,
        }
    }

    /// Read operand value (register or immediate)
    fn read_operand(
        &self,
        cpu: &CpuState,
        memory: &Memory,
        instr: &Instruction,
        op_idx: u32,
    ) -> Result<u32, EmulatorError> {
        match instr.op_kind(op_idx) {
            OpKind::Register => {
                let reg = instr.op_register(op_idx);
                if let Some(r) = Self::iced_to_register(reg) {
                    Ok(cpu.get_register(r))
                } else {
                    Err(EmulatorError::InvalidOperand(format!(
                        "Unsupported register: {:?}",
                        reg
                    )))
                }
            }
            OpKind::Immediate8 => Ok(instr.immediate8() as u32),
            OpKind::Immediate16 => Ok(instr.immediate16() as u32),
            OpKind::Immediate32 => Ok(instr.immediate32()),
            OpKind::Immediate8to32 => Ok(instr.immediate8to32() as u32),
            OpKind::Immediate8to16 => Ok(instr.immediate8to16() as u32),
            OpKind::Memory => {
                let addr = self.calculate_memory_address(cpu, instr)?;
                memory.read_u32(addr).map_err(EmulatorError::from)
            }
            _ => Err(EmulatorError::InvalidOperand(format!(
                "Unsupported operand kind: {:?}",
                instr.op_kind(op_idx)
            ))),
        }
    }

    /// Write to operand (register or memory)
    fn write_operand(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        op_idx: u32,
        value: u32,
    ) -> Result<(), EmulatorError> {
        match instr.op_kind(op_idx) {
            OpKind::Register => {
                let reg = instr.op_register(op_idx);
                if let Some(r) = Self::iced_to_register(reg) {
                    cpu.set_register(r, value);
                    Ok(())
                } else {
                    Err(EmulatorError::InvalidOperand(format!(
                        "Unsupported register: {:?}",
                        reg
                    )))
                }
            }
            OpKind::Memory => {
                let addr = self.calculate_memory_address(cpu, instr)?;
                memory.write_u32(addr, value).map_err(EmulatorError::from)
            }
            _ => Err(EmulatorError::InvalidOperand(format!(
                "Cannot write to operand kind: {:?}",
                instr.op_kind(op_idx)
            ))),
        }
    }

    /// Calculate effective address for memory operand
    fn calculate_memory_address(
        &self,
        cpu: &CpuState,
        instr: &Instruction,
    ) -> Result<u32, EmulatorError> {
        let mut addr = instr.memory_displacement64() as u32;

        // Add base register
        if instr.memory_base() != IcedRegister::None {
            if let Some(r) = Self::iced_to_register(instr.memory_base()) {
                addr = addr.wrapping_add(cpu.get_register(r));
            }
        }

        // Add index register * scale
        if instr.memory_index() != IcedRegister::None {
            if let Some(r) = Self::iced_to_register(instr.memory_index()) {
                let index_val = cpu.get_register(r);
                let scale = instr.memory_index_scale();
                addr = addr.wrapping_add(index_val.wrapping_mul(scale));
            }
        }

        Ok(addr)
    }

    // ==================== Instruction implementations ====================

    fn exec_mov(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 1)?;
        self.write_operand(cpu, memory, instr, 0, value)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_push(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 0)?;
        cpu.regs.esp = cpu.regs.esp.wrapping_sub(4);
        memory.write_u32(cpu.regs.esp, value)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_pop(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = memory.read_u32(cpu.regs.esp)?;
        cpu.regs.esp = cpu.regs.esp.wrapping_add(4);
        self.write_operand(cpu, memory, instr, 0, value)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_xchg(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let val1 = self.read_operand(cpu, memory, instr, 0)?;
        let val2 = self.read_operand(cpu, memory, instr, 1)?;
        self.write_operand(cpu, memory, instr, 0, val2)?;
        self.write_operand(cpu, memory, instr, 1, val1)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_lea(
        &self,
        cpu: &mut CpuState,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let addr = self.calculate_memory_address(cpu, instr)?;
        let dest_reg = instr.op_register(0);
        if let Some(r) = Self::iced_to_register(dest_reg) {
            cpu.set_register(r, addr);
            Ok(ExecutionResult::Continue { next_eip })
        } else {
            Err(EmulatorError::InvalidOperand(format!(
                "Invalid LEA destination: {:?}",
                dest_reg
            )))
        }
    }

    fn exec_add(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let op1 = self.read_operand(cpu, memory, instr, 0)?;
        let op2 = self.read_operand(cpu, memory, instr, 1)?;
        let result = op1.wrapping_add(op2);
        cpu.eflags.update_arithmetic(result, op1, op2, false);
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_sub(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let op1 = self.read_operand(cpu, memory, instr, 0)?;
        let op2 = self.read_operand(cpu, memory, instr, 1)?;
        let result = op1.wrapping_sub(op2);
        cpu.eflags.update_arithmetic(result, op1, op2, true);
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_inc(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 0)?;
        let result = value.wrapping_add(1);
        cpu.eflags.update_inc(result, value);
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_dec(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 0)?;
        let result = value.wrapping_sub(1);
        cpu.eflags.update_dec(result, value);
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_neg(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 0)?;
        let result = (-(value as i32)) as u32;
        cpu.eflags.update_arithmetic(result, 0, value, true);
        cpu.eflags.cf = value != 0; // NEG sets CF if operand is non-zero
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_imul(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        // Handle different IMUL forms
        let op_count = instr.op_count();

        match op_count {
            1 => {
                // IMUL r/m32: EDX:EAX = EAX * r/m32
                let src = self.read_operand(cpu, memory, instr, 0)? as i32;
                let eax = cpu.regs.eax as i32;
                let result = (eax as i64) * (src as i64);
                cpu.regs.eax = result as u32;
                cpu.regs.edx = (result >> 32) as u32;
                // OF=CF=1 if result doesn't fit in 32 bits
                let fits = result >= i32::MIN as i64 && result <= i32::MAX as i64;
                cpu.eflags.of = !fits;
                cpu.eflags.cf = !fits;
            }
            2 => {
                // IMUL r32, r/m32: r32 = r32 * r/m32
                let src = self.read_operand(cpu, memory, instr, 1)? as i32;
                let dest = self.read_operand(cpu, memory, instr, 0)? as i32;
                let result = (dest as i64) * (src as i64);
                self.write_operand(cpu, memory, instr, 0, result as u32)?;
                let fits = result >= i32::MIN as i64 && result <= i32::MAX as i64;
                cpu.eflags.of = !fits;
                cpu.eflags.cf = !fits;
            }
            3 => {
                // IMUL r32, r/m32, imm: r32 = r/m32 * imm
                let src = self.read_operand(cpu, memory, instr, 1)? as i32;
                let imm = self.read_operand(cpu, memory, instr, 2)? as i32;
                let result = (src as i64) * (imm as i64);
                self.write_operand(cpu, memory, instr, 0, result as u32)?;
                let fits = result >= i32::MIN as i64 && result <= i32::MAX as i64;
                cpu.eflags.of = !fits;
                cpu.eflags.cf = !fits;
            }
            _ => {
                return Err(EmulatorError::InvalidOperand(
                    "Invalid IMUL operand count".to_string(),
                ));
            }
        }

        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_mul(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        // MUL r/m32: EDX:EAX = EAX * r/m32 (unsigned)
        let src = self.read_operand(cpu, memory, instr, 0)? as u64;
        let eax = cpu.regs.eax as u64;
        let result = eax * src;
        cpu.regs.eax = result as u32;
        cpu.regs.edx = (result >> 32) as u32;
        // OF=CF=1 if high half is non-zero
        cpu.eflags.of = cpu.regs.edx != 0;
        cpu.eflags.cf = cpu.regs.edx != 0;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_and(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let op1 = self.read_operand(cpu, memory, instr, 0)?;
        let op2 = self.read_operand(cpu, memory, instr, 1)?;
        let result = op1 & op2;
        cpu.eflags.update_logical(result);
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_or(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let op1 = self.read_operand(cpu, memory, instr, 0)?;
        let op2 = self.read_operand(cpu, memory, instr, 1)?;
        let result = op1 | op2;
        cpu.eflags.update_logical(result);
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_xor(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let op1 = self.read_operand(cpu, memory, instr, 0)?;
        let op2 = self.read_operand(cpu, memory, instr, 1)?;
        let result = op1 ^ op2;
        cpu.eflags.update_logical(result);
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_not(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 0)?;
        let result = !value;
        // NOT does not affect flags
        self.write_operand(cpu, memory, instr, 0, result)?;
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_shl(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 0)?;
        let count = (self.read_operand(cpu, memory, instr, 1)? & 0x1F) as u32;

        if count > 0 {
            let result = value << count;
            cpu.eflags.cf = ((value >> (32 - count)) & 1) != 0;
            cpu.eflags.update_logical(result);
            if count == 1 {
                cpu.eflags.of = ((result >> 31) & 1) != cpu.eflags.cf as u32;
            }
            self.write_operand(cpu, memory, instr, 0, result)?;
        }

        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_shr(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 0)?;
        let count = (self.read_operand(cpu, memory, instr, 1)? & 0x1F) as u32;

        if count > 0 {
            let result = value >> count;
            cpu.eflags.cf = ((value >> (count - 1)) & 1) != 0;
            cpu.eflags.update_logical(result);
            if count == 1 {
                cpu.eflags.of = (value >> 31) != 0;
            }
            self.write_operand(cpu, memory, instr, 0, result)?;
        }

        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_sar(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let value = self.read_operand(cpu, memory, instr, 0)? as i32;
        let count = (self.read_operand(cpu, memory, instr, 1)? & 0x1F) as u32;

        if count > 0 {
            let result = (value >> count) as u32;
            cpu.eflags.cf = ((value >> (count as i32 - 1)) & 1) != 0;
            cpu.eflags.update_logical(result);
            if count == 1 {
                cpu.eflags.of = false; // SAR always clears OF for count=1
            }
            self.write_operand(cpu, memory, instr, 0, result)?;
        }

        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_cmp(
        &self,
        cpu: &mut CpuState,
        memory: &Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let op1 = self.read_operand(cpu, memory, instr, 0)?;
        let op2 = self.read_operand(cpu, memory, instr, 1)?;
        let result = op1.wrapping_sub(op2);
        cpu.eflags.update_arithmetic(result, op1, op2, true);
        // CMP doesn't store result, only updates flags
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_test(
        &self,
        cpu: &mut CpuState,
        memory: &Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let op1 = self.read_operand(cpu, memory, instr, 0)?;
        let op2 = self.read_operand(cpu, memory, instr, 1)?;
        let result = op1 & op2;
        cpu.eflags.update_logical(result);
        // TEST doesn't store result, only updates flags
        Ok(ExecutionResult::Continue { next_eip })
    }

    fn exec_jmp(
        &self,
        _cpu: &CpuState,
        instr: &Instruction,
    ) -> Result<ExecutionResult, EmulatorError> {
        let target = match instr.op_kind(0) {
            OpKind::NearBranch16 => instr.near_branch16() as u32,
            OpKind::NearBranch32 => instr.near_branch32(),
            OpKind::NearBranch64 => instr.near_branch64() as u32,
            _ => {
                return Err(EmulatorError::InvalidOperand(format!(
                    "Unsupported JMP operand: {:?}",
                    instr.op_kind(0)
                )));
            }
        };
        Ok(ExecutionResult::Continue { next_eip: target })
    }

    fn exec_jcc(
        &self,
        _cpu: &CpuState,
        instr: &Instruction,
        next_eip: u32,
        condition: bool,
    ) -> Result<ExecutionResult, EmulatorError> {
        if condition {
            let target = match instr.op_kind(0) {
                OpKind::NearBranch16 => instr.near_branch16() as u32,
                OpKind::NearBranch32 => instr.near_branch32(),
                OpKind::NearBranch64 => instr.near_branch64() as u32,
                _ => {
                    return Err(EmulatorError::InvalidOperand(format!(
                        "Unsupported Jcc operand: {:?}",
                        instr.op_kind(0)
                    )));
                }
            };
            Ok(ExecutionResult::Continue { next_eip: target })
        } else {
            Ok(ExecutionResult::Continue { next_eip })
        }
    }

    fn exec_call(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        // Push return address
        cpu.regs.esp = cpu.regs.esp.wrapping_sub(4);
        memory.write_u32(cpu.regs.esp, next_eip)?;

        // Get target address
        let target = match instr.op_kind(0) {
            OpKind::NearBranch16 => instr.near_branch16() as u32,
            OpKind::NearBranch32 => instr.near_branch32(),
            OpKind::NearBranch64 => instr.near_branch64() as u32,
            _ => {
                return Err(EmulatorError::InvalidOperand(format!(
                    "Unsupported CALL operand: {:?}",
                    instr.op_kind(0)
                )));
            }
        };

        Ok(ExecutionResult::Continue { next_eip: target })
    }

    fn exec_ret(
        &self,
        cpu: &mut CpuState,
        memory: &mut Memory,
        instr: &Instruction,
    ) -> Result<ExecutionResult, EmulatorError> {
        // Pop return address
        let return_addr = memory.read_u32(cpu.regs.esp)?;
        cpu.regs.esp = cpu.regs.esp.wrapping_add(4);

        // Handle RET imm16 (pop additional bytes)
        if instr.op_count() > 0 {
            if let OpKind::Immediate16 = instr.op_kind(0) {
                let pop_bytes = instr.immediate16() as u32;
                cpu.regs.esp = cpu.regs.esp.wrapping_add(pop_bytes);
            }
        }

        Ok(ExecutionResult::Continue {
            next_eip: return_addr,
        })
    }

    fn exec_int(
        &self,
        _cpu: &CpuState,
        instr: &Instruction,
        next_eip: u32,
    ) -> Result<ExecutionResult, EmulatorError> {
        let vector = match instr.op_kind(0) {
            OpKind::Immediate8 => instr.immediate8(),
            _ => {
                return Err(EmulatorError::InvalidOperand(
                    "Invalid INT operand".to_string(),
                ));
            }
        };

        // INT 3 is a breakpoint
        if vector == 3 {
            return Ok(ExecutionResult::Breakpoint);
        }

        // Other interrupts could be used for simulated syscalls
        Ok(ExecutionResult::Interrupt(vector))
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test() -> (CpuState, Memory, Executor) {
        let cpu = CpuState::new(0x1000, 0x3000);
        let mut memory = Memory::new(0x4000);
        let executor = Executor::new();
        (cpu, memory, executor)
    }

    #[test]
    fn test_mov_reg_imm() {
        let (mut cpu, mut memory, mut executor) = setup_test();
        // MOV EAX, 0x12345678
        memory.load(0x1000, &[0xB8, 0x78, 0x56, 0x34, 0x12]).unwrap();

        let result = executor.execute_one(&mut cpu, &mut memory).unwrap();
        assert!(matches!(result, ExecutionResult::Continue { next_eip: 0x1005 }));
        assert_eq!(cpu.regs.eax, 0x12345678);
    }

    #[test]
    fn test_add_reg_reg() {
        let (mut cpu, mut memory, mut executor) = setup_test();
        cpu.regs.eax = 10;
        cpu.regs.ebx = 20;
        // ADD EAX, EBX
        memory.load(0x1000, &[0x01, 0xD8]).unwrap();

        executor.execute_one(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.regs.eax, 30);
    }

    #[test]
    fn test_cmp_sets_zf() {
        let (mut cpu, mut memory, mut executor) = setup_test();
        cpu.regs.eax = 5;
        cpu.regs.ebx = 5;
        // CMP EAX, EBX
        memory.load(0x1000, &[0x39, 0xD8]).unwrap();

        executor.execute_one(&mut cpu, &mut memory).unwrap();
        assert!(cpu.eflags.zf);
    }

    #[test]
    fn test_jne_not_taken() {
        let (mut cpu, mut memory, mut executor) = setup_test();
        cpu.eflags.zf = true;
        // JNE +5
        memory.load(0x1000, &[0x75, 0x05]).unwrap();

        let result = executor.execute_one(&mut cpu, &mut memory).unwrap();
        assert!(matches!(result, ExecutionResult::Continue { next_eip: 0x1002 }));
    }

    #[test]
    fn test_jne_taken() {
        let (mut cpu, mut memory, mut executor) = setup_test();
        cpu.eflags.zf = false;
        // JNE +5
        memory.load(0x1000, &[0x75, 0x05]).unwrap();

        let result = executor.execute_one(&mut cpu, &mut memory).unwrap();
        assert!(matches!(result, ExecutionResult::Continue { next_eip: 0x1007 }));
    }

    #[test]
    fn test_push_pop() {
        let (mut cpu, mut memory, mut executor) = setup_test();
        cpu.regs.eax = 0xDEADBEEF;

        // PUSH EAX
        memory.load(0x1000, &[0x50]).unwrap();
        executor.execute_one(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.regs.esp, 0x2FFC);

        // Zero EAX
        cpu.regs.eax = 0;
        cpu.eip = 0x1001;

        // POP EBX
        memory.load(0x1001, &[0x5B]).unwrap();
        executor.execute_one(&mut cpu, &mut memory).unwrap();
        assert_eq!(cpu.regs.ebx, 0xDEADBEEF);
        assert_eq!(cpu.regs.esp, 0x3000);
    }
}
