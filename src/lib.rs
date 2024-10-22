use goblin::elf::Elf;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::Range;
use std::os::fd::FromRawFd;
use std::process;

const ENTRYPOINT_SYMNAME: &str = "_start";
const GLOBAL_POINTER_SYMNAME: &str = "__global_pointer$";
const REG_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", /* "fp" */ "s0", "s1", "a0", "a1", "a2",
    "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
    "t3", "t4", "t5", "t6",
];

pub const R_ZERO: usize = 0; /* hardwired to 0, ignores writes    */
pub const R_RA: usize = 1; /*   return address for jumps          */
pub const R_SP: usize = 2; /*   stack pointer	                  */
pub const R_GP: usize = 3; /*   global pointer                    */
pub const R_TP: usize = 4; /*   thread pointer                    */
pub const R_T0: usize = 5; /*   temporary register 0              */
pub const R_T1: usize = 6; /*   temporary register 1              */
pub const R_T2: usize = 7; /*   temporary register 2              */
pub const R_FP: usize = 8; /*   saved register 0/frame pointer    */
pub const R_S0: usize = 8; /*   saved register 0/frame pointer    */
pub const R_S1: usize = 9; /*   saved register 1                  */
pub const R_A0: usize = 10; /*  return value/function argument 0  */
pub const R_A1: usize = 11; /*  return value/function argument 1  */
pub const R_A2: usize = 12; /*  function argument 2               */
pub const R_A3: usize = 13; /*  function argument 3               */
pub const R_A4: usize = 14; /*  function argument 4               */
pub const R_A5: usize = 15; /*  function argument 5               */
pub const R_A6: usize = 16; /*  function argument 6               */
pub const R_A7: usize = 17; /*  function argument 7               */
pub const R_S2: usize = 18; /*  saved register 2                  */
pub const R_S3: usize = 19; /*  saved register 3                  */
pub const R_S4: usize = 20; /*  saved register 4                  */
pub const R_S5: usize = 21; /*  saved register 5                  */
pub const R_S6: usize = 22; /*  saved register 6                  */
pub const R_S7: usize = 23; /*  saved register 7                  */
pub const R_S8: usize = 24; /*  saved register 8                  */
pub const R_S9: usize = 25; /*  saved register 9                  */
pub const R_S10: usize = 26; /* saved register 10                 */
pub const R_S11: usize = 27; /* saved register 11                 */
pub const R_T3: usize = 28; /*  temporary register 3              */
pub const R_T4: usize = 29; /*  temporary register 4              */
pub const R_T5: usize = 30; /*  temporary register 5              */
pub const R_T6: usize = 31; /*  temporary register 6              */

pub const DEFAULT_MEMORY_SIZE: usize = 1 << 20;

// enum Instruction
include!(concat!(env!("OUT_DIR"), "/enum.rs"));

