// build.rs

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::read_to_string;
use std::path::Path;
use syn::Ident;

fn sanitize_name(name: &str) -> String {
    name.replace(".", "_")
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let decode_path = Path::new(&out_dir).join("decode.rs");
    let enum_path = Path::new(&out_dir).join("enum.rs");
    let exec_path = Path::new(&out_dir).join("exec.rs");

    let mut variants: Vec<TokenStream> = vec![];

    let mut btype: HashMap<u32, HashMap<u32, Ident>> = HashMap::new();
    let mut itype: HashMap<u32, HashMap<u32, Ident>> = HashMap::new();
    let mut shamt: HashMap<u32, HashMap<u32, HashMap<u32, Ident>>> = HashMap::new();
    let mut rtype: HashMap<u32, HashMap<u32, HashMap<u32, Ident>>> = HashMap::new();
    let mut stype: HashMap<u32, HashMap<u32, Ident>> = HashMap::new();

    let mut opcode_matches: Vec<TokenStream> = vec![];
    let mut exec_matches: Vec<TokenStream> = vec![];

    let mut tables: Vec<&str> = vec!["src/rv32i.tab"];

    #[cfg(feature = "rv32m")]
    tables.push("src/rv32m.tab");

    for filename in tables {
        for line in read_to_string(filename).unwrap().lines() {
            let pieces: Vec<&str> = line.split(&[' ', '\t', '\r', '\n']).collect();

            let opname = format_ident!("{}", sanitize_name(pieces[pieces.len() - 1]));
            let lcname = sanitize_name(pieces[pieces.len() - 1]).to_lowercase();
            let funname = format_ident!("{}", lcname);
            let opcode = u32::from_str_radix(pieces[pieces.len() - 2], 2).unwrap();

            // TODO this will work for now, but could use refinement/refactoring
            match pieces[0] {
                // B-Type: imm[12|10:5] rs2 rs1 000 imm[4:1|11] 1100011 BEQ
                "imm[12|10:5]" => {
                    variants.push(quote! {#opname{rs1: Reg, rs2: Reg, imm: u32}});
                    exec_matches.push(
                        quote! {Inst::#opname{rs1, rs2, imm} => em.#funname(*rs1, *rs2, *imm)},
                    );

                    let funct3 = u32::from_str_radix(pieces[3], 2).unwrap();
                    let funct3s = btype.entry(opcode).or_default();
                    funct3s.insert(funct3, opname);
                }
                // I-Type: imm[11:0] rs1 000 rd 0010011 ADDI
                "imm[11:0]" => {
                    variants.push(quote! {#opname{rd: Reg, rs1: Reg, imm: u32}});
                    exec_matches
                        .push(quote! {Inst::#opname{rd, rs1, imm} => em.#funname(*rd, *rs1, *imm)});

                    let funct3 = u32::from_str_radix(pieces[2], 2).unwrap();
                    let funct3s = itype.entry(opcode).or_default();
                    funct3s.insert(funct3, opname);
                }
                // J-Type: imm[20|10:1|11|19:12] rd 1101111 JAL
                "imm[20|10:1|11|19:12]" => {
                    variants.push(quote! {#opname{rd: Reg,  imm: u32}});
                    exec_matches.push(quote! {Inst::#opname{rd, imm} => em.#funname(*rd, *imm)});

                    opcode_matches.push(quote! {
                        #opcode => Ok(Inst::#opname{rd: Inst::rd(inst), imm: Inst::imm_j(inst)})
                    });
                }
                // R-Type: 0000000 rs2 rs1 000 rd 0110011 ADD
                "0000000" | "0000001" | "0100000" => {
                    let funct3 = u32::from_str_radix(pieces[3], 2).unwrap();
                    let funct7 = u32::from_str_radix(pieces[0], 2).unwrap();
                    // shamt (special case): 0000000 shamt rs1 001 rd 0010011 SLLI
                    if pieces[1] == "shamt" {
                        variants.push(quote! {#opname{rd: Reg, rs1: Reg, shamt: u32}});
                        exec_matches.push(quote!{Inst::#opname{rd, rs1, shamt} => em.#funname(*rd, *rs1, *shamt)});

                        let funct3s = shamt.entry(opcode).or_default();
                        let funct7s = funct3s.entry(funct3).or_default();
                        funct7s.insert(funct7, opname);
                    } else {
                        // 0000000 rs2 rs1 000 rd 0110011 ADD
                        variants.push(quote! {#opname{rd: Reg, rs1: Reg, rs2: Reg}});
                        exec_matches.push(
                            quote! {Inst::#opname{rd, rs1, rs2} => em.#funname(*rd, *rs1, *rs2)},
                        );

                        let funct3s = rtype.entry(opcode).or_default();
                        let funct7s = funct3s.entry(funct3).or_default();
                        funct7s.insert(funct7, opname);
                    }
                }
                // S-Type: imm[11:5] rs2 rs1 000 imm[4:0] 0100011 SB
                "imm[11:5]" => {
                    variants.push(quote! {#opname{rs1: Reg, rs2: Reg, imm: u32}});
                    exec_matches.push(
                        quote! {Inst::#opname{rs1, rs2, imm} => em.#funname(*rs1, *rs2, *imm)},
                    );

                    let funct3 = u32::from_str_radix(pieces[3], 2).unwrap();
                    let funct3s = stype.entry(opcode).or_default();
                    funct3s.insert(funct3, opname);
                }
                // U-Type: imm[31:12] rd 0110111 LUI
                "imm[31:12]" => {
                    variants.push(quote! {#opname{rd: Reg, imm: u32}});
                    exec_matches.push(quote! {Inst::#opname{rd, imm} => em.#funname(*rd, *imm)});

                    opcode_matches.push(quote! {
                        #opcode => Ok(Inst::#opname{rd: Inst::rd(inst), imm: inst >> 12})
                    });
                }
                _ => {
                    if opname == "ECALL" {
                        variants.push(quote! {#opname});
                        opcode_matches.push(quote! {
                            #opcode => Ok(Inst::#opname)
                        });
                        exec_matches.push(quote! {Inst::ECALL => em.ecall()});
                    } else {
                        variants.push(quote! {
                            // keep the compiler from griping about unused variants
                            #[allow(dead_code)]
                            #opname
                        });
                        exec_matches.push(quote! {Inst::#opname => em.nop()});
                    }
                }
            }
        }
    }

    // B-Type
    for (opcode, funct3s) in btype {
        let mut funct3_matches: Vec<TokenStream> = vec![];
        for (funct3, opname) in funct3s {
            funct3_matches.push(quote!{
                #funct3 => Ok(Inst::#opname{rs1: Inst::rs1(inst), rs2: Inst::rs2(inst), imm: Inst::imm_b(inst)})
            });
        }
        opcode_matches.push(quote! {
            #opcode => {
                let funct3 = Inst::funct3(inst);
                match funct3 {
                    #(#funct3_matches,)*
                    _ => { Err(EmulatorError::InstructionDecode(format!("unknown/unimplemented opcode+funct3 {:07b} {:03b}", opcode, funct3))) }
                }
            }
        })
    }

    // I-Type
    for (opcode, funct3s) in itype {
        let mut funct3_matches: Vec<TokenStream> = vec![];
        for (funct3, opname) in funct3s {
            funct3_matches.push(quote! {
                #funct3 => Ok(Inst::#opname{rd: Inst::rd(inst), rs1: Inst::rs1(inst), imm: inst >> 20})
            });
        }
        // special case for I-Types w/shamt instead of rs2
        if let Some(funct3s) = shamt.get(&opcode) {
            for (funct3, funct7s) in funct3s {
                let mut funct7_matches: Vec<TokenStream> = vec![];
                for (funct7, opname) in funct7s {
                    funct7_matches.push(quote!{
                        #funct7 => Ok(Inst::#opname{rd: Inst::rd(inst), rs1: Inst::rs1(inst), shamt: Inst::shamt(inst)})
                    });
                }
                funct3_matches.push(quote!{
                    #funct3 => {
                        let funct7 = Inst::funct7(inst);
                        match funct7 {
                            #(#funct7_matches,)*
                            _ => { Err(EmulatorError::InstructionDecode(format!("unknown/unimplemented opcode+funct3+funct7 {:07b} {:03b} {:07b}", opcode, funct3, funct7))) }
                        }
                    }
                });
            }
        }
        opcode_matches.push(quote! {
            #opcode => {
                let funct3 = Inst::funct3(inst);
                match funct3 {
                    #(#funct3_matches,)*
                    _ => { Err(EmulatorError::InstructionDecode(format!("unknown/unimplemented opcode+funct3 {:07b} {:03b}", opcode, funct3))) }
                }
            }
        });
    }

    // R-Type
    for (opcode, funct3s) in rtype {
        let mut funct3_matches: Vec<TokenStream> = vec![];
        for (funct3, funct7s) in funct3s {
            let mut funct7_matches: Vec<TokenStream> = vec![];
            for (funct7, opname) in funct7s {
                funct7_matches.push(quote!{
                    #funct7 => Ok(Inst::#opname{rd: Inst::rd(inst), rs1: Inst::rs1(inst), rs2: Inst::rs2(inst)})
                });
            }
            funct3_matches.push(quote!{
                #funct3 => {
                    let funct7 = Inst::funct7(inst);
                    match funct7 {
                        #(#funct7_matches,)*
                        _ => { Err(EmulatorError::InstructionDecode(format!("unknown/unimplemented opcode+funct3+funct7 {:07b} {:03b} {:07b}", opcode, funct3, funct7))) }
                    }
                }
            });
        }

        opcode_matches.push(quote! {
            #opcode => {
                let funct3 = Inst::funct3(inst);
                match funct3 {
                    #(#funct3_matches,)*
                    _ => { Err(EmulatorError::InstructionDecode(format!("unknown/unimplemented opcode+funct3 {:07b} {:03b}", opcode, funct3))) }
                }
            }
        });
    }

    // S-Type
    for (opcode, funct3s) in stype {
        let mut funct3_matches: Vec<TokenStream> = vec![];
        for (funct3, opname) in funct3s {
            funct3_matches.push(quote!{
                #funct3 => Ok(Inst::#opname{rs1: Inst::rs1(inst), rs2: Inst::rs2(inst), imm: Inst::imm_s(inst)})
            });
        }
        opcode_matches.push(quote! {
            #opcode => {
                let funct3 = Inst::funct3(inst);
                match funct3 {
                    #(#funct3_matches,)*
                    _ => { Err(EmulatorError::InstructionDecode(format!("unknown/unimplemented opcode+funct3 {:07b} {:03b}", opcode, funct3))) }
                }
            }
        })
    }

    let enum_output = quote! {
        #[derive(Debug)]
        #[allow(non_camel_case_types)] // to keep the compiler from griping about FENCE_I
        /// Enumeration of all known instruction types.
        pub enum Inst {
            #(#variants,)*
        }
    };
    let syntax_tree = syn::parse2(enum_output).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    fs::write(&enum_path, formatted).unwrap();

    let exec_output = quote! {
        impl Inst {
            /// Executes a single instruction.
            pub(crate) fn execute(&self, em: &mut Emulator) {
                match self {
                    #(#exec_matches),*
                }
            }
        }
    };
    let syntax_tree = syn::parse2(exec_output).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    fs::write(&exec_path, formatted).unwrap();

    let decode_output = quote! {
        impl TryFrom<u32> for Inst {
            type Error = EmulatorError;

            fn try_from(inst: u32) -> Result<Self, Self::Error> {
                let opcode = Inst::opcode(inst);
                match opcode {
                    #(#opcode_matches,)*
                    _ => Err(EmulatorError::InstructionDecode(format!("unknown/unimplemented opcode: {:07b}", opcode)))
                }
            }
        }
    };
    let syntax_tree = syn::parse2(decode_output).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    fs::write(&decode_path, formatted).unwrap();

    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=src/rv32i.tab");
}
