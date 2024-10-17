use goblin::elf::Elf;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fs::File;
use std::io::{Read, Write};
use std::{convert::TryFrom, os::fd::FromRawFd};
use std::{env, process};
use strum_macros::{Display, EnumString};

const GLOBAL_POINTER_SYMNAME: &str = "__global_pointer$";

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

macro_rules! rs2 {
    ($value:expr) => {
        ((($value >> 7) & 0b1_1111) as usize)
    };
}

macro_rules! sext {
    ($value:expr, $bits:expr) => {
        if (($value) & (1 << (($bits) - 1))) == 0 {
            $value
        } else {
            (($value) & (0xffffff << ($bits)))
        }
    };
}

#[derive(Debug, Display, EnumString, TryFromPrimitive, IntoPrimitive)]
#[strum(serialize_all = "lowercase")]
#[repr(usize)]
enum Reg {
    Zero = 0,
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0, /* Fp */
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
}

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

    pub fn run(&mut self) {
        while self.mem[self.pc] != 0 {
            let inst = self.curr();

            let opcode = opcode!(inst);
            match opcode {
                0b001_0111 => self.auipc(rd!(inst), inst >> 12),
                0b001_0011 => {
                    let funct3 = funct3!(inst);
                    match funct3 {
                        0b000 => self.addi(rd!(inst), rs1!(inst), inst >> 20),
                        _ => {
                            log::error!(
                                "{:x} {:08x}: unknown opcode+funct3: {:07b} {:03b}",
                                self.pc,
                                inst,
                                opcode,
                                funct3
                            );
                        }
                    }
                }
                0b111_0011 => {
                    if inst == 0b111_0011 {
                        self.ecall();
                    } else {
                        log::error!("{:x} {:08x}: unimplemented environment call", self.pc, inst);
                    }
                }
                _ => {
                    log::error!("{:x} {:08x}: unknown opcode: {:07b}", self.pc, inst, opcode);
                }
            }

            self.pc += 4;
        }
    }

    fn memdump(&self, prefix: &str) {
        let mut i: usize = 0;
        for value in self.mem.iter() {
            if *value != 0 {
                log::debug!("{}{:x}: {:02x}", prefix, i, value);
            }
            i += 1;
        }
    }

    /* instructions */
    fn auipc(&mut self, rd: usize, imm20: u32) {
        log::debug!(
            "{:x} {:08x}: auipc {}, 0x{:x}",
            self.pc,
            self.curr(),
            Reg::try_from(rd).unwrap(),
            imm20
        );
        self.reg[rd] = self.pc as u32 + (imm20 << 12);
    }

    fn addi(&mut self, rd: usize, rs1: usize, imm12: u32) {
        log::debug!(
            "{:x} {:08x}: addi {}, {}, {}",
            self.pc,
            self.curr(),
            Reg::try_from(rd).unwrap(),
            Reg::try_from(rs1).unwrap(),
            sext!(imm12, 12)
        );
        self.reg[rd] = self.reg[rs1] + sext!(imm12, 12);
    }

    fn ecall(&mut self) {
        log::debug!("{:x} {:08x}: ecall", self.pc, self.curr());
        let a7: usize = Reg::A7.into();
        let syscall = self.reg[a7];
        match syscall {
            64 => {
                // RISC-V write
                let a0: usize = Reg::A0.into();
                let a1: usize = Reg::A1.into();
                let a2: usize = Reg::A2.into();

                log::debug!(
                    "write syscall: fp: {} addr: {:x} len: {}",
                    self.reg[a0],
                    self.reg[a1],
                    self.reg[a2]
                );

                let mut fp = unsafe { File::from_raw_fd(self.reg[a0] as i32) };
                let addr = self.reg[a1] as usize;
                let len = self.reg[a2] as usize;
                if let Ok(len) = fp.write(&self.mem[addr..addr + len]) {
                    log::debug!("wrote {} bytes", len);
                    self.reg[a0] = len as u32;
                } else {
                    log::debug!("write error");
                    self.reg[a0] = 0xffffffff;
                }
            }
            93 => {
                // RISC-V exit
                let a0: usize = Reg::A0.into();
                log::debug!("exit syscall: rc: {}", self.reg[a0]);
                process::exit(self.reg[a0] as i32);
            }
            _ => {
                log::error!("unknown/unimplemented syscall: {}", syscall);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut vm: VirtualMachine = VirtualMachine::load_from("hello")?;
    log::debug!("start address: 0x{:x}", vm.pc);

    if let Ok(value) = env::var("RUST_LOG") {
        if value.to_lowercase() == "debug".to_string() {
            vm.memdump("memdump: ");
        }
    }

    vm.run();

    Ok(())
}
