use crate::{reg::Reg, Emulator, EmulatorError};

include!(concat!(env!("OUT_DIR"), "/enum.rs")); // enum Inst
include!(concat!(env!("OUT_DIR"), "/exec.rs")); // Inst::execute()
include!(concat!(env!("OUT_DIR"), "/decode.rs")); // impl TryFrom<u32> for Inst
include!(concat!(env!("OUT_DIR"), "/encode.rs")); // impl From<Inst> for u32

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

    /// Extracts funct3 bits from an instruction (inst[14:12]).
    fn funct3(inst: u32) -> u32 {
        (inst >> 12) & 0b111
    }

    /// Extracts funct7 bits from an instruction (inst[31:25]).
    fn funct7(inst: u32) -> u32 {
        (inst >> 25) & 0b111_1111
    }

    /// Extracts immediate value for a B-Type instruction.
    fn imm_b(inst: u32) -> i32 {
        let base = ((((inst) >> 31) & 0x1) << 12)
            | ((((inst) >> 7) & 0b1) << 11)
            | ((((inst) >> 25) & 0b111111) << 5)
            | ((((inst) >> 8) & 0b1111) << 1);
        if inst & (1 << 31) == (1 << 31) {
            (base | 0xfffff000) as i32
        } else {
            base as i32
        }
    }

    /// Extracts immediate value for an I-Type instruction.
    fn imm_i(inst: u32) -> i32 {
        let base = inst >> 20;
        if inst & (1 << 31) == (1 << 31) {
            (base | 0xfffff000) as i32
        } else {
            base as i32
        }
    }

    /// Extracts immediate value for a J-Type instruction.
    fn imm_j(inst: u32) -> i32 {
        let base = ((((inst) >> 31) & 0b1) << 20)
            | ((((inst) >> 12) & 0b11111111) << 12)
            | ((((inst) >> 20) & 0b1) << 11)
            | ((((inst) >> 21) & 0b1111111111) << 1);
        if inst & (1 << 31) == (1 << 31) {
            (base | 0xfff00000) as i32
        } else {
            base as i32
        }
    }

    /// Extracts immediate value for an S-Type instruction.
    fn imm_s(inst: u32) -> i32 {
        let base = (((inst) >> 25) << 5) | (((inst) >> 7) & 0b11111);
        if inst & (1 << 31) == (1 << 31) {
            (base | 0xfffff000) as i32
        } else {
            base as i32
        }
    }

    /// Extracts immediate value for a U-Type instruction.
    fn imm_u(inst: u32) -> i32 {
        let base = inst >> 12;
        if inst & (1 << 31) == (1 << 31) {
            (base | 0xfff00000) as i32
        } else {
            base as i32
        }
    }

    /// Encodes a B-Type Inst as a u32.
    ///
    /// ```rust
    /// use rvem::Inst;
    ///
    /// let word = 0x02238a63; // beq x7, x2, 52
    /// let decode = Inst::try_from(word).unwrap();
    /// let encode = u32::from(decode);
    /// assert_eq!(word, encode);
    /// ```
    fn b_type(opcode: u32, funct3: u32, rs1: Reg, rs2: Reg, imm: i32) -> u32 {
        let bits = imm as u32;
        let bits = ((bits & (1 << 12)) << 19) // inst[31]
            | ((bits & (0b0011_1111 << 5)) << 20)  // inst[30:25]
            | ((bits & (0b1111 << 1)) << 7) // inst[11:8]
            | ((bits & (1 << 11)) >> 4); // inst[7]
        bits | (u32::from(rs2) << 20) | (u32::from(rs1) << 15) | (funct3 << 12) | opcode
    }

    /// Encodes a I-Type Inst as a u32.
    ///
    /// ```rust
    /// use rvem::Inst;
    ///
    /// let word = 0x02058593; // addi x11, x11, 32
    /// let inst = Inst::try_from(word).unwrap();
    /// let decode = u32::from(inst);
    /// assert_eq!(word, decode);
    /// ```
    fn i_type(opcode: u32, funct3: u32, rd: Reg, rs1: Reg, imm: i32) -> u32 {
        let bits = (imm << 20) as u32;
        bits | (u32::from(rs1) << 15) | (funct3 << 12) | (u32::from(rd) << 7) | opcode
    }

    /// Encodes a I-Type shamt Inst as a u32.
    ///
    /// ```rust
    /// use rvem::Inst;
    ///
    /// let word = 0x00361613; // slli x12, x12, 3
    /// let decode = Inst::try_from(word).unwrap();
    /// let encode = u32::from(decode);
    /// assert_eq!(word, encode);
    /// ```
    fn i_type_shamt(opcode: u32, funct3: u32, funct7: u32, rd: Reg, rs1: Reg, shamt: u32) -> u32 {
        (funct7 << 25)
            | (shamt << 20)
            | (u32::from(rs1) << 15)
            | (funct3 << 12)
            | (u32::from(rd) << 7)
            | opcode
    }

    /// Encodes a J-Type Inst as a u32.
    ///
    /// ```rust
    /// use rvem::Inst;
    ///
    /// let word = 0xfedff06f; // jal x0, -20
    /// let decode = Inst::try_from(word).unwrap();
    /// let encode = u32::from(decode);
    /// assert_eq!(word, encode);
    /// ```
    fn j_type(opcode: u32, rd: Reg, imm: i32) -> u32 {
        let bits = imm as u32;
        let bits = ((bits & (1<<20)) << 11) // inst[31]
            | ((bits & (0b11_1111_1111 << 1)) << 20) // inst[30:21]
            | ((bits & (1 << 11)) << 9) // inst[20]
            | (bits & (0b1111_1111 << 12)) // inst[19:12]
        ;
        bits | (u32::from(rd) << 7) | opcode
    }

    /// Encodes a R-Type shamt Inst as a u32.
    ///
    /// ```rust
    /// use rvem::Inst;
    ///
    /// let word = 0x00e787b3; // add x15, x15, x14
    /// let decode = Inst::try_from(word).unwrap();
    /// let encode = u32::from(decode);
    /// assert_eq!(word, encode);
    /// ```
    fn r_type(opcode: u32, funct3: u32, funct7: u32, rd: Reg, rs1: Reg, rs2: Reg) -> u32 {
        (funct7 << 25)
            | (u32::from(rs2) << 20)
            | (u32::from(rs1) << 15)
            | (funct3 << 12)
            | (u32::from(rd) << 7)
            | opcode
    }

    /// Encodes a S-Type Inst as a u32.
    ///
    /// ```rust
    /// use rvem::Inst;
    ///
    /// let word = 0xd6a1a023; // sw x10, -672(x3)
    /// let decode = Inst::try_from(word).unwrap();
    /// let encode = u32::from(decode);
    /// assert_eq!(word, encode);
    /// ```
    fn s_type(opcode: u32, funct3: u32, rs1: Reg, rs2: Reg, imm: i32) -> u32 {
        let bits = imm as u32;
        let bits = ((bits & (0b111_1111 << 5)) << 20) // inst[31:25]
         | ((bits & (0b1_1111)) << 7); // inst[11:7]
        bits | (u32::from(rs2) << 20) | (u32::from(rs1) << 15) | (funct3 << 12) | opcode
    }

    /// Encodes a U-Type Inst as a u32.
    ///
    /// ```rust
    /// use rvem::Inst;
    ///
    /// let word = 0x808088b7; // lui x17, -522232
    /// let decode = Inst::try_from(word).unwrap();
    /// let encode = u32::from(decode);
    /// assert_eq!(word, encode);
    /// ```
    fn u_type(opcode: u32, rd: Reg, imm: i32) -> u32 {
        let bits = (imm << 12) as u32;
        bits | (u32::from(rd) << 7) | opcode
    }
}

