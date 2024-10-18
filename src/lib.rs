use goblin::elf::Elf;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::process;

macro_rules! opcode {
    ($inst:expr) => {
        ($inst) & 0b111_1111
    };
}

macro_rules! rd {
    ($inst:expr) => {
        ((($inst >> 7) & 0b1_1111) as usize)
    };
}

macro_rules! rs1 {
    ($inst:expr) => {
        ((($inst >> 15) & 0b1_1111) as usize)
    };
}

macro_rules! rs2 {
    ($inst:expr) => {
        ((($inst >> 20) & 0b1_1111) as usize)
    };
}

macro_rules! funct3 {
    ($inst:expr) => {
        (($inst >> 12) & 0b111)
    };
}

macro_rules! funct7 {
    ($inst:expr) => {
        (($inst >> 25) & 0b111_1111)
    };
}

macro_rules! imm_b {
    ($inst:expr) => {
        ((((($inst) >> 31) & 0x1) << 12) | (((($inst) >> 7) & 0b1) << 11) | (((($inst) >> 25) & 0b111111) << 5) | (((($inst) >> 8) & 0b1111) << 1))
    };
}

macro_rules! imm_j {
    ($inst:expr) => {
        ((((($inst) >> 31) & 0b1) << 20) | (((($inst) >> 12) & 0b11111111) << 12) | (((($inst) >> 20) & 0b1) << 11) | (((($inst) >> 21) & 0b1111111111) << 1))
    };
}

macro_rules! imm_s {
    ($inst:expr) => {
        (((($inst) >> 25) << 7) | ((($inst) >> 7) & 0b11111))
    };
}

macro_rules! sext {
    ($value:expr, $bits:expr) => {
        if (($value) & (1 << (($bits) - 1))) == 0 {
            $value
        } else {
            (($value) | (0xffffffff << ($bits)))
        }
    };
}

const GLOBAL_POINTER_SYMNAME: &str = "__global_pointer$";
const REG_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", /* "fp" */ "s0", "s1", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6",
    "s7", "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
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

pub struct VirtualMachine {
    pub pc: usize,
    pub reg: [u32; 32],
    pub mem: [u8; 1 << 20],
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        VirtualMachine {
            pc: 0x0,
            reg: [0u32; 32],
            mem: [0u8; 1 << 20],
        }
    }

    pub fn load_from(path: &str) -> Result<VirtualMachine, Box<dyn std::error::Error>> {
        let mut vm = VirtualMachine::new();
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
                // TODO get rid of unwraps
                log::debug!(
                    "found section: {}; address: 0x{:x}, length: {} bytes",
                    elf.shdr_strtab.get_at(section.sh_name).unwrap(),
                    section.sh_addr,
                    section.sh_size
                );
                self.mem[section.vm_range()].copy_from_slice(&buf[section.file_range().unwrap()]);
                if section.is_executable() {
                    self.pc = section.sh_addr as usize;
                }
            }
        }

        for sym in elf.syms.iter() {
            let sym_name = elf.strtab.get_at(sym.st_name);
            match sym_name {
                Some(GLOBAL_POINTER_SYMNAME) => {
                    log::debug!("found global pointer address: 0x{:x}", sym.st_value);
                    self.reg[R_GP] = sym.st_value as u32;
                }
                _ => {}
            }
        }

        // initialize the stack pointer
        self.reg[R_SP] = self.reg.len() as u32; // TODO find a better way to do this

        Ok(())
    }

    pub fn curr(&self) -> u32 {
        // TODO get rid of unwrap
        u32::from_le_bytes(self.mem[self.pc..self.pc + 4].try_into().unwrap())
    }

    pub fn memdump(&self, prefix: &str) {
        let mut i: usize = 0;
        for value in self.mem.iter() {
            if *value != 0 {
                log::trace!("{}{:x}: {:02x}", prefix, i, value);
            }
            i += 1;
        }
    }

    pub fn regdump(&self) {
        let mut s = String::new();
        for i in 0..self.reg.len() {
            s += &format!("{}: 0x{:x} ", REG_NAMES[i], self.reg[i]);
        }
        log::trace!("{}", s);
    }
}

