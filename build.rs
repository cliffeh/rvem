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

struct InstDef {
    op: Ident,
    funct3: Option<u32>,
    funct7: Option<u32>,
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let decode_path = Path::new(&out_dir).join("decode.rs");
    let enum_path = Path::new(&out_dir).join("enum.rs");

    let mut variants: Vec<TokenStream> = vec![];

    let mut btype: HashMap<u32, HashMap<u32, Ident>> = HashMap::new();
    let mut itype: HashMap<u32, HashMap<u32, Ident>> = HashMap::new();
    let mut shamt: HashMap<u32, HashMap<u32, HashMap<u32, Ident>>> = HashMap::new();
    let mut rtype: HashMap<u32, HashMap<u32, HashMap<u32, Ident>>> = HashMap::new();
    let mut stype: HashMap<u32, HashMap<u32, Ident>> = HashMap::new();

    let mut opcode_matches: Vec<TokenStream> = vec![];

    for line in read_to_string("src/rv32i.tab").unwrap().lines() {
        let pieces: Vec<&str> = line.split(&[' ', '\t', '\r', '\n']).collect();

        let opname = format_ident!("{}", sanitize_name(pieces[pieces.len() - 1]));
        let opcode = u32::from_str_radix(pieces[pieces.len() - 2], 2).unwrap();

        // TODO this will work for now, but could use refinement/refactoring
        match pieces[0] {
            // B-Type: imm[12|10:5] rs2 rs1 000 imm[4:1|11] 1100011 BEQ
            "imm[12|10:5]" => {
                variants.push(quote! {#opname{rs1: usize, rs2: usize, imm: u32}});

                let funct3 = u32::from_str_radix(pieces[3], 2).unwrap();
                let funct3s = btype.entry(opcode).or_default();
                funct3s.insert(funct3, opname);
            }
            // I-Type: imm[11:0] rs1 000 rd 0010011 ADDI
            "imm[11:0]" => {
                variants.push(quote! {#opname{rd: usize, rs1: usize, imm: u32}});

                let funct3 = u32::from_str_radix(pieces[2], 2).unwrap();
                let funct3s = itype.entry(opcode).or_default();
                funct3s.insert(funct3, opname);
            }
            // J-Type: imm[20|10:1|11|19:12] rd 1101111 JAL
            "imm[20|10:1|11|19:12]" => {
                variants.push(quote! {#opname{rd: usize,  imm: u32}});

                opcode_matches.push(quote! {
                    #opcode => Ok(Instruction::#opname{rd: rd!(inst), imm: imm_j!(inst)})
                });
            }
            // R-Type: 0000000 rs2 rs1 000 rd 0110011 ADD
            "0000000" | "0100000" => {
                let funct3 = u32::from_str_radix(pieces[3], 2).unwrap();
                let funct7 = u32::from_str_radix(pieces[0], 2).unwrap();
                // shamt (special case): 0000000 shamt rs1 001 rd 0010011 SLLI
                if pieces[1] == "shamt" {
                    variants.push(quote! {#opname{rd: usize, rs1: usize, shamt: u32}});

                    let funct3s = shamt.entry(opcode).or_default();
                    let funct7s = funct3s.entry(funct3).or_default();
                    funct7s.insert(funct7, opname);
                } else {
                    // 0000000 rs2 rs1 000 rd 0110011 ADD
                    variants.push(quote! {#opname{rd: usize, rs1: usize, rs2: usize}});

                    let funct3s = rtype.entry(opcode).or_default();
                    let funct7s = funct3s.entry(funct3).or_default();
                    funct7s.insert(funct7, opname);
                }
            }
            // S-Type: imm[11:5] rs2 rs1 000 imm[4:0] 0100011 SB
            "imm[11:5]" => {
                variants.push(quote! {#opname{rs1: usize, rs2: usize, imm: u32}});

                let funct3 = u32::from_str_radix(pieces[3], 2).unwrap();
                let funct3s = stype.entry(opcode).or_default();
                funct3s.insert(funct3, opname);
            }
            // U-Type: imm[31:12] rd 0110111 LUI
            "imm[31:12]" => {
                variants.push(quote! {#opname{rd: usize, imm: u32}});

                opcode_matches.push(quote! {
                    #opcode => Ok(Instruction::#opname{rd: rd!(inst), imm: inst >> 12})
                });
            }
            // TODO is there a way to output build warnings about ignored lines?
            _ => {
                variants.push(quote! {#opname});
                if opname == "ECALL" {
                    // TODO get rid of this?
                    opcode_matches.push(quote! {
                        #opcode => Ok(Instruction::ECALL)
                    });
                }
            }
        }
    }

    // B-Type
    for (opcode, funct3s) in btype {
        let mut funct3_matches: Vec<TokenStream> = vec![];
        for (funct3, opname) in funct3s {
            funct3_matches.push(quote!{
                #funct3 => Ok(Instruction::#opname{rs1: rs1!(inst), rs2: rs2!(inst), imm: imm_b!(inst)})
            });
        }
        opcode_matches.push(quote! {
            #opcode => {
                let funct3 = funct3!(inst);
                match funct3 {
                    #(#funct3_matches,)*
                    _ => { Err(format!("unknown/unimplemented opcode+funct3 {:07b} {:03b}", opcode, funct3))}
                }
            }
        })
    }

    // I-Type
    for (opcode, funct3s) in itype {
        let mut funct3_matches: Vec<TokenStream> = vec![];
        for (funct3, opname) in funct3s {
            funct3_matches.push(quote! {
                #funct3 => Ok(Instruction::#opname{rd: rd!(inst), rs1: rs1!(inst), imm: inst >> 20})
            });
        }
        opcode_matches.push(quote! {
            #opcode => {
                let funct3 = funct3!(inst);
                match funct3 {
                    #(#funct3_matches,)*
                    _ => { Err(format!("unknown/unimplemented opcode+funct3 {:07b} {:03b}", opcode, funct3))}
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
                    #funct7 => Ok(Instruction::#opname{rd: rd!(inst), rs1: rs1!(inst), rs2: rs2!(inst)})
                });
            }
            funct3_matches.push(quote!{
                #funct3 => {
                    let funct7 = funct7!(inst);
                    match funct7 {
                        #(#funct7_matches,)*
                        _ => { Err(format!("unknown/unimplemented opcode+funct3+funct7 {:07b} {:03b} {:07b}", opcode, funct3, funct7))}
                    }
                }
            });
        }
        // special case for I-Types w/shamt instead of rs2
        if let Some(funct3s) = shamt.get(&opcode) {
            for (funct3, funct7s) in funct3s {
                let mut funct7_matches: Vec<TokenStream> = vec![];
                for (funct7, opname) in funct7s {
                    funct7_matches.push(quote!{
                        #funct7 => Ok(Instruction::#opname{rd: rd!(inst), rs1: rs1!(inst), shamt: shamt!(inst)})
                    });
                }
                funct3_matches.push(quote!{
                    #funct3 => {
                        let funct7 = funct7!(inst);
                        match funct7 {
                            #(#funct7_matches,)*
                            _ => { Err(format!("unknown/unimplemented opcode+funct3+funct7 {:07b} {:03b} {:07b}", opcode, funct3, funct7))}
                        }
                    }
                });
            }
        }
        opcode_matches.push(quote! {
            #opcode => {
                let funct3 = funct3!(inst);
                match funct3 {
                    #(#funct3_matches,)*
                    _ => { Err(format!("unknown/unimplemented opcode+funct3 {:07b} {:03b}", opcode, funct3))}
                }
            }
        });
    }

    // S-Type
    for (opcode, funct3s) in stype {
        let mut funct3_matches: Vec<TokenStream> = vec![];
        for (funct3, opname) in funct3s {
            funct3_matches.push(quote!{
                #funct3 => Ok(Instruction::#opname{rs1: rs1!(inst), rs2: rs2!(inst), imm: imm_s!(inst)})
            });
        }
        opcode_matches.push(quote! {
            #opcode => {
                let funct3 = funct3!(inst);
                match funct3 {
                    #(#funct3_matches,)*
                    _ => { Err(format!("unknown/unimplemented opcode+funct3 {:07b} {:03b}", opcode, funct3))}
                }
            }
        })
    }

    let enum_output = quote! {
        #[derive(Debug)]
        pub enum Instruction {
           #(#variants),*
        }
    };
    let syntax_tree = syn::parse2(enum_output).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    fs::write(&enum_path, formatted).unwrap();

    let decode_output = quote! {
        impl TryFrom<u32> for Instruction {
            type Error = String;

            fn try_from(inst: u32) -> Result<Self, Self::Error> {
                let opcode = opcode!(inst);
                match opcode {
                    #(#opcode_matches,)*
                    _ => Err(format!("unknown/unimplemented opcode: {:07b}", opcode))
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
