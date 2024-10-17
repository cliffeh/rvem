use goblin::elf::Elf;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use strum_macros::{Display, EnumString};

fn load_elf_data(path: &str, out: &mut [u8]) -> Result<usize, Box<dyn std::error::Error>> {
    let mut exec_start = 0usize;

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
            out[section.vm_range()].copy_from_slice(&buf[section.file_range().unwrap()]);
            if section.is_executable() {
                exec_start = section.sh_addr as usize;
            }
        }
    }

    Ok(exec_start)

    // TODO use this to get the value of gp
    // println!("\nSymbols:");
    // for sym in elf.syms.iter() {
    //     let sym_name = elf.strtab.get(sym.st_name);
    //     match sym_name {
    //         Some(Ok(name)) => println!("Symbol: {} - Address: 0x{:x}", name, sym.st_value),
    //         _ => println!("Unknown symbol name"),
    //     }
    // }
}

#[repr(u32)]
#[derive(Debug, Display, EnumString, TryFromPrimitive)]
#[strum(serialize_all = "lowercase")]
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

macro_rules! opcode {
    ($value:expr) => {
        ($value) & 0b111_1111
    };
}

macro_rules! rd {
    ($value:expr) => {
        ($value >> 7) & 0b1_1111
    };
}

macro_rules! rs1 {
    ($value:expr) => {
        ($value >> 15) & 0b1_1111
    };
}

// macro_rules! rs2 {
//     ($value:expr) => {
//         ($value >> 7) & 0b1_1111
//     };
// }

macro_rules! funct3 {
    ($value:expr) => {
        ($value >> 12) & 0b111
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

macro_rules! inst_error {
    ($pc:expr, $inst:expr, $($arg:tt)+) => (log::error!("{:x}: {:08x} {}", $pc, $inst, format!($($arg)+)))
}

macro_rules! inst_debug {
    ($pc:expr, $inst:expr, $($arg:tt)+) => (log::debug!("{:x}: {:08x} {}", $pc, $inst, format!($($arg)+)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut buf: [u8; 1 << 20] = [0u8; 1 << 20];
    let mut pc = load_elf_data("hello", &mut buf)?;
    log::debug!("start address: 0x{pc:x}");

    while buf[pc] != 0 {
        let inst = u32::from_le_bytes(
            buf[pc..pc + 4]
                .try_into()
                .expect("incorrect byte slice length"),
        );

        match opcode!(inst) {
            0b0010111 => {
                // AUIPC
                inst_debug!(
                    pc,
                    inst,
                    "auipc {}, 0x{:x}",
                    Reg::try_from(rd!(inst))?,
                    ((inst & 0xfffff000) >> 12)
                );
            }
            0b0010011 => {
                // I-Type
                match funct3!(inst) {
                    0b000 => {
                        // ADDI
                        inst_debug!(
                            pc,
                            inst,
                            "addi {}, {}, {}",
                            Reg::try_from(rd!(inst))?,
                            Reg::try_from(rs1!(inst))?,
                            sext!(inst >> 20, 12) as i32
                        );
                    }
                    _ => {
                        inst_error!(
                            pc,
                            inst,
                            "unknown opcode+funct3: {:07b} {:03b}",
                            opcode!(inst),
                            funct3!(inst)
                        );
                    }
                }
            }
            _ => {
                log::error!("{pc:x}: {inst:08x} unknown opcode: {:07b}", opcode!(inst));
            }
        }
        pc += 4;
    }

    Ok(())
}