impl std::fmt::Display for Inst {
    // NB this is a bit of a hack, but we're going to repurpose precision
    // to pass in the instruction's address in memory
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            /* B-Type */
            Inst::BEQ { rs1, rs2, imm } => {
                let addr = if let Some(pc) = f.precision() {
                    format!("{:x}", pc as i32 + *imm)
                } else {
                    format!("PC+{}", *imm)
                };
                write!(f, "beq {}, {}, {addr}", rs1, rs2)
            }
            Inst::BNE { rs1, rs2, imm } => {
                let addr = if let Some(pc) = f.precision() {
                    format!("{:x}", pc as i32 + *imm)
                } else {
                    format!("PC+{}", *imm)
                };
                write!(f, "bne {}, {}, {addr}", rs1, rs2)
            }
            Inst::BLT { rs1, rs2, imm } => {
                let addr = if let Some(pc) = f.precision() {
                    format!("{:x}", pc as i32 + *imm)
                } else {
                    format!("PC+{}", *imm)
                };
                write!(f, "blt {}, {}, {addr}", rs1, rs2)
            }
            Inst::BGE { rs1, rs2, imm } => {
                let addr = if let Some(pc) = f.precision() {
                    format!("{:x}", pc as i32 + *imm)
                } else {
                    format!("PC+{}", *imm)
                };
                write!(f, "bge {}, {}, {addr}", rs1, rs2)
            }
            Inst::BLTU { rs1, rs2, imm } => {
                let addr = if let Some(pc) = f.precision() {
                    format!("{:x}", pc as i32 + *imm)
                } else {
                    format!("PC+{}", *imm)
                };
                write!(f, "bltu {}, {}, {addr}", rs1, rs2)
            }
            Inst::BGEU { rs1, rs2, imm } => {
                let addr = if let Some(pc) = f.precision() {
                    format!("{:x}", pc as i32 + *imm)
                } else {
                    format!("PC+{}", *imm)
                };
                write!(f, "bgeu {}, {}, {addr}", rs1, rs2)
            }