impl Instruction {
    fn execute(&self, vm: &mut VirtualMachine) -> Result<usize, String> {
        match self {
            /* B-Type (branches) */
            Instruction::BEQ { rs1, rs2, imm } => Ok(vm.pc
                + if vm.reg[*rs1] as i32 == vm.reg[*rs2] as i32 {
                    sext!(*imm, 12) as usize
                } else {
                    4
                }),
            Instruction::BNE { rs1, rs2, imm } => Ok(vm.pc
                + if vm.reg[*rs1] as i32 != vm.reg[*rs2] as i32 {
                    sext!(*imm, 12) as usize
                } else {
                    4
                }),
            Instruction::BLT { rs1, rs2, imm } => Ok(vm.pc
                + if (vm.reg[*rs1] as i32) < (vm.reg[*rs2] as i32) {
                    sext!(*imm, 12) as usize
                } else {
                    4
                }),
            Instruction::BGE { rs1, rs2, imm } => Ok(vm.pc
                + if vm.reg[*rs1] as i32 >= vm.reg[*rs2] as i32 {
                    sext!(*imm, 12) as usize
                } else {
                    4
                }),
            Instruction::BLTU { rs1, rs2, imm } => Ok(vm.pc
                + if vm.reg[*rs1] < vm.reg[*rs2] {
                    sext!(*imm, 12) as usize
                } else {
                    4
                }),
            Instruction::BGEU { rs1, rs2, imm } => Ok(vm.pc
                + if vm.reg[*rs1] >= vm.reg[*rs2] {
                    sext!(*imm, 12) as usize
                } else {
                    4
                }),

            /* I-Type */
            // integer operations
            Instruction::ADDI { rd, rs1, imm } => {
                vm.reg[*rd] = ((vm.reg[*rs1] as i32) + (sext!(*imm, 12) as i32)) as u32;
                Ok(vm.pc + 4)
            }
            Instruction::ANDI { rd, rs1, imm } => {
                vm.reg[*rd] = ((vm.reg[*rs1] as i32) & (sext!(*imm, 12) as i32)) as u32;
                Ok(vm.pc + 4)
            }
            Instruction::ORI { rd, rs1, imm } => {
                vm.reg[*rd] = ((vm.reg[*rs1] as i32) | (sext!(*imm, 12) as i32)) as u32;
                Ok(vm.pc + 4)
            }
            Instruction::SLTI { rd, rs1, imm } => {
                vm.reg[*rd] = if (vm.reg[*rs1] as i32) < (sext!(*imm, 12) as i32) {
                    1
                } else {
                    0
                };
                Ok(vm.pc + 4)
            }
            Instruction::SLTIU { rd, rs1, imm } => {
                vm.reg[*rd] = if vm.reg[*rs1] < sext!(*imm, 12) { 1 } else { 0 };
                Ok(vm.pc + 4)
            }
            Instruction::XORI { rd, rs1, imm } => {
                vm.reg[*rd] = ((vm.reg[*rs1] as i32) ^ (sext!(*imm, 12) as i32)) as u32;
                Ok(vm.pc + 4)
            }

            // shamts
            Instruction::SLLI { rd, rs1, shamt } => {
                vm.reg[*rd] = vm.reg[*rs1] << *shamt;
                Ok(vm.pc + 4)
            }
            Instruction::SRLI { rd, rs1, shamt } => {
                vm.reg[*rd] = vm.reg[*rs1] >> *shamt;
                Ok(vm.pc + 4)
            }
            Instruction::SRAI { rd, rs1, shamt } => {
                vm.reg[*rd] = ((vm.reg[*rs1] as i32) << *shamt) as u32;
                Ok(vm.pc + 4)
            }

            // loads
            Instruction::LB { rd, rs1, imm } => {
                let addr = (vm.reg[*rs1] + sext!(*imm, 12)) as usize;
                vm.reg[*rd] = sext!(vm.mem[addr] as u32, 8);
                Ok(vm.pc + 4)
            }
            Instruction::LH { rd, rs1, imm } => {
                let addr = (vm.reg[*rs1] + sext!(*imm, 12)) as usize;
                vm.reg[*rd] = vm.mem[addr] as u32;
                vm.reg[*rd] |= sext!((vm.mem[addr + 1] as u32) << 8, 16);
                Ok(vm.pc + 4)
            }
            Instruction::LW { rd, rs1, imm } => {
                let addr = (vm.reg[*rs1] + sext!(*imm, 12)) as usize;
                vm.reg[*rd] = u32::from_le_bytes(vm.mem[addr..addr + 4].try_into().unwrap());
                Ok(vm.pc + 4)
            }
            Instruction::LBU { rd, rs1, imm } => {
                let addr = (vm.reg[*rs1] + sext!(*imm, 12)) as usize;
                vm.reg[*rd] = vm.mem[addr] as u32;
                Ok(vm.pc + 4)
            }
            Instruction::LHU { rd, rs1, imm } => {
                let addr = (vm.reg[*rs1] + sext!(*imm, 12)) as usize;
                vm.reg[*rd] = vm.mem[addr] as u32;
                vm.reg[*rd] |= (vm.mem[addr + 1] as u32) << 8;
                Ok(vm.pc + 4)
            }

            // jump
            Instruction::JALR { rd, rs1, imm } => {
                vm.reg[*rd] = vm.pc as u32 + 4;
                Ok((vm.reg[*rs1] + sext!(*imm, 12)) as usize)
            }

            /* J-Type */
            Instruction::JAL { rd, imm } => {
                vm.reg[*rd] = (vm.pc + 4) as u32;
                Ok((vm.pc as u32).wrapping_add(sext!(*imm, 20)) as usize)
            }

            /* R-Type */
            Instruction::ADD { rd, rs1, rs2 } => {
                vm.reg[*rd] = vm.reg[*rs1] + vm.reg[*rs2];
                Ok(vm.pc + 4)
            }
            Instruction::AND { rd, rs1, rs2 } => {
                vm.reg[*rd] = vm.reg[*rs1] & vm.reg[*rs2];
                Ok(vm.pc + 4)
            }
            Instruction::OR { rd, rs1, rs2 } => {
                vm.reg[*rd] = vm.reg[*rs1] | vm.reg[*rs2];
                Ok(vm.pc + 4)
            }
            Instruction::SLL { rd, rs1, rs2 } => {
                vm.reg[*rd] = vm.reg[*rs1] << vm.reg[*rs2];
                Ok(vm.pc + 4)
            }
            Instruction::SLT { rd, rs1, rs2 } => {
                vm.reg[*rd] = if (vm.reg[*rs1] as i32) < (vm.reg[*rs2] as i32) {
                    1
                } else {
                    0
                };
                Ok(vm.pc + 4)
            }
            Instruction::SLTU { rd, rs1, rs2 } => {
                vm.reg[*rd] = if vm.reg[*rs1] < vm.reg[*rs2] { 1 } else { 0 };
                Ok(vm.pc + 4)
            }
            Instruction::SRL { rd, rs1, rs2 } => {
                vm.reg[*rd] = vm.reg[*rs1] >> vm.reg[*rs2];
                Ok(vm.pc + 4)
            }
            Instruction::SRA { rd, rs1, rs2 } => {
                vm.reg[*rd] = ((vm.reg[*rs1] as i32) >> vm.reg[*rs2]) as u32;
                Ok(vm.pc + 4)
            }
            Instruction::SUB { rd, rs1, rs2 } => {
                vm.reg[*rd] = vm.reg[*rs1] - vm.reg[*rs2];
                Ok(vm.pc + 4)
            }
            Instruction::XOR { rd, rs1, rs2 } => {
                vm.reg[*rd] = vm.reg[*rs1] ^ vm.reg[*rs2];
                Ok(vm.pc + 4)
            }

            /* S-Type */
            Instruction::SB { rs1, rs2, imm } => {
                let addr = vm.reg[*rs1].wrapping_add(sext!(*imm, 12)) as usize;
                vm.mem[addr] = (vm.reg[*rs2] & 0xff) as u8;
                Ok(vm.pc + 4)
            }
            Instruction::SH { rs1, rs2, imm } => {
                let addr = vm.reg[*rs1].wrapping_add(sext!(*imm, 12)) as usize;
                vm.mem[addr] = (vm.reg[*rs2] & 0xff) as u8;
                vm.mem[addr + 1] = ((vm.reg[*rs2] & 0xff00) << 8) as u8;
                Ok(vm.pc + 4)
            }
            Instruction::SW { rs1, rs2, imm } => {
                let addr = vm.reg[*rs1].wrapping_add(sext!(*imm, 12)) as usize;
                vm.mem[addr] = (vm.reg[*rs2] & 0xff) as u8;
                vm.mem[addr + 1] = ((vm.reg[*rs2] & 0xff00) << 8) as u8;
                vm.mem[addr + 2] = ((vm.reg[*rs2] & 0xffff00) << 8) as u8;
                vm.mem[addr + 3] = ((vm.reg[*rs2] & 0xffffff00) << 8) as u8;
                Ok(vm.pc + 4)
            }

            /* U-Type */
            Instruction::AUIPC { rd, imm } => {
                vm.reg[*rd] = vm.pc as u32 + (imm << 12);
                Ok(vm.pc + 4)
            }
            Instruction::LUI { rd, imm } => {
                vm.reg[*rd] = imm << 12;
                Ok(vm.pc + 4)
            }

            /* syscalls */
            Instruction::ECALL => {
                let syscall = vm.reg[R_A7];
                match syscall {
                    1 => {
                        log::trace!("MIPS print_int"); // https://student.cs.uwaterloo.ca/~isg/res/mips/traps
                        println!("{}", (vm.reg[R_A0] as i32));
                        std::io::stdout().flush().unwrap();
                    }
                    4 => {
                        log::trace!("MIPS print_string");
                        let pos = vm.reg[R_A0] as usize;
                        let mut len = 0usize;
                        while vm.mem[pos + len] != 0 {
                            len += 1;
                        }

                        print!(
                            "{}",
                            String::from_utf8(vm.mem[pos..pos + len].into()).unwrap()
                        );
                        std::io::stdout().flush().unwrap();
                    }
                    5 => {
                        log::trace!("MIPS read_int");
                        let mut buf: String = String::new();
                        // TODO catch error
                        let _ = std::io::stdin().read_line(&mut buf);
                        vm.reg[R_A0] = buf.trim().parse::<u32>().unwrap(); // TODO get rid of unwrap
                    }
                    10 => {
                        log::trace!("MIPS exit");
                        process::exit(0);
                    }
                    64 => {
                        // RISC-V write
                        log::trace!(
                            "RISC-V linux write syscall: fp: {} addr: {:x} len: {}",
                            vm.reg[R_A0],
                            vm.reg[R_A1],
                            vm.reg[R_A2]
                        );

                        let mut fp = unsafe { File::from_raw_fd(vm.reg[R_A0] as i32) };
                        let addr = vm.reg[R_A1] as usize;
                        let len = vm.reg[R_A2] as usize;
                        if let Ok(len) = fp.write(&vm.mem[addr..addr + len]) {
                            log::trace!("wrote {} bytes", len);
                            vm.reg[R_A0] = len as u32;
                        } else {
                            log::trace!("write error");
                            vm.reg[R_A0] = 0xffffff; // -1
                        }
                    }
                    93 => {
                        // RISC-V exit
                        log::trace!("RISC-V linux exit syscall: rc: {}", vm.reg[R_A0]);
                        process::exit(vm.reg[R_A0] as i32);
                    }
                    _ => {
                        return Err(format!("unknown/unimplemented syscall: {}", syscall));
                    }
                }
                Ok(vm.pc + 4)
            }
            _ => Err("unimplemented".into()),
        }
    }
}

