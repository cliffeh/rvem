use goblin::elf::Elf;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::{Index, IndexMut, Range};
use std::os::fd::FromRawFd;
use std::path::Path;
use std::process;
use strum::IntoEnumIterator;
use thiserror::Error;

pub(crate) mod reg;
pub use reg::Reg;
pub(crate) mod inst;
pub use inst::Inst;

/// Default amount of memory to allocate if not specified
pub const DEFAULT_MEMORY_SIZE: usize = 1 << 20;
/// Symbol name for the program entrypoint
const ENTRYPOINT_SYM: &str = "_start";
/// Symbol name for the global pointer
const GLOBAL_POINTER_SYM: &str = "__global_pointer$";
/// Symbol names for the start/end of the BSS region
const BSS_START_SYM: &str = "__bss_start";
const BSS_END_SYM: &str = "__BSS_END__";

/// Sign-extend `$value` from `$bits` to 32 bits.
pub(crate) fn sext(value: u32, bits: usize) -> u32 {
    ((value << (32 - bits)) as i32 >> (32 - bits)) as u32
}

/// Representation of a RISC-V machine.
pub struct Emulator {
    /// Program counter
    pc: usize,
    /// Registers
    reg: [u32; 32],
    /// Memory
    mem: Vec<u8>,
    /// Map of section names to their corresponding memory ranges
    sections: HashMap<String, Range<usize>>,
    /// Symbol table
    symtab: HashMap<String, usize>,
    /// The Great Bit-Bucket in the Sky
    dev_null: u32,
}

impl Emulator {
    /// Allocates a new Emulator with `alloc` bytes of memory,
    /// or [DEFAULT_MEMORY_SIZE] bytes if `None` is provided.
    pub fn new(alloc: Option<usize>) -> Emulator {
        Emulator {
            pc: 0x0,
            reg: [0u32; 32],
            mem: vec![
                0u8;
                if let Some(n) = alloc {
                    n
                } else {
                    DEFAULT_MEMORY_SIZE
                }
            ],
            sections: HashMap::new(),
            symtab: HashMap::new(),
            dev_null: 0x0,
        }
    }

    /// Loads a RISC-V program from the ELF file at `path` and returns the
    /// resulting [Emulator], or an [EmulatorError] if an error occurred
    /// (e.g., the file doesn't exist, isn't formatted correctly, etc.).
    pub fn load_from<P: AsRef<Path>>(
        path: P,
        alloc: Option<usize>,
    ) -> Result<Emulator, EmulatorError> {
        let mut em = Emulator::new(alloc);
        em.load(path)?;
        Ok(em)
    }