            /* I-Type */
            // integer operations
            Inst::ADDI { rd, rs1, imm } => {
                if *rs1 == Reg::zero {
                    write!(f, "li {}, {}", rd, *imm)
                } else {
                    write!(f, "addi {}, {}, {}", rd, rs1, imm)
                }
            }
            Inst::ANDI { rd, rs1, imm } => {
                write!(f, "andi {}, {}, {}", rd, rs1, *imm)
            }
            Inst::ORI { rd, rs1, imm } => {
                write!(f, "ori {}, {}, {}", rd, rs1, *imm)
            }
            Inst::SLTI { rd, rs1, imm } => {
                write!(f, "slti {}, {}, {}", rd, rs1, *imm)
            }
            Inst::SLTIU { rd, rs1, imm } => {
                write!(f, "sltiu {}, {}, {}", rd, rs1, *imm)
            }
            Inst::XORI { rd, rs1, imm } => {
                write!(f, "xori {}, {}, {}", rd, rs1, *imm)
            }

            // loads
            Inst::LB { rd, rs1, imm } => {
                write!(f, "lb {}, {}({})", rd, *imm, rs1)
            }
            Inst::LH { rd, rs1, imm } => {
                write!(f, "lh {}, {}({})", rd, *imm, rs1)
            }
            Inst::LW { rd, rs1, imm } => {
                write!(f, "lw {}, {}({})", rd, *imm, rs1)
            }
            Inst::LBU { rd, rs1, imm } => {
                write!(f, "lbu {}, {}({})", rd, *imm, rs1)
            }
            Inst::LHU { rd, rs1, imm } => {
                write!(f, "lhu {}, {}({})", rd, *imm, rs1)
            }

            // shifts
            Inst::SLLI { rd, rs1, shamt } => {
                write!(f, "slli {rd}, {rs1}, {shamt}")
            }
            Inst::SRLI { rd, rs1, shamt } => {
                write!(f, "srli {rd}, {rs1}, {shamt}")
            }
            Inst::SRAI { rd, rs1, shamt } => {
                write!(f, "slai {rd}, {rs1}, {shamt}")
            }

            // jumps
            Inst::JALR { rd, rs1, imm } => {
                write!(f, "jalr {}, {}({})", rd, *imm, rs1)
            }

            /* J-Type */
            Inst::JAL { rd, imm } => {
                if let Some(pc) = f.precision() {
                    write!(f, "j {:x}", (pc as i32 + *imm))
                } else {
                    write!(f, "jal {}, {:x}", rd, *imm)
                }
            }

