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
    let disasm_path = Path::new(&out_dir).join("disasm.rs");

    let mut disasm_cases = String::new();
    let mut decode_cases = String::new();
    let mut variants: Vec<TokenStream> = vec![];

    let mut btype: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut itype: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut shamt: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    let mut rtype: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    let mut stype: HashMap<String, HashMap<String, String>> = HashMap::new();

    let preamble = r#"match opcode {"#;

    for line in read_to_string("src/rv32i.tab").unwrap().lines() {
        let pieces: Vec<&str> = line.split(&[' ', '\t', '\r', '\n']).collect();

        let op = sanitize_name(pieces[pieces.len() - 1]);
        let opcode = pieces[pieces.len() - 2];
        let name = format_ident!("{}", op);

        // TODO this will work for now, but could use refinement/refactoring
        match pieces[0] {
            // imm[12|10:5] rs2 rs1 000 imm[4:1|11] 1100011 BEQ
            "imm[12|10:5]" => {
                // B-Type
                let funct3 = btype.entry(opcode.into()).or_default();
                funct3.insert(pieces[3].into(), pieces[6].into());
                // let name = format_ident!("{}", pieces[6]);
                variants.push(quote! {#name{rs1: usize, rs2: usize, imm: u32}});
                // variants += &format!("{}{{rs1: usize, rs2: usize, imm: u32}},\n", pieces[6]);
            }
            // imm[11:0] rs1 000 rd 0010011 ADDI
            "imm[11:0]" => {
                // I-Type
                let funct3 = itype.entry(opcode.into()).or_default();
                funct3.insert(pieces[2].into(), pieces[5].into());
                // let name = format_ident!("{}", pieces[5]);
                variants.push(quote! {#name{rd: usize, rs1: usize, imm: u32}});
                // variants += &format!("{}{{rd: usize, rs1: usize, imm: u32}},\n", pieces[5]);
            }
            // imm[20|10:1|11|19:12] rd 1101111 JAL
            "imm[20|10:1|11|19:12]" => {
                // J-Type
                decode_cases += &format!(
                    "0b{} => Ok(Instruction::{}{{rd: rd!(inst), imm: imm_j!(inst)}}),\n",
                    pieces[2], pieces[3]
                );
                disasm_cases += &format!(
                    "0b{} => format!(\"{} {{:x}}\", (pc as u32).wrapping_add(sext!(imm_j!(inst), 20))),\n",
                    pieces[2],
                    pieces[3].to_lowercase()
                );
                // let name = format_ident!("{}", pieces[3]);
                variants.push(quote! {#name{rd: usize,  imm: u32}});
                // variants += &format!("{}{{rd: usize, imm: u32}},\n", pieces[3]);
            }
            // 0000000 rs2 rs1 000 rd 0110011 ADD
            "0000000" | "0100000" => {
                // R-Type/shamt
                // let name = format_ident!("{}", pieces[6]);
                if pieces[1] == "shamt" {
                    // 0000000 shamt rs1 001 rd 0010011 SLLI
                    variants.push(quote! {#name{rd: usize, rs1: usize, shamt: u32}});
                    let funct3 = shamt.entry(pieces[5].into()).or_default();
                    let funct7 = funct3.entry(pieces[3].into()).or_default();
                    funct7.insert(pieces[0].into(), pieces[6].into());
                    // variants += &format!("{}{{rd: usize, rs1: usize, shamt: u32}},\n", pieces[6]);
                } else {
                    variants.push(quote! {#name{rd: usize, rs1: usize, rs2: usize}});
                    let funct3 = rtype.entry(pieces[5].into()).or_default();
                    let funct7 = funct3.entry(pieces[3].into()).or_default();
                    funct7.insert(pieces[0].into(), pieces[6].into());
                    // variants += &format!("{}{{rd: usize, rs1: usize, rs2: usize}},\n", pieces[6]);
                }
            }
            // imm[11:5] rs2 rs1 000 imm[4:0] 0100011 SB
            "imm[11:5]" => {
                // S-Type
                let funct3 = stype.entry(pieces[5].into()).or_default();
                funct3.insert(pieces[3].into(), pieces[6].into());
                // variants += &format!("{}{{rs1: usize, rs2: usize, imm: u32}},\n", pieces[6]);
                // let name = format_ident!("{}", pieces[6]);
                variants.push(quote! {#name{rs1: usize, rs2: usize, imm: u32}});
            }
            // imm[31:12] rd 0110111 LUI
            "imm[31:12]" => {
                // U-Type
                decode_cases += &format!(
                    "0b{} => Ok(Instruction::{}{{rd: rd!(inst), imm: inst >> 12}}),\n",
                    pieces[2], pieces[3]
                );
                disasm_cases += &format!(
                    "0b{} => format!(\"{} {{}}, 0x{{:x}}\", REG_NAMES[rd!(inst)], inst >> 12),\n",
                    pieces[2],
                    pieces[3].to_lowercase()
                );
                let name = format_ident!("{}", pieces[3]);
                variants.push(quote! {#name{rd: usize, imm: u32}});
                // variants += &format!("{}{{rd: usize, imm: u32}},\n", pieces[3]);
            }
            // TODO is there a way to output build warnings about ignored lines?
            _ => {
                variants.push(quote!{#name});
            }
        }
    }

    // B-Type
    for (opcode, subcodes) in btype {
        decode_cases += &format!("0b{} => {{\n", opcode);
        decode_cases += &format!("let funct3 = funct3!(inst);\n");
        decode_cases += "match funct3 {";
        disasm_cases += &format!("0b{} => {{\n", opcode);
        disasm_cases += &format!("let funct3 = funct3!(inst);\n");
        disasm_cases += "match funct3 {";
        for (funct3, op) in subcodes {
            decode_cases += &format!(
                "0b{} => Ok(Instruction::{}{{ rs1: rs1!(inst), rs2: rs2!(inst), imm: imm_b!(inst) }}),",
                funct3, op
            );
            disasm_cases += &format!(
                "0b{} => format!(\"{} {{}}, {{}}, {{:x}}\", rs1!(inst), rs2!(inst), pc as u32+sext!(imm_b!(inst), 12)),\n",
                funct3,
                op.to_lowercase()
            );
        }
        disasm_cases += "_ => \"unknown/unimplemented\".to_string()";
        disasm_cases += "}},";
        decode_cases += "_ => Err(\"unknown/unimplemented\".to_string())";
        decode_cases += "}},";
    }

    // I-Type
    for (opcode, funct3s) in itype {
        disasm_cases += &format!("0b{} => {{\n", opcode);
        disasm_cases += &format!("let funct3 = funct3!(inst);\n");
        disasm_cases += "match funct3 {";
        decode_cases += &format!("0b{} => {{\n", opcode);
        decode_cases += &format!("let funct3 = funct3!(inst);\n");
        decode_cases += "match funct3 {";
        for (funct3, op) in funct3s {
            decode_cases += &format!(
                "0b{} => Ok(Instruction::{}{{rd: rd!(inst), rs1: rs1!(inst), imm: inst >> 20}}),\n",
                funct3, op
            );
            disasm_cases += &format!(
                "0b{} => format!(\"{} {{}}, {{}}, {{}}\", REG_NAMES[rd!(inst)], REG_NAMES[rs1!(inst)], sext!(inst >> 20, 12)),",
                funct3,
                op.to_lowercase()
            );
        }
        // special case for I-Types w/shamt instead of rs2
        if let Some(funct3s) = shamt.get(&opcode) {
            for (funct3, funct7s) in funct3s {
                disasm_cases += &format!("0b{} => {{", funct3);
                disasm_cases += &format!("let funct7 = funct7!(inst);\n");
                disasm_cases += "match funct7 {";
                decode_cases += &format!("0b{} => {{", funct3);
                decode_cases += &format!("let funct7 = funct7!(inst);\n");
                decode_cases += "match funct7 {";
                for (funct7, op) in funct7s {
                    decode_cases += &format!(
                        "0b{} => Ok(Instruction::{}{{rd: rd!(inst), rs1: rs1!(inst), shamt: shamt!(inst)}}),\n",
                        funct3, op
                    );
                    disasm_cases += &format!(
                        "0b{} => format!(\"{} {{}}, {{}}, {{}}\", REG_NAMES[rd!(inst)], REG_NAMES[rs1!(inst)], rs2!(inst)),\n",
                        funct7,
                        op.to_lowercase()
                    );
                }
                disasm_cases += "_ => \"unknown/unimplemented\".to_string()";
                disasm_cases += "}},";
                decode_cases += "_ => Err(\"unknown/unimplemented\".to_string())";
                decode_cases += "}},";
            }
        }
        disasm_cases += "_ => \"unknown/unimplemented\".to_string()";
        disasm_cases += "}},";
        decode_cases += "_ => Err(\"unknown/unimplemented\".to_string())";
        decode_cases += "}},";
    }

    // R-Type
    for (opcode, funct3s) in rtype {
        decode_cases += &format!("0b{} => {{\n", opcode);
        decode_cases += &format!("let funct3 = funct3!(inst);\n");
        decode_cases += "match funct3 {";
        disasm_cases += &format!("0b{} => {{\n", opcode);
        disasm_cases += &format!("let funct3 = funct3!(inst);\n");
        disasm_cases += "match funct3 {";
        for (funct3, funct7s) in funct3s {
            decode_cases += &format!("0b{} => {{", funct3);
            decode_cases += &format!("let funct7 = funct7!(inst);\n");
            decode_cases += "match funct7 {";
            disasm_cases += &format!("0b{} => {{", funct3);
            disasm_cases += &format!("let funct7 = funct7!(inst);\n");
            disasm_cases += "match funct7 {";
            for (funct7, op) in funct7s {
                disasm_cases += &format!(
                    "0b{} => format!(\"{} {{}}, {{}}, {{}}\", REG_NAMES[rd!(inst)], REG_NAMES[rs1!(inst)], REG_NAMES[rs2!(inst)]),\n",
                    funct7,
                    op.to_lowercase()
                );
                decode_cases += &format!(
                    "0b{} => Ok(Instruction::{}{{ rd: rd!(inst), rs1: rs1!(inst), rs2: rs2!(inst)}}),",
                    funct7,
                    op
                );
            }
            disasm_cases += "_ => \"unknown/unimplemented\".to_string()";
            disasm_cases += "}},";
            decode_cases += "_ => Err(\"unknown/unimplemented\".to_string())";
            decode_cases += "}},";
        }
        disasm_cases += "_ => \"unknown/unimplemented\".to_string()";
        disasm_cases += "}},";
        decode_cases += "_ => Err(\"unknown/unimplemented\".to_string())";
        decode_cases += "}},";
    }

    // S-Type
    for (opcode, funct3s) in stype {
        disasm_cases += &format!("0b{} => {{\n", opcode);
        disasm_cases += &format!("let funct3 = funct3!(inst);\n");
        disasm_cases += "match funct3 {";
        decode_cases += &format!("0b{} => {{\n", opcode);
        decode_cases += &format!("let funct3 = funct3!(inst);\n");
        decode_cases += "match funct3 {";
        for (funct3, op) in funct3s {
            disasm_cases += &format!(
                "0b{} => format!(\"{} {{}}, {{}}({{}})\", REG_NAMES[rs2!(inst)], sext!(imm_s!(inst), 12), REG_NAMES[rs1!(inst)]),",
                funct3,
                op.to_lowercase()
            );
            decode_cases += &format!(
                "0b{} => Ok(Instruction::{}{{rs1: rs1!(inst), rs2: rs2!(inst), imm: imm_s!(inst)}}),\n",
                funct3, op
            );
        }
        disasm_cases += "_ => \"unknown/unimplemented\".to_string()";
        disasm_cases += "}},";
        decode_cases += "_ => Err(\"unknown/unimplemented\".to_string())";
        decode_cases += "}},";
    }

    let disasm_postamble = r#"0b1110011 => {
                    if inst == 0b1110011 { // ECALL
                        "ecall".to_string()
                    } else {
                        "unknown/unimplemented".to_string()
                    }
                }
                _ => {
                    "unknown/unimplemented".to_string()
                }
            }"#;

    let decode_postamble = r#"0b1110011 => {
                    if inst == 0b1110011 { // ECALL
                        Ok(Instruction::ECALL)
                    } else {
                        Err("unimplemented syscall".to_string())
                    }
                }
                _ => {
                    Err(format!("unimplemented opcode: {:07b}", opcode))
                }
            }"#;

    let mut names: Vec<Ident> = vec![];
    let vname = format_ident!("Foo");
    let foo = quote! {Bar};
    // names.push(vname);
    // let mut variants: Vec<proc_macro2::TokenStream> = vec![];
    // variants.push(quote!{#vname{rd: usize}});
    // variants.push(quote! {ECALL,});
    let output = quote! {
        pub enum Instruction {
           #(#variants),*
        }
    };
    let syntax_tree = syn::parse2(output).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);

    fs::write(
        &decode_path,
        format!("{preamble} {decode_cases} {decode_postamble}"),
    )
    .unwrap();
    fs::write(
        &disasm_path,
        format!("{preamble} {disasm_cases} {disasm_postamble}"),
    )
    .unwrap();
    fs::write(&enum_path, formatted).unwrap();
    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=src/rv32i.tab");
}
