use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Address 0x{address:08X} out of bounds (size: 0x{size:X})")]
    OutOfBounds { address: u32, size: usize },

    #[error("Access violation at 0x{address:08X}: {reason}")]
    AccessViolation { address: u32, reason: String },

    #[error("Unaligned access at 0x{address:08X} (alignment: {alignment})")]
    UnalignedAccess { address: u32, alignment: u32 },
}

/// Memory region permissions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permissions {
    pub fn rwx() -> Self {
        Self { read: true, write: true, execute: true }
    }

    pub fn rw() -> Self {
        Self { read: true, write: true, execute: false }
    }

    pub fn rx() -> Self {
        Self { read: true, write: false, execute: true }
    }

    pub fn ro() -> Self {
        Self { read: true, write: false, execute: false }
    }
}

/// A named memory region with permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub permissions: Permissions,
}

impl MemoryRegion {
    pub fn contains(&self, address: u32) -> bool {
        address >= self.start && address < self.end
    }

    pub fn size(&self) -> u32 {
        self.end - self.start
    }
}

/// Memory subsystem for the emulator
#[derive(Debug, Clone)]
pub struct Memory {
    /// Flat memory array
    data: Vec<u8>,

    /// Named regions with permissions
    regions: Vec<MemoryRegion>,

    /// Whether to enforce permissions (can be disabled for puzzles)
    enforce_permissions: bool,
}

