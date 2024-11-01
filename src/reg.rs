use strum::{Display, EnumIter};

/// Enumeration of all available registers.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Display, EnumIter, PartialEq)]
#[repr(u32)]
pub enum Reg {
    /// x0 - hardwired to 0, ignores writes
    zero,
    /// x1 - return address for jumps
    ra,
    /// x2 - stack pointer
    sp,
    /// x3 = global pointer
    gp,
    /// x4 - thread pointer
    tp,
    /// x5 - temporary register 0
    t0,
    /// x6 - temporary register 1
    t1,
    /// x7 - temporary register 2
    t2,
    /// x8 - saved register 0 or frame pointer
    s0,
    /// x9 - saved register 1
    s1,
    /// x10 - return value or function argument 0
    a0,
    /// x11 - return value or function argument 1
    a1,
    /// x12 - function argument 2
    a2,
    /// x13 - function argument 3
    a3,
    /// x14 - function argument 4
    a4,
    /// x15 - function argument 5
    a5,
    /// x16 - function argument 6
    a6,
    /// x17 - function argument 7
    a7,
    /// x18 - saved register 2
    s2,
    /// x19 - saved register 3
    s3,
    /// x20 - saved register 4
    s4,
    /// x21 - saved register 5
    s5,
    /// x22 - saved register 6
    s6,
    /// x23 - saved register 7
    s7,
    /// x24 - saved register 8
    s8,
    /// x25 - saved register 9
    s9,
    /// x26 - saved register 10
    s10,
    /// x27 - saved register 11
    s11,
    /// x28 - temporary register 3
    t3,
    /// x29 - temporary register 4
    t4,
    /// x30 - temporary register 5
    t5,
    /// x31 - temporary register 6
    t6,
}

#[allow(non_upper_case_globals)]
impl Reg {
    pub const x0: Reg = Reg::zero;
    pub const x1: Reg = Reg::ra;
    pub const x2: Reg = Reg::sp;
    pub const x3: Reg = Reg::gp;
    pub const x4: Reg = Reg::tp;
    pub const x5: Reg = Reg::t0;
    pub const x6: Reg = Reg::t1;
    pub const x7: Reg = Reg::t2;
    pub const x8: Reg = Reg::s0;
    pub const x9: Reg = Reg::s1;
    pub const x10: Reg = Reg::a0;
    pub const x11: Reg = Reg::a1;
    pub const x12: Reg = Reg::a2;
    pub const x13: Reg = Reg::a3;
    pub const x14: Reg = Reg::a4;
    pub const x15: Reg = Reg::a5;
    pub const x16: Reg = Reg::a6;
    pub const x17: Reg = Reg::a7;
    pub const x18: Reg = Reg::s2;
    pub const x19: Reg = Reg::s3;
    pub const x20: Reg = Reg::s4;
    pub const x21: Reg = Reg::s5;
    pub const x22: Reg = Reg::s6;
    pub const x23: Reg = Reg::s7;
    pub const x24: Reg = Reg::s8;
    pub const x25: Reg = Reg::s9;
    pub const x26: Reg = Reg::s10;
    pub const x27: Reg = Reg::s11;
    pub const x28: Reg = Reg::t3;
    pub const x29: Reg = Reg::t4;
    pub const x30: Reg = Reg::t5;
    pub const x31: Reg = Reg::t6;
    pub const fp: Reg = Reg::s0;
}

impl From<u32> for Reg {
    fn from(value: u32) -> Self {
        match value {
            0 => Reg::x0,
            1 => Reg::x1,
            2 => Reg::x2,
            3 => Reg::x3,
            4 => Reg::x4,
            5 => Reg::x5,
            6 => Reg::x6,
            7 => Reg::x7,
            8 => Reg::x8,
            9 => Reg::x9,
            10 => Reg::x10,
            11 => Reg::x11,
            12 => Reg::x12,
            13 => Reg::x13,
            14 => Reg::x14,
            15 => Reg::x15,
            16 => Reg::x16,
            17 => Reg::x17,
            18 => Reg::x18,
            19 => Reg::x19,
            20 => Reg::x20,
            21 => Reg::x21,
            22 => Reg::x22,
            23 => Reg::x23,
            24 => Reg::x24,
            25 => Reg::x25,
            26 => Reg::x26,
            27 => Reg::x27,
            28 => Reg::x28,
            29 => Reg::x29,
            30 => Reg::x30,
            31 => Reg::x31,
            _ => unimplemented!("unimplemented register value: {}", value),
        }
    }
}

impl From<Reg> for u32 {
    fn from(value: Reg) -> Self {
        match value {
            Reg::x0 => 0u32,
            Reg::x1 => 1u32,
            Reg::x2 => 2u32,
            Reg::x3 => 3u32,
            Reg::x4 => 4u32,
            Reg::x5 => 5u32,
            Reg::x6 => 6u32,
            Reg::x7 => 7u32,
            Reg::x8 => 8u32,
            Reg::x9 => 9u32,
            Reg::x10 => 10u32,
            Reg::x11 => 11u32,
            Reg::x12 => 12u32,
            Reg::x13 => 13u32,
            Reg::x14 => 14u32,
            Reg::x15 => 15u32,
            Reg::x16 => 16u32,
            Reg::x17 => 17u32,
            Reg::x18 => 18u32,
            Reg::x19 => 19u32,
            Reg::x20 => 20u32,
            Reg::x21 => 21u32,
            Reg::x22 => 22u32,
            Reg::x23 => 23u32,
            Reg::x24 => 24u32,
            Reg::x25 => 25u32,
            Reg::x26 => 26u32,
            Reg::x27 => 27u32,
            Reg::x28 => 28u32,
            Reg::x29 => 29u32,
            Reg::x30 => 30u32,
            Reg::x31 => 31u32,
        }
    }
}