impl VirtualMachine {
    /* B-Type (branches) */
    fn beq(&mut self, rs1: usize, rs2: usize, imm13: u32) {
        log::debug!("{:x} {:08x}: beq {}, {}, {:x}", self.pc, self.curr(), REG_NAMES[rs1], REG_NAMES[rs2], sext!(imm13, 12) + self.pc as u32);
        if self.reg[rs1] == self.reg[rs2] {
            self.pc += (sext!(imm13, 12) - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn bne(&mut self, rs1: usize, rs2: usize, imm13: u32) {
        log::debug!("{:x} {:08x}: bne {}, {}, {:x}", self.pc, self.curr(), REG_NAMES[rs1], REG_NAMES[rs2], sext!(imm13, 12) + self.pc as u32);
        if self.reg[rs1] != self.reg[rs2] {
            self.pc += (sext!(imm13, 12) - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn blt(&mut self, rs1: usize, rs2: usize, imm13: u32) {
        log::debug!("{:x} {:08x}: blt {}, {}, {:x}", self.pc, self.curr(), REG_NAMES[rs1], REG_NAMES[rs2], sext!(imm13, 12) + self.pc as u32);
        if self.reg[rs1] < self.reg[rs2] {
            self.pc += (sext!(imm13, 12) - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn bge(&mut self, rs1: usize, rs2: usize, imm13: u32) {
        log::debug!("{:x} {:08x}: bge {}, {}, {:x}", self.pc, self.curr(), REG_NAMES[rs1], REG_NAMES[rs2], sext!(imm13, 12) + self.pc as u32);
        if self.reg[rs1] >= self.reg[rs2] {
            self.pc += (sext!(imm13, 12) - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn bltu(&mut self, rs1: usize, rs2: usize, imm13: u32) {
        log::debug!("{:x} {:08x}: bltu {}, {}, {:x}", self.pc, self.curr(), REG_NAMES[rs1], REG_NAMES[rs2], sext!(imm13, 12) + self.pc as u32);
        if (self.reg[rs1] as u32) < (self.reg[rs2] as u32) {
            self.pc += (sext!(imm13, 12) - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }
    fn bgeu(&mut self, rs1: usize, rs2: usize, imm13: u32) {
        log::debug!("{:x} {:08x}: bltu {}, {}, {:x}", self.pc, self.curr(), REG_NAMES[rs1], REG_NAMES[rs2], sext!(imm13, 12) + self.pc as u32);
        if (self.reg[rs1] as u32) >= (self.reg[rs2] as u32) {
            self.pc += (sext!(imm13, 12) - 4) as usize; // NB subtract 4 since we're auto-incrementing
        }
    }

    /* I-Type */

    // integer operations
    fn addi(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: addi {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], sext!(imm12, 12) as i32);
        self.reg[rd] = ((self.reg[rs1] as i32) + (sext!(imm12, 12) as i32)) as u32;
    }
    fn andi(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: andi {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], sext!(imm12, 12));
        self.reg[rd] = self.reg[rs1] & sext!(imm12, 12);
    }
    fn ori(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: xori {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], sext!(imm12, 12));
        self.reg[rd] = self.reg[rs1] | sext!(imm12, 12);
    }
    fn slti(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: slti {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], sext!(imm12, 12));
        self.reg[rd] = if self.reg[rs1] < sext!(imm12, 12) { 1 } else { 0 };
    }
    fn sltiu(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: sltiu {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], sext!(imm12, 12));
        self.reg[rd] = if (self.reg[rs1] as u32) < (sext!(imm12, 12) as u32) { 1 } else { 0 };
    }
    fn xori(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: xori {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], sext!(imm12, 12));
        self.reg[rd] = self.reg[rs1] ^ sext!(imm12, 12);
    }

    // loads
    fn lb(&mut self, rd: usize, rs1: usize, imm12: u32) {
        let addr = (self.reg[rs1] + sext!(imm12, 12)) as usize;
        log::debug!("{:x} {:08x}: lb {}, {}({}) # {:x}", self.pc, self.curr(), REG_NAMES[rd], sext!(imm12, 12), REG_NAMES[rs1], addr);
        self.reg[rd] = sext!(self.mem[addr] as u32, 8);
    }
    fn lh(&mut self, rd: usize, rs1: usize, imm12: u32) {
        let addr = (self.reg[rs1] + sext!(imm12, 12)) as usize;
        log::debug!("{:x} {:08x}: lh {}, {}({}) # {:x}", self.pc, self.curr(), REG_NAMES[rd], sext!(imm12, 12), REG_NAMES[rs1], addr);
        self.reg[rd] = self.mem[addr] as u32;
        self.reg[rd] |= sext!((self.mem[addr + 1] as u32) << 8, 16);
    }
    fn lw(&mut self, rd: usize, rs1: usize, imm12: u32) {
        let addr = (self.reg[rs1] + sext!(imm12, 12)) as usize;
        log::debug!("{:x} {:08x}: lw {}, {}({}) # {:x}", self.pc, self.curr(), REG_NAMES[rd], sext!(imm12, 12), REG_NAMES[rs1], addr);
        self.reg[rd] = u32::from_le_bytes(self.mem[addr..addr + 4].try_into().unwrap());
    }
    fn lbu(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: lbu {}, {}({})", self.pc, self.curr(), REG_NAMES[rd], sext!(imm12, 12), REG_NAMES[rs1]);
        let addr = (self.reg[rs1] + sext!(imm12, 12)) as usize;
        self.reg[rd] = self.mem[addr] as u32;
    }
    fn lhu(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: lhu {}, {}({})", self.pc, self.curr(), REG_NAMES[rd], sext!(imm12, 12), REG_NAMES[rs1]);
        let addr = (self.reg[rs1] + sext!(imm12, 12)) as usize;
        self.reg[rd] = self.mem[addr] as u32;
        self.reg[rd] |= (self.mem[addr + 1] as u32) << 8;
    }

    // jump
    fn jalr(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: jalr 0x{:x}", self.pc, self.curr(), self.reg[rs1] + sext!(imm12, 12));
        self.reg[rd] = self.pc as u32 + 4;
        self.pc = (self.reg[rs1] + sext!(imm12, 12) - 4) as usize; // NB subtract 4 since we're auto-incrementing
    }

    /* J-Type */
    fn jal(&mut self, rd: usize, imm20: u32) {
        log::debug!("{:x} {:08x}: jal 0x{:x}", self.pc, self.curr(), (self.pc as u32).wrapping_add(sext!(imm20, 20)));
        self.reg[rd] = (self.pc + 4) as u32;
        let addr = (self.pc as u32).wrapping_add(sext!(imm20, 20)) - 4; // NB subtract 4 since we're auto-incrementing
        self.pc = addr as usize;
    }

    /* R-Type */
    fn add(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: add {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = self.reg[rs1] + self.reg[rs2];
    }
    fn and(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: and {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = self.reg[rs1] & self.reg[rs2];
    }
    fn or(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: or {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = self.reg[rs1] | self.reg[rs2];
    }
    fn sll(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: sll {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = self.reg[rs1] << self.reg[rs2];
    }
    fn slt(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: slt {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = if (self.reg[rs1] as i32) < (self.reg[rs2] as i32) { 1 } else { 0 };
    }
    fn sltu(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: sltu {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = if self.reg[rs1] < self.reg[rs2] { 1 } else { 0 };
    }
    fn sra(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: sra {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = ((self.reg[rs1] as i32) >> (self.reg[rs2] as i32)) as u32;
    }
    fn srl(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: srl {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = self.reg[rs1] >> self.reg[rs2];
    }
    fn sub(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: sub {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = self.reg[rs1] - self.reg[rs2];
    }
    fn xor(&mut self, rd: usize, rs1: usize, rs2: usize) {
        log::debug!("{:x} {:08x}: xor {}, {}, {}", self.pc, self.curr(), REG_NAMES[rd], REG_NAMES[rs1], REG_NAMES[rs2]);
        self.reg[rd] = self.reg[rs1] ^ self.reg[rs2];
    }

    /* S-Type */
    fn sb(&mut self, rs1: usize, rs2: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: sb {}, {}({})", self.pc, self.curr(), REG_NAMES[rs2], sext!(imm12, 12), REG_NAMES[rs1]);
        // M[x[rs1] + sext(imm[11:0])][7:0] = x[rs2][7:0]
        let addr = self.reg[rs1].wrapping_add(sext!(imm12, 12)) as usize;
        self.mem[addr] = (self.reg[rs2] & 0xff) as u8;
    }
    fn sh(&mut self, rs1: usize, rs2: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: sh {}, {}({})", self.pc, self.curr(), REG_NAMES[rs2], sext!(imm12, 12), REG_NAMES[rs1]);
        let addr = self.reg[rs1].wrapping_add(sext!(imm12, 12)) as usize;
        self.mem[addr] = (self.reg[rs2] & 0xff) as u8;
        self.mem[addr + 1] = ((self.reg[rs2] & 0xff00) << 8) as u8;
    }
    fn sw(&mut self, rs1: usize, rs2: usize, imm12: u32) {
        log::debug!("{:x} {:08x}: sw {}, {}({})", self.pc, self.curr(), REG_NAMES[rs2], sext!(imm12, 12), REG_NAMES[rs1]);
        let addr = self.reg[rs1].wrapping_add(sext!(imm12, 12)) as usize;
        self.mem[addr] = (self.reg[rs2] & 0xff) as u8;
        self.mem[addr + 1] = ((self.reg[rs2] & 0xff00) << 8) as u8;
        self.mem[addr + 2] = ((self.reg[rs2] & 0xffff00) << 8) as u8;
        self.mem[addr + 3] = ((self.reg[rs2] & 0xffffff00) << 8) as u8;
    }

    /* U-Type */
    fn auipc(&mut self, rd: usize, imm20: u32) {
        log::debug!("{:x} {:08x}: auipc {}, 0x{:x}", self.pc, self.curr(), REG_NAMES[rd], imm20);
        self.reg[rd] = self.pc as u32 + (imm20 << 12);
    }

    fn lui(&mut self, rd: usize, imm20: u32) {
        self.reg[rd] = imm20 << 12;
    }

    /* system calls */
    fn ecall(&mut self) {
        log::debug!("{:x} {:08x}: ecall", self.pc, self.curr());
        let syscall = self.reg[R_A7];
        match syscall {
            1 => {
                log::trace!("MIPS print_int"); // https://student.cs.uwaterloo.ca/~isg/res/mips/traps
                println!("{}", (self.reg[R_A0] as i32));
                std::io::stdout().flush().unwrap();
            }
            4 => {
                log::trace!("MIPS print_string");
                let pos = self.reg[R_A0] as usize;
                let mut len = 0usize;
                while self.mem[pos + len] != 0 {
                    len += 1;
                }

                print!("{}", String::from_utf8(self.mem[pos..pos + len].into()).unwrap());
                std::io::stdout().flush().unwrap();
            }
            5 => {
                log::trace!("MIPS read_int");
                let mut buf: String = String::new();
                // TODO catch error
                let _ = std::io::stdin().read_line(&mut buf);
                self.reg[R_A0] = buf.trim().parse::<u32>().unwrap(); // TODO get rid of unwrap
            }
            10 => {
                log::trace!("MIPS exit");
                process::exit(0);
            }
            64 => {
                // RISC-V write
                log::debug!("RISC-V linux write syscall: fp: {} addr: {:x} len: {}", self.reg[R_A0], self.reg[R_A1], self.reg[R_A2]);

                let mut fp = unsafe { File::from_raw_fd(self.reg[R_A0] as i32) };
                let addr = self.reg[R_A1] as usize;
                let len = self.reg[R_A2] as usize;
                if let Ok(len) = fp.write(&self.mem[addr..addr + len]) {
                    log::trace!("wrote {} bytes", len);
                    self.reg[R_A0] = len as u32;
                } else {
                    log::trace!("write error");
                    self.reg[R_A0] = 0xffffff; // -1
                }
            }
            93 => {
                // RISC-V exit
                log::trace!("RISC-V linux exit syscall: rc: {}", self.reg[R_A0]);
                process::exit(self.reg[R_A0] as i32);
            }
            _ => {
                log::error!("unknown/unimplemented syscall: {}", syscall);
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/rv32i.rs"));