impl Memory {
    /// Create a new memory instance with the given size
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            regions: Vec::new(),
            enforce_permissions: false, // Default off for simpler puzzles
        }
    }

    /// Create memory with default puzzle layout
    pub fn with_puzzle_layout(code_start: u32, data_start: u32, stack_start: u32) -> Self {
        let size = (stack_start + 0x1000) as usize; // Stack grows down, give it 4KB
        let mut mem = Self::new(size);

        mem.regions.push(MemoryRegion {
            name: "code".to_string(),
            start: code_start,
            end: data_start,
            permissions: Permissions::rx(),
        });

        mem.regions.push(MemoryRegion {
            name: "data".to_string(),
            start: data_start,
            end: stack_start - 0x1000,
            permissions: Permissions::rw(),
        });

        mem.regions.push(MemoryRegion {
            name: "stack".to_string(),
            start: stack_start - 0x1000,
            end: stack_start + 0x1000,
            permissions: Permissions::rw(),
        });

        mem
    }

    /// Get the total memory size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Enable or disable permission enforcement
    pub fn set_enforce_permissions(&mut self, enforce: bool) {
        self.enforce_permissions = enforce;
    }

    /// Add a memory region
    pub fn add_region(&mut self, region: MemoryRegion) {
        self.regions.push(region);
    }

    /// Get the region containing an address
    pub fn get_region(&self, address: u32) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(address))
    }

    /// Check if an address is within bounds
    fn check_bounds(&self, address: u32, size: usize) -> Result<(), MemoryError> {
        let end = address as usize + size;
        if end > self.data.len() {
            return Err(MemoryError::OutOfBounds {
                address,
                size: self.data.len()
            });
        }
        Ok(())
    }

    /// Check read permission
    fn check_read(&self, address: u32) -> Result<(), MemoryError> {
        if !self.enforce_permissions {
            return Ok(());
        }

        if let Some(region) = self.get_region(address) {
            if !region.permissions.read {
                return Err(MemoryError::AccessViolation {
                    address,
                    reason: format!("Region '{}' is not readable", region.name),
                });
            }
        }
        Ok(())
    }

    /// Check write permission
    fn check_write(&self, address: u32) -> Result<(), MemoryError> {
        if !self.enforce_permissions {
            return Ok(());
        }

        if let Some(region) = self.get_region(address) {
            if !region.permissions.write {
                return Err(MemoryError::AccessViolation {
                    address,
                    reason: format!("Region '{}' is not writable", region.name),
                });
            }
        }
        Ok(())
    }

    /// Read a single byte
    pub fn read_u8(&self, address: u32) -> Result<u8, MemoryError> {
        self.check_bounds(address, 1)?;
        self.check_read(address)?;
        Ok(self.data[address as usize])
    }

    /// Read a 16-bit value (little-endian)
    pub fn read_u16(&self, address: u32) -> Result<u16, MemoryError> {
        self.check_bounds(address, 2)?;
        self.check_read(address)?;
        let addr = address as usize;
        Ok(u16::from_le_bytes([self.data[addr], self.data[addr + 1]]))
    }

    /// Read a 32-bit value (little-endian)
    pub fn read_u32(&self, address: u32) -> Result<u32, MemoryError> {
        self.check_bounds(address, 4)?;
        self.check_read(address)?;
        let addr = address as usize;
        Ok(u32::from_le_bytes([
            self.data[addr],
            self.data[addr + 1],
            self.data[addr + 2],
            self.data[addr + 3],
        ]))
    }

    /// Read a slice of bytes
    pub fn read_bytes(&self, address: u32, count: usize) -> Result<Vec<u8>, MemoryError> {
        self.check_bounds(address, count)?;
        self.check_read(address)?;
        let addr = address as usize;
        Ok(self.data[addr..addr + count].to_vec())
    }

    /// Write a single byte
    pub fn write_u8(&mut self, address: u32, value: u8) -> Result<(), MemoryError> {
        self.check_bounds(address, 1)?;
        self.check_write(address)?;
        self.data[address as usize] = value;
        Ok(())
    }

    /// Write a 16-bit value (little-endian)
    pub fn write_u16(&mut self, address: u32, value: u16) -> Result<(), MemoryError> {
        self.check_bounds(address, 2)?;
        self.check_write(address)?;
        let bytes = value.to_le_bytes();
        let addr = address as usize;
        self.data[addr] = bytes[0];
        self.data[addr + 1] = bytes[1];
        Ok(())
    }

    /// Write a 32-bit value (little-endian)
    pub fn write_u32(&mut self, address: u32, value: u32) -> Result<(), MemoryError> {
        self.check_bounds(address, 4)?;
        self.check_write(address)?;
        let bytes = value.to_le_bytes();
        let addr = address as usize;
        self.data[addr] = bytes[0];
        self.data[addr + 1] = bytes[1];
        self.data[addr + 2] = bytes[2];
        self.data[addr + 3] = bytes[3];
        Ok(())
    }

    /// Write a slice of bytes
    pub fn write_bytes(&mut self, address: u32, bytes: &[u8]) -> Result<(), MemoryError> {
        self.check_bounds(address, bytes.len())?;
        self.check_write(address)?;
        let addr = address as usize;
        self.data[addr..addr + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    /// Load binary data at an address (bypasses permission checks for initial setup)
    pub fn load(&mut self, address: u32, data: &[u8]) -> Result<(), MemoryError> {
        self.check_bounds(address, data.len())?;
        let addr = address as usize;
        self.data[addr..addr + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Get a slice of raw memory for display purposes
    pub fn slice(&self, address: u32, count: usize) -> Option<&[u8]> {
        let addr = address as usize;
        if addr + count <= self.data.len() {
            Some(&self.data[addr..addr + count])
        } else {
            None
        }
    }

    /// Get raw access to memory (for debugging/display)
    pub fn raw(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_u8() {
        let mut mem = Memory::new(0x1000);
        mem.write_u8(0x100, 0xAB).unwrap();
        assert_eq!(mem.read_u8(0x100).unwrap(), 0xAB);
    }

    #[test]
    fn test_read_write_u32() {
        let mut mem = Memory::new(0x1000);
        mem.write_u32(0x100, 0xDEADBEEF).unwrap();
        assert_eq!(mem.read_u32(0x100).unwrap(), 0xDEADBEEF);

        // Check little-endian byte order
        assert_eq!(mem.read_u8(0x100).unwrap(), 0xEF);
        assert_eq!(mem.read_u8(0x101).unwrap(), 0xBE);
        assert_eq!(mem.read_u8(0x102).unwrap(), 0xAD);
        assert_eq!(mem.read_u8(0x103).unwrap(), 0xDE);
    }

    #[test]
    fn test_out_of_bounds() {
        let mem = Memory::new(0x100);
        assert!(mem.read_u8(0x100).is_err());
        assert!(mem.read_u32(0xFE).is_err()); // Would read past end
    }

    #[test]
    fn test_load() {
        let mut mem = Memory::new(0x1000);
        let data = [0x90, 0x90, 0xCC, 0xC3]; // NOP NOP INT3 RET
        mem.load(0x1000 - 4, &data).unwrap();

        assert_eq!(mem.read_u8(0xFFC).unwrap(), 0x90);
        assert_eq!(mem.read_u8(0xFFD).unwrap(), 0x90);
        assert_eq!(mem.read_u8(0xFFE).unwrap(), 0xCC);
        assert_eq!(mem.read_u8(0xFFF).unwrap(), 0xC3);
    }
}
