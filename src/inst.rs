use crate::{reg::Reg, sext, Emulator, EmulatorError};

include!(concat!(env!("OUT_DIR"), "/enum.rs")); // enum Inst
include!(concat!(env!("OUT_DIR"), "/exec.rs")); // Inst::execute()
include!(concat!(env!("OUT_DIR"), "/decode.rs")); // impl TryFrom<u32> for Inst

impl Inst {
    /// Extracts the opcode from an instruction (inst[6:0]).
    fn opcode(inst: u32) -> u32 {
        inst & 0b0111_1111
    }

    /// Extracts the destination register bits from an instruction (inst[11:7]).
    fn rd(inst: u32) -> Reg {
        Reg::from((inst >> 7) & 0b1_1111)
    }

    /// Extracts the first argument register bits from an instruction (inst[19:15]).
    fn rs1(inst: u32) -> Reg {
        Reg::from((inst >> 15) & 0b1_1111)
    }

    /// Extracts the second argument register bits from an instruction (inst[24:20]).
    fn rs2(inst: u32) -> Reg {
        Reg::from((inst >> 20) & 0b1_1111)
    }

    /// Extracts shift amount bits from an instruction (inst[24:20]).
    fn shamt(inst: u32) -> u32 {
        (inst >> 20) & 0b1_1111
    }

    /// Extracts funct3 buts from an instruction (inst[14:12]).
    fn funct3(inst: u32) -> u32 {
        (inst >> 12) & 0b111
    }

    /// Extracts funct7 buts from an instruction (inst[31:25]).
    fn funct7(inst: u32) -> u32 {
        inst >> 25
    }

    /// Extracts immediate value for a B-Type instruction.
    fn imm_b(inst: u32) -> u32 {
        ((((inst) >> 31) & 0x1) << 12)
            | ((((inst) >> 7) & 0b1) << 11)
            | ((((inst) >> 25) & 0b111111) << 5)
            | ((((inst) >> 8) & 0b1111) << 1)
    }

    /// Extracts immediate value for a J-Type instruction.
    fn imm_j(inst: u32) -> u32 {
        ((((inst) >> 31) & 0b1) << 20)
            | ((((inst) >> 12) & 0b11111111) << 12)
            | ((((inst) >> 20) & 0b1) << 11)
            | ((((inst) >> 21) & 0b1111111111) << 1)
    }

    /// Extracts immediate value for an S-Type instruction.
    fn imm_s(inst: u32) -> u32 {
        (((inst) >> 25) << 7) | (((inst) >> 7) & 0b11111)
    }
}

impl std::fmt::Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Inst::ADDI { rd, rs1, imm } => {
                if *rs1 == Reg::zero {
                    write!(f, "li {}, {}", rd, sext(*imm, 12) as i32)
                } else {
                    write!(f, "addi {}, {}, {}", rd, rs1, sext(*imm, 12) as i32)?;
                    Ok(())
                }
            }

            Inst::ORI { rd, rs1, imm } => {
                write!(f, "or {}, {}, {}", rd, rs1, sext(*imm, 12) as i32)
            }

            Inst::AUIPC { rd, imm } => {
                write!(f, "auipc {}, 0x{:x}", rd, *imm)
            }

            Inst::LW { rd, rs1, imm } => {
                write!(f, "lw {}, {}({})", rd, sext(*imm, 12) as i32, rs1)
            }

            Inst::BEQ { rs1, rs2, imm } => {
                let addr = if let Some(pc) = f.precision() {
                    format!("{:x}", pc as i32 + sext(*imm, 12) as i32)
                } else {
                    format!("PC+{}", sext(*imm, 12) as i32)
                };
                write!(f, "beq {}, {}, {addr}", rs1, rs2)
            }

            Inst::ADD { rd, rs1, rs2 } => {
                write!(f, "add {}, {}, {}", rd, rs1, rs2)
            }

            Inst::JAL { rd, imm } => {
                if let Some(pc) = f.precision() {
                    write!(f, "j {:x}", (pc as i32 + sext(*imm, 20) as i32))
                } else {
                    write!(f, "jal {}, {:x}", rd, *imm)
                }
            }

            Inst::ECALL => write!(f, "ecall"),
            _ => {
                // TODO implement Diplay for all the rest of the instruction types
                write!(f, "{:?}", self)
            }
        }
    }
}