// impl TryFrom<u32> for Instruction
include!(concat!(env!("OUT_DIR"), "/decode.rs"));

pub struct VirtualMachine {
    pub pc: usize,
    pub reg: [u32; 32],
    pub mem: Vec<u8>,
    pub sections: HashMap<String, Range<usize>>,
    pub symtab: HashMap<String, usize>,
}

impl VirtualMachine {
    pub fn new(alloc: Option<usize>) -> VirtualMachine {
        VirtualMachine {
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
        }
    }

    pub fn load_from(
        path: &str,
        alloc: Option<usize>,
    ) -> Result<VirtualMachine, Box<dyn std::error::Error>> {
        let mut vm = VirtualMachine::new(alloc);
        vm.load(path)?;
        Ok(vm)
    }

    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let elf = Elf::parse(&buf)?;

        for section in &elf.section_headers {
            if section.is_alloc() {
                let name = elf.shdr_strtab.get_at(section.sh_name).unwrap().to_string();
                log::debug!(
                    "found section: {}; address: 0x{:x}, length: {} bytes",
                    name,
                    section.sh_addr,
                    section.sh_size
                );
                self.mem[section.vm_range()].copy_from_slice(&buf[section.file_range().unwrap()]);
                self.sections.insert(name, section.vm_range());
            }
        }