            /* R-Type */
            // integer operations
            Inst::ADD { rd, rs1, rs2 } => {
                write!(f, "add {}, {}, {}", rd, rs1, rs2)
            }
            Inst::AND { rd, rs1, rs2 } => {
                write!(f, "and {}, {}, {}", rd, rs1, rs2)
            }
            Inst::OR { rd, rs1, rs2 } => {
                write!(f, "or {}, {}, {}", rd, rs1, rs2)
            }
            Inst::SLL { rd, rs1, rs2 } => {
                write!(f, "sll {}, {}, {}", rd, rs1, rs2)
            }
            Inst::SLT { rd, rs1, rs2 } => {
                write!(f, "slt {}, {}, {}", rd, rs1, rs2)
            }
            Inst::SLTU { rd, rs1, rs2 } => {
                write!(f, "sltu {}, {}, {}", rd, rs1, rs2)
            }
            Inst::SRL { rd, rs1, rs2 } => {
                write!(f, "srl {}, {}, {}", rd, rs1, rs2)
            }
            Inst::SRA { rd, rs1, rs2 } => {
                write!(f, "sra {}, {}, {}", rd, rs1, rs2)
            }
            Inst::SUB { rd, rs1, rs2 } => {
                write!(f, "sub {}, {}, {}", rd, rs1, rs2)
            }
            Inst::XOR { rd, rs1, rs2 } => {
                write!(f, "xor {}, {}, {}", rd, rs1, rs2)
            }

            // multiplication/division extension
            #[cfg(feature = "rv32m")]
            Inst::MUL { rd, rs1, rs2 } => {
                write!(f, "mul {}, {}, {}", rd, rs1, rs2)
            }
            #[cfg(feature = "rv32m")]
            Inst::MULH { rd, rs1, rs2 } => {
                write!(f, "mulh {}, {}, {}", rd, rs1, rs2)
            }
            #[cfg(feature = "rv32m")]
            Inst::MULHSU { rd, rs1, rs2 } => {
                write!(f, "mulhsu {}, {}, {}", rd, rs1, rs2)
            }
            #[cfg(feature = "rv32m")]
            Inst::MULHU { rd, rs1, rs2 } => {
                write!(f, "mulhu {}, {}, {}", rd, rs1, rs2)
            }
            #[cfg(feature = "rv32m")]
            Inst::DIV { rd, rs1, rs2 } => {
                write!(f, "div {}, {}, {}", rd, rs1, rs2)
            }
            #[cfg(feature = "rv32m")]
            Inst::DIVU { rd, rs1, rs2 } => {
                write!(f, "divu {}, {}, {}", rd, rs1, rs2)
            }
            #[cfg(feature = "rv32m")]
            Inst::REM { rd, rs1, rs2 } => {
                write!(f, "rem {}, {}, {}", rd, rs1, rs2)
            }
            #[cfg(feature = "rv32m")]
            Inst::REMU { rd, rs1, rs2 } => {
                write!(f, "remu {}, {}, {}", rd, rs1, rs2)
            }

            /* S-Type */
            Inst::SB { rs1, rs2, imm } => {
                write!(f, "sb {}, {}({})", rs2, *imm, rs1)
            }

            Inst::SH { rs1, rs2, imm } => {
                write!(f, "sh {}, {}({})", rs2, *imm, rs1)
            }

            Inst::SW { rs1, rs2, imm } => {
                write!(f, "sw {}, {}({})", rs2, *imm, rs1)
            }

            /* U-Type */
            Inst::AUIPC { rd, imm } => {
                write!(f, "auipc {}, 0x{:x}", rd, *imm)
            }
            Inst::LUI { rd, imm } => {
                write!(f, "lui {}, 0x{:x}", rd, *imm)
            }

            /* syscalls */
            Inst::ECALL => write!(f, "ecall"),

            _ => {
                // TODO implement Diplay for all the rest of the instruction types
                write!(f, "{:?}", self)
            }
        }
    }
}