    /// Loads a RISC-V program from the ELF file at `path` into `self`.
    /// Returns the unit type, or an [EmulatorError] if an error occurred
    /// (e.g., the file doesn't exist, isn't formatted correctly, etc.).
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), EmulatorError> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let elf = Elf::parse(&buf)?;

        // load allocatable sections
        for section in &elf.section_headers {
            if section.is_alloc() {
                let name = elf.shdr_strtab.get_at(section.sh_name).unwrap().to_string();
                log::debug!(
                    "found section: {}; address: 0x{:x}, length: {} bytes",
                    name,
                    section.sh_addr,
                    section.sh_size
                );

                if let Some(range) = section.file_range() {
                    self[section.vm_range()].copy_from_slice(&buf[range]);
                    self.sections.insert(name, section.vm_range());
                } // TODO if SHT_NOBITS initialize the memory (e.g., .tbss)
            }
        }

        // load the symbol table
        for sym in elf.syms.iter() {
            if let Some(name) = elf.strtab.get_at(sym.st_name) {
                if name.len() > 0 {
                    self.symtab.insert(name.into(), sym.st_value as usize);
                }
            }
        }

        // zero the Block Started by Symbol (BSS) region
        if let Some(bss_start) = self.symtab.get(BSS_START_SYM) {
            if let Some(bss_end) = self.symtab.get(BSS_END_SYM) {
                for i in *bss_start..*bss_end {
                    self[i] = 0u8;
                }
            }
        }

        Ok(())
    }

    /// Runs a loaded program, returning the unit type or an [EmulatorError].
    pub fn run(&mut self) -> Result<(), EmulatorError> {
        // TODO refactor the initialization code into an init() function?
        // find the range for our executable code
        let text_range = self
            .sections
            .get(".text")
            .ok_or_else(|| EmulatorError::Execution("no .text section found".into()))?
            .clone();

        // set the global pointer address
        if let Some(gp) = self.symtab.get(GLOBAL_POINTER_SYM) {
            log::debug!("global pointer address: 0x{:x}", gp);
            self[Reg::gp] = *gp as u32;
        } else {
            log::warn!("global pointer address not found");
        }

        // determine where we should start executing code
        if let Some(pc) = self.symtab.get(ENTRYPOINT_SYM) {
            log::debug!("program entrypoint: 0x{:x}", pc);
            self.pc = *pc;
        } else {
            log::warn!(
                "program entrypoint {} not found; falling back to beginning of .text section: {:x}",
                ENTRYPOINT_SYM,
                text_range.start
            );
            self.pc = text_range.start;
        }

        // stack pointer in the middle?
        self[Reg::sp] = (self.mem.len() / 2) as u32;

        while text_range.contains(&self.pc) {
            if log::log_enabled!(log::Level::Trace) {
                // dump registers
                log::trace!("{self:?}");
            }

            let inst = self.curr()?;

            if log::log_enabled!(log::Level::Debug) {
                let word = self[self.pc];
                log::debug!("{:x}: {:08x} {:.*}", self.pc, word, self.pc, inst);
            }

            inst.execute(self);

            self.pc += 4;
        }

        if text_range.contains(&self.pc) {
            Ok(())
        } else {
            Err(EmulatorError::Execution(format!(
                "program counter outside bounds of .text section: {:08x}",
                self.pc
            )))
        }
    }

    /// Returns the current instruction - i.e., the instruction the program
    /// counter is currently pointing at.
    pub fn curr(&self) -> Result<Inst, EmulatorError> {
        self.inst(self.pc)
    }

    /// Returns the instruction at memory address `addr`.
    pub fn inst(&self, addr: usize) -> Result<Inst, EmulatorError> {
        let word: u32 = *bytemuck::from_bytes(&self[addr..addr + 4]);
        Inst::try_from(word)
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            pc: Default::default(),
            reg: Default::default(),
            mem: vec![0u8; DEFAULT_MEMORY_SIZE],
            sections: Default::default(),
            symtab: Default::default(),
            dev_null: Default::default(),
        }
    }
}

impl Index<Reg> for Emulator {
    type Output = u32;

    fn index(&self, index: Reg) -> &Self::Output {
        if index == Reg::zero {
            &0u32
        } else {
            &self.reg[index as usize]
        }
    }
}

impl IndexMut<Reg> for Emulator {
    fn index_mut(&mut self, index: Reg) -> &mut Self::Output {
        if index == Reg::zero {
            self.dev_null = 0u32;
            &mut self.dev_null
        } else {
            &mut self.reg[index as usize]
        }
    }
}

impl Index<usize> for Emulator {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mem[index]
    }
}

impl IndexMut<usize> for Emulator {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.mem[index]
    }
}

impl Index<Range<usize>> for Emulator {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.mem[index]
    }
}

impl IndexMut<Range<usize>> for Emulator {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.mem[index]
    }
}

impl std::fmt::Debug for Emulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // default behavior: dump PC and registers
        write!(f, "PC: 0x{:x} ", self.pc)?;
        for reg in Reg::iter() {
            write!(f, " {}: 0x{:x}", reg, self[reg])?;
        }

        // alternate behavior: also dump all sections in memory
        if f.alternate() {
            if let Some(range) = self.sections.get(".text") {
                write!(f, "\n.text:")?;
                let mut i = range.start;
                while i < range.end {
                    let word: u32 = *bytemuck::from_bytes(&self[i..i + 4]);
                    let inst = Inst::try_from(word).unwrap();
                    write!(f, "\n  {:x}: {:08x} {:.*}", i, word, i, inst)?;

                    i += 4;
                }
            }
            for (name, range) in &self.sections {
                if name != ".text" {
                    write!(f, "\n{name}")?;
                    let mut i = range.start;
                    while i < range.end {
                        write!(f, "\n  {:x}: {:02x}", i, self[i])?;
                        i += 1;
                    }
                }
            }
            write!(f, "\nSymbols:")?;
            let mut sorted: Vec<(&String, &usize)> = self.symtab.iter().collect();
            sorted.sort_by(|a, b| a.1.cmp(b.1));
            for (sym, addr) in sorted {
                write!(f, "\n  {:08x}: {}", addr, sym)?;
            }
        }
        Ok(())
    }
}