        for sym in elf.syms.iter() {
            if let Some(name) = elf.strtab.get_at(sym.st_name) {
                self.symtab.insert(name.into(), sym.st_value as usize);
            }
        }

        if let Some(gp) = self.symtab.get(GLOBAL_POINTER_SYMNAME) {
            log::debug!("global pointer address: 0x{:x}", gp);
            self.reg[R_GP] = *gp as u32;
        } else {
            log::warn!("global pointer address not found");
        }

        if let Some(pc) = self.symtab.get(ENTRYPOINT_SYMNAME) {
            log::debug!("program entrypoint: 0x{:x}", pc);
            self.pc = *pc;
        } else if let Some(range) = self.sections.get(".text") {
            log::warn!(
                "program entrypoint {} not found; falling back to beginning of .text section: {:x}",
                ENTRYPOINT_SYMNAME,
                range.start
            );
            self.pc = range.start;
        } else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "program entrypoint could not be determined",
            )));
        }

        Ok(())
    }

    pub fn curr(&self) -> u32 {
        self.inst(self.pc)
    }

    pub fn inst(&self, addr: usize) -> u32 {
        u32::from_le_bytes(self.mem[addr..addr + 4].try_into().unwrap())
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self {
            pc: Default::default(),
            reg: Default::default(),
            mem: vec![0u8; DEFAULT_MEMORY_SIZE],
            sections: Default::default(),
            symtab: Default::default(),
        }
    }
}

