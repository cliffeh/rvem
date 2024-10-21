// build.rs

use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::read_to_string;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let exec_path = Path::new(&out_dir).join("exec.rs");
    let _debug_path = Path::new(&out_dir).join("debug.rs"); // TODO

    let mut exec_cases = String::new();
    let mut debug_cases = String::new();

    let mut btype: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut itype: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut shamt: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    let mut rtype: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    let mut stype: HashMap<String, HashMap<String, String>> = HashMap::new();

    let preamble = r#"match opcode {"#;

    for line in read_to_string("src/rv32i.tab").unwrap().lines() {
        let pieces: Vec<&str> = line.split(&[' ', '\t', '\r', '\n']).collect();

        // TODO this will work for now, but could use refinement/refactoring
        match pieces[0] {
            // imm[12|10:5] rs2 rs1 000 imm[4:1|11] 1100011 BEQ
            "imm[12|10:5]" => {
                // B-Type
                let funct3 = btype.entry(pieces[5].into()).or_default();
                funct3.insert(pieces[3].into(), pieces[6].into());
            }
            // imm[11:0] rs1 000 rd 0010011 ADDI
            "imm[11:0]" => {
                // I-Type
                let funct3 = itype.entry(pieces[4].into()).or_default();
                funct3.insert(pieces[2].into(), pieces[5].into());
            }
            // imm[20|10:1|11|19:12] rd 1101111 JAL
            "imm[20|10:1|11|19:12]" => {
                // J-Type
                exec_cases += &format!("0b{} => self.{}(rd!(inst), imm_j!(inst)),\n", pieces[2], pieces[3].to_lowercase());
                debug_cases += &format!("0b{} => log::debug!({}),", pieces[2], pieces[3].to_lowercase());
            }
            // 0000000 rs2 rs1 000 rd 0110011 ADD
            "0000000" | "0100000" => {
                // R-Type/shamt
                if pieces[1] == "shamt" {
                    // 0000000 shamt rs1 001 rd 0010011 SLLI
                    let funct3 = shamt.entry(pieces[5].into()).or_default();
                    let funct7 = funct3.entry(pieces[3].into()).or_default();
                    funct7.insert(pieces[0].into(), pieces[6].into());
                } else {
                    let funct3 = rtype.entry(pieces[5].into()).or_default();
                    let funct7 = funct3.entry(pieces[3].into()).or_default();
                    funct7.insert(pieces[0].into(), pieces[6].into());
                }
            }
            // imm[11:5] rs2 rs1 000 imm[4:0] 0100011 SB
            "imm[11:5]" => {
                // S-Type
                let funct3 = stype.entry(pieces[5].into()).or_default();
                funct3.insert(pieces[3].into(), pieces[6].into());
            }
            // imm[31:12] rd 0110111 LUI
            "imm[31:12]" => {
                // U-Type
                exec_cases += &format!("0b{} => self.{}(rd!(inst), inst >> 12),\n", pieces[2], pieces[3].to_lowercase());
            }
            // TODO is there a way to output build warnings about ignored lines?
            _ => {}
        }
    }

    // B-Type
    for (opcode, subcodes) in btype {
        exec_cases += &format!("0b{} => {{\n", opcode);
        exec_cases += &format!("let funct3 = funct3!(inst);\n");
        exec_cases += "match funct3 {";
        for (funct3, op) in subcodes {
            exec_cases += &format!("0b{} => self.{}(rs1!(inst), rs2!(inst), imm_b!(inst)),\n", funct3, op.to_lowercase());
        }
        exec_cases += "_ => log::error!(\"{:x} {:08x}: unknown opcode+funct3: {:07b} {:03b}\", self.pc, inst, opcode, funct3)";
        exec_cases += "}},";
    }

    // I-Type
    for (opcode, funct3s) in itype {
        exec_cases += &format!("0b{} => {{\n", opcode);
        exec_cases += &format!("let funct3 = funct3!(inst);\n");
        exec_cases += "match funct3 {";
        for (funct3, op) in funct3s {
            exec_cases += &format!("0b{} => self.{}(rd!(inst), rs1!(inst), inst >> 20),\n", funct3, op.to_lowercase());
        }
        // special case for I-Types w/shamt instead of rs2
        if let Some(funct3s) = shamt.get(&opcode) {
            for (funct3, funct7s) in funct3s {
                exec_cases += &format!("0b{} => {{", funct3);
                exec_cases += &format!("let funct7 = funct7!(inst);\n");
                exec_cases += "match funct7 {";
                for (funct7, op) in funct7s {
                    exec_cases += &format!("0b{} => self.{}(rd!(inst), rs1!(inst), rs2!(inst)),\n", funct7, op.to_lowercase());
                }
                exec_cases +=
                    "_ => log::error!(\"{:x} {:08x}: unknown opcode+funct3+funct7: {:07b} {:03b} {:07b}\", self.pc, inst, opcode, funct3, funct7)";
                exec_cases += "}},";
            }
        }
        exec_cases += "_ => log::error!(\"{:x} {:08x}: unknown opcode+funct3: {:07b} {:03b}\", self.pc, inst, opcode, funct3)";
        exec_cases += "}},";
    }

    // R-Type
    for (opcode, funct3s) in rtype {
        exec_cases += &format!("0b{} => {{\n", opcode);
        exec_cases += &format!("let funct3 = funct3!(inst);\n");
        exec_cases += "match funct3 {";
        for (funct3, funct7s) in funct3s {
            exec_cases += &format!("0b{} => {{", funct3);
            exec_cases += &format!("let funct7 = funct7!(inst);\n");
            exec_cases += "match funct7 {";
            for (funct7, op) in funct7s {
                exec_cases += &format!("0b{} => self.{}(rd!(inst), rs1!(inst), rs2!(inst)),\n", funct7, op.to_lowercase());
            }
            exec_cases +=
                "_ => log::error!(\"{:x} {:08x}: unknown opcode+funct3+funct7: {:07b} {:03b} {:07b}\", self.pc, inst, opcode, funct3, funct7)";
            exec_cases += "}},";
        }
        exec_cases += "_ => log::error!(\"{:x} {:08x}: unknown opcode+funct3: {:07b} {:03b}\", self.pc, inst, opcode, funct3)";
        exec_cases += "}},";
    }

    // S-Type
    for (opcode, funct3s) in stype {
        exec_cases += &format!("0b{} => {{\n", opcode);
        exec_cases += &format!("let funct3 = funct3!(inst);\n");
        exec_cases += "match funct3 {";
        for (funct3, op) in funct3s {
            exec_cases += &format!("0b{} => self.{}(rs1!(inst), rs2!(inst), imm_s!(inst)),\n", funct3, op.to_lowercase());
        }
        exec_cases += "_ => log::error!(\"{:x} {:08x}: unknown opcode+funct3: {:07b} {:03b}\", self.pc, inst, opcode, funct3)";
        exec_cases += "}},";
    }

    let exec_postamble = r#"0b1110011 => {
                    if inst == 0b1110011 { // ECALL
                        self.ecall();
                    } else {
                        log::error!("{:x} {:08x}: unimplemented environment call", self.pc, inst);
                    }
                }
                _ => {
                    log::error!("{:x} {:08x}: unimplemented opcode: {:07b}", self.pc, inst, opcode);
                }
            }"#;

    fs::write(&exec_path, format!("{preamble} {exec_cases} {exec_postamble}")).unwrap();
    println!("cargo::rerun-if-changed=src/lib.rs");
    println!("cargo::rerun-if-changed=src/rv32i.tab");
}