/// Errors encountered while loading or emulating a program.
#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("{0}")]
    IO(#[from] io::Error),

    #[error("error parsing ELF data: {0}")]
    ELF(#[from] goblin::error::Error),

    #[error("program entrypoint could not be located")]
    EntryPoint,

    #[error("instruction could not be decoded: {0}")]
    InstructionDecode(String),

    #[error("execution error: {0}")]
    Execution(String),
}

// rv32i
impl Emulator {
    fn nop(&mut self) {
        log::warn!("nop called");
    }

    /* B-Type (branches) */
    fn beq(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        if self[rs1] == self[rs2] {
            self.pc = (self.pc as i32 + imm - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn bne(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        if self[rs1] != self[rs2] {
            self.pc = (self.pc as i32 + imm - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn blt(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        if (self[rs1] as i32) < (self[rs2] as i32) {
            self.pc = (self.pc as i32 + imm - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn bge(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        if (self[rs1] as i32) >= (self[rs2] as i32) {
            self.pc = (self.pc as i32 + imm - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn bltu(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        if self[rs1] < self[rs2] {
            self.pc = (self.pc as i32 + imm - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn bgeu(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        if self[rs1] >= self[rs2] {
            self.pc = (self.pc as i32 + imm - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }

    /* I-Type */

    // integer operations
    fn addi(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        self[rd] = ((self[rs1] as i32) + imm) as u32;
    }
    fn andi(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        self[rd] = self[rs1] & (imm as u32);
    }
    fn ori(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        self[rd] = self[rs1] | (imm as u32);
    }
    fn slti(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        self[rd] = if (self[rs1] as i32) < imm { 1 } else { 0 };
    }
    fn sltiu(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        self[rd] = if self[rs1] < (imm as u32) { 1 } else { 0 };
    }
    fn xori(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        self[rd] = self[rs1] ^ (imm as u32)
    }

    // loads
    fn lb(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        let addr = ((self[rs1] as i32) + imm) as usize;
        let val = self[addr] as u32;
        self[rd] = sext(val, 8);
    }
    fn lh(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        let addr = ((self[rs1] as i32) + imm) as usize;
        let val = (self[addr] as u32) | ((self[addr + 1] as u32) << 8);
        self[rd] = sext(val, 16);
    }
    fn lw(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        let addr = ((self[rs1] as i32) + imm) as usize;
        self[rd] = *bytemuck::from_bytes(&self[addr..addr + 4]);
    }
    fn lbu(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        let addr = ((self[rs1] as i32) + imm) as usize;
        let val = self[addr] as u32;
        self[rd] = val;
    }
    fn lhu(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        let addr = ((self[rs1] as i32) + imm) as usize;
        let val = (self[addr] as u32) | ((self[addr + 1] as u32) << 8);
        self[rd] = val;
    }

    // jump
    fn jalr(&mut self, rd: Reg, rs1: Reg, imm: i32) {
        let addr = ((self[rs1] as i32) + imm) as usize;
        self[rd] = self.pc as u32 + 4;
        self.pc = (addr - 4) as usize; // NB subtract 4 since we're auto-incrementing
    }

    /* J-Type */
    fn jal(&mut self, rd: Reg, imm: i32) {
        self[rd] = (self.pc + 4) as u32;
        let addr = self.pc as i32 + imm - 4; // NB subtract 4 since we're auto-incrementing
        self.pc = addr as usize;
    }

    /* R-Type */
    fn add(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1].wrapping_add(self[rs2]);
    }
    fn and(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1] & self[rs2];
    }
    fn or(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1] | self[rs2];
    }
    fn sll(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1] << self[rs2];
    }
    fn slt(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = if (self[rs1] as i32) < (self[rs2] as i32) {
            1
        } else {
            0
        };
    }
    fn sltu(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = if self[rs1] < self[rs2] { 1 } else { 0 };
    }
    fn sra(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = ((self[rs1] as i32) >> (self[rs2] as i32)) as u32;
    }
    fn srl(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1] >> self[rs2];
    }
    fn slli(&mut self, rd: Reg, rs1: Reg, shamt: u32) {
        self[rd] = self[rs1] << shamt;
    }
    fn srli(&mut self, rd: Reg, rs1: Reg, shamt: u32) {
        self[rd] = self[rs1] >> shamt;
    }
    fn srai(&mut self, rd: Reg, rs1: Reg, shamt: u32) {
        self[rd] = ((self[rs1] as i32) >> shamt) as u32;
    }
    fn sub(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1].wrapping_sub(self[rs2]);
    }
    fn xor(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1] ^ self[rs2];
    }

    /* S-Type */
    fn sb(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        let addr = (self[rs1] as i32 + imm) as usize;
        let bytes = self[rs2].to_le_bytes();
        self[addr] = bytes[0];
    }
    fn sh(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        let addr = (self[rs1] as i32 + imm) as usize;
        let bytes = self[rs2].to_le_bytes();
        self[addr] = bytes[0];
        self[addr + 1] = bytes[1];
    }
    fn sw(&mut self, rs1: Reg, rs2: Reg, imm: i32) {
        let addr = (self[rs1] as i32 + imm) as usize;
        let bytes = self[rs2].to_le_bytes();
        self[addr] = bytes[0];
        self[addr + 1] = bytes[1];
        self[addr + 2] = bytes[2];
        self[addr + 3] = bytes[3];
    }

    /* U-Type */
    fn auipc(&mut self, rd: Reg, imm: i32) {
        self[rd] = self.pc as u32 + (imm << 12) as u32;
    }
    fn lui(&mut self, rd: Reg, imm: i32) {
        self[rd] = (imm << 12) as u32;
    }

    /* system calls */
    fn ecall(&mut self) {
        let syscall = self[Reg::a7];
        match syscall {
            1 => {
                log::trace!("MIPS print_int"); // https://student.cs.uwaterloo.ca/~isg/res/mips/traps
                print!("{}", (self[Reg::a0] as i32));
                std::io::stdout().flush().unwrap();
            }
            4 => {
                log::trace!("MIPS print_string");
                let pos = self[Reg::a0] as usize;
                let mut len = 0usize;
                while self[pos + len] != 0 {
                    len += 1;
                }

                print!(
                    "{}",
                    String::from_utf8(self[pos..pos + len].into()).unwrap()
                );
                std::io::stdout().flush().unwrap();
            }
            5 => {
                log::trace!("MIPS read_int");
                let mut buf: String = String::new();
                // TODO catch error
                let _ = std::io::stdin().read_line(&mut buf);
                self[Reg::a0] = buf.trim().parse::<u32>().unwrap(); // TODO get rid of unwrap
            }
            10 => {
                log::trace!("MIPS exit");
                process::exit(0);
            }
            64 => {
                // RISC-V write
                log::trace!(
                    "RISC-V linux write syscall: fp: {} addr: {:x} len: {}",
                    self[Reg::a0],
                    self[Reg::a1],
                    self[Reg::a2]
                );

                let mut fp = unsafe { File::from_raw_fd(self[Reg::a0] as i32) };
                let addr = self[Reg::a1] as usize;
                let len = self[Reg::a2] as usize;
                if let Ok(len) = fp.write(&self[addr..addr + len]) {
                    log::trace!("wrote {} bytes", len);
                    self[Reg::a0] = len as u32;
                } else {
                    log::trace!("write error");
                    self[Reg::a0] = -1i32 as u32;
                }
            }
            93 => {
                // RISC-V exit
                log::trace!("RISC-V linux exit syscall: rc: {}", self[Reg::a0]);
                process::exit(self[Reg::a0] as i32);
            }
            _ => {
                log::error!("unknown/unimplemented syscall: {}", syscall);
            }
        }
    }
}

#[cfg(feature = "rv32m")]
impl Emulator {
    // NB all multiplication extensions are R-Type
    fn mul(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = ((self[rs1] as i32) * (self[rs2] as i32)) as u32;
    }
    fn mulh(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = (((self[rs1] as i64) * (self[rs2] as i64)) >> 32) as u32;
    }
    fn mulhu(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = (((self[rs1] as u64) * (self[rs2] as u64)) >> 32) as u32;
    }
    fn mulhsu(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        // NB I don't think this is quite correct, but I'm fuzzy on what is...
        self[rd] = (((self[rs1] as u64) * (self[rs2] as u64)) >> 32) as u32;
    }
    fn div(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = ((self[rs1] as i32) / (self[rs2] as i32)) as u32;
    }
    fn divu(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1] / self[rs2];
    }
    fn rem(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = ((self[rs1] as i32) % (self[rs2] as i32)) as u32;
    }
    fn remu(&mut self, rd: Reg, rs1: Reg, rs2: Reg) {
        self[rd] = self[rs1] % self[rs2];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Works
    #[test]
    fn test_sext() {
        let value = 0xff0u32;
        assert_eq!(sext(value, 12), 0xfffffff0);

        let value = 0x7f0u32;
        assert_eq!(sext(value, 12), value);
    }
}