impl std::fmt::Debug for VirtualMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // default behavior: dump PC and registers
        write!(f, "PC: 0x{:x} ", self.pc)?;
        for i in 0..self.reg.len() {
            write!(f, " {}: 0x{:x}", REG_NAMES[i], self.reg[i])?;
        }

        // alternate behavior: also dump all sections in memory
        if f.alternate() {
            if let Some(range) = self.sections.get(".text") {
                write!(f, "\n.text:")?;
                let mut i = range.start;
                while i < range.end {
                    let word = u32::from_le_bytes(self.mem[i..i + 4].try_into().unwrap());
                    let inst = Instruction::try_from(word).unwrap();
                    write!(f, "\n  {:x}: {:08x} {:?}", i, word, inst)?;

                    i += 4;
                }
            }
            for (name, range) in &self.sections {
                if name != ".text" {
                    write!(f, "\n{name}")?;
                    let mut i = range.start;
                    while i < range.end {
                        write!(f, "\n  {:x}: {:02x}", i, self.mem[i])?;
                        i += 1;
                    }
                }
            }
        }
        Ok(())
    }
}

fn disassemble(pc: usize, inst: u32) -> String {
    // let opcode = opcode!(inst);
    // include!(concat!(env!("OUT_DIR"), "/disasm.rs"))
    "".into()
}

impl VirtualMachine {
    pub fn run(&mut self) -> Result<(), String> {
        while self.mem[self.pc] != 0 {
            // we'll just reset to zero each iteration rather than blocking writes
            self.reg[0] = 0;

            if log::log_enabled!(log::Level::Trace) {
                // dump registers
                log::trace!("{self:?}");
            }

            let word = self.curr();
            let inst = Instruction::try_from(word).unwrap();
            // let opcode = opcode!(inst);

            if log::log_enabled!(log::Level::Debug) {
                log::debug!(
                    "{:x}: {:08x} {:?} ({})",
                    self.pc,
                    word,
                    inst,
                    disassemble(self.pc, word)
                );
            }

            self.pc = inst.execute(self)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! opcode {
    ($inst:expr) => {
        ($inst) & 0b111_1111
    };
}

#[macro_export]
macro_rules! rd {
    ($inst:expr) => {
        ((($inst >> 7) & 0b1_1111) as usize)
    };
}

#[macro_export]
macro_rules! rs1 {
    ($inst:expr) => {
        ((($inst >> 15) & 0b1_1111) as usize)
    };
}

#[macro_export]
macro_rules! rs2 {
    ($inst:expr) => {
        ((($inst >> 20) & 0b1_1111) as usize)
    };
}

// same as rs2!, but as u32
#[macro_export]
macro_rules! shamt {
    ($inst:expr) => {
        (($inst >> 20) & 0b1_1111)
    };
}

#[macro_export]
macro_rules! funct3 {
    ($inst:expr) => {
        (($inst >> 12) & 0b111)
    };
}

#[macro_export]
macro_rules! funct7 {
    ($inst:expr) => {
        (($inst >> 25) & 0b111_1111)
    };
}

#[macro_export]
macro_rules! imm_b {
    ($inst:expr) => {
        ((((($inst) >> 31) & 0x1) << 12)
            | (((($inst) >> 7) & 0b1) << 11)
            | (((($inst) >> 25) & 0b111111) << 5)
            | (((($inst) >> 8) & 0b1111) << 1))
    };
}

#[macro_export]
macro_rules! imm_j {
    ($inst:expr) => {
        ((((($inst) >> 31) & 0b1) << 20)
            | (((($inst) >> 12) & 0b11111111) << 12)
            | (((($inst) >> 20) & 0b1) << 11)
            | (((($inst) >> 21) & 0b1111111111) << 1))
    };
}

#[macro_export]
macro_rules! imm_s {
    ($inst:expr) => {
        (((($inst) >> 25) << 7) | ((($inst) >> 7) & 0b11111))
    };
}

#[macro_export]
macro_rules! sext {
    ($value:expr, $bits:expr) => {
        if (($value) & (1 << (($bits) - 1))) == 0 {
            $value
        } else {
            (($value) | (0xffffffff << ($bits)))
        }
    };
}
