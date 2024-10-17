use goblin::elf::Elf;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::process;

macro_rules! opcode {
    ($value:expr) => {
        ($value) & 0b111_1111
    };
}

macro_rules! rd {
    ($value:expr) => {
        ((($value >> 7) & 0b1_1111) as usize)
    };
}

macro_rules! rs1 {
    ($value:expr) => {
        ((($value >> 15) & 0b1_1111) as usize)
    };
}

macro_rules! funct3 {
    ($value:expr) => {
        ((($value >> 12) & 0b111) as usize)
    };
}

// macro_rules! rs2 {
//     ($value:expr) => {
//         ((($value >> 7) & 0b1_1111) as usize)
//     };
// }

macro_rules! sext {
    ($value:expr, $bits:expr) => {
        if (($value) & (1 << (($bits) - 1))) == 0 {
            $value
        } else {
            (($value) & (0xffffff << ($bits)))
        }
    };
}

const GLOBAL_POINTER_SYMNAME: &str = "__global_pointer$";
const REG_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", /* "fp" */ "s0", "s1", "a0", "a1", "a2", "a3", "a4", "a5",
    "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
];

const R_ZERO: usize = 0;
const R_RA: usize = 1;
const R_SP: usize = 2;
const R_GP: usize = 3;
const R_TP: usize = 4;
const R_T0: usize = 5;
const R_T1: usize = 6;
const R_T2: usize = 7;
const R_FP: usize = 8;
const R_S0: usize = 8;
const R_S1: usize = 9;
const R_A0: usize = 10;
const R_A1: usize = 11;
const R_A2: usize = 12;
const R_A3: usize = 13;
const R_A4: usize = 14;
const R_A5: usize = 15;
const R_A6: usize = 16;
const R_A7: usize = 17;
const R_S2: usize = 18;
const R_S3: usize = 19;
const R_S4: usize = 20;
const R_S5: usize = 21;
const R_S6: usize = 22;
const R_S7: usize = 23;
const R_S8: usize = 24;
const R_S9: usize = 25;
const R_S10: usize = 26;
const R_S11: usize = 27;
const R_T3: usize = 28;
const R_T4: usize = 29;
const R_T5: usize = 30;
const R_T6: usize = 31;

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
                    self.reg[3/*Reg::Gp*/] = sym.st_value as u32;
                }
                _ => {}
            }
        }

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
}

impl VirtualMachine {
    /* I-Type */
    fn addi(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!(
            "{:x} {:08x}: addi {}, {}, {}",
            self.pc,
            self.curr(),
            REG_NAMES[rd],
            REG_NAMES[rs1],
            sext!(imm12, 12)
        );
        self.reg[rd] = self.reg[rs1] + sext!(imm12, 12);
    }

    /* U-Type */
    fn auipc(&mut self, rd: usize, imm: u32) {
        log::debug!("{:x} {:08x}: auipc {}, 0x{:x}", self.pc, self.curr(), REG_NAMES[rd], imm);
        self.reg[rd] = self.pc as u32 + (imm << 12);
    }

    fn lui(&mut self, rd: usize, imm: u32) {
        self.reg[rd] = imm << 12;
    }
    /* system calls */
    fn ecall(&mut self) {
        log::debug!("{:x} {:08x}: ecall", self.pc, self.curr());
        let syscall = self.reg[R_A7];
        match syscall {
            64 => {
                // RISC-V write
                log::debug!("write syscall: fp: {} addr: {:x} len: {}", self.reg[R_A0], self.reg[R_A1], self.reg[R_A2]);

                let mut fp = unsafe { File::from_raw_fd(self.reg[R_A0] as i32) };
                let addr = self.reg[R_A1] as usize;
                let len = self.reg[R_A2] as usize;
                if let Ok(len) = fp.write(&self.mem[addr..addr + len]) {
                    log::trace!("wrote {} bytes", len);
                    self.reg[R_A0] = len as u32;
                } else {
                    log::trace!("write error");
                    self.reg[R_A0] = 0xffffffff;
                }
            }
            93 => {
                // RISC-V exit
                log::trace!("exit syscall: rc: {}", self.reg[R_A0]);
                process::exit(self.reg[R_A0] as i32);
            }
            _ => {
                log::error!("unknown/unimplemented syscall: {}", syscall);
            }
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/rv32i.rs"));
