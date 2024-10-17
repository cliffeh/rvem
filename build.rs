// build.rs

use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::read_to_string;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("rv32i.rs");

    let template_path = Path::new("src/run.in");
    let template = read_to_string(template_path).unwrap();

    let mut cases = String::new();

    let mut itype: HashMap<String, HashMap<String, String>> = HashMap::new();

    for line in read_to_string("src/rv32i.tab").unwrap().lines() {
        let pieces: Vec<&str> = line.split(&[' ', '\t', '\r', '\n']).collect();

        match pieces[0] {
            // imm[31:12] rd 0110111 LUI
            "imm[31:12]" => {
                // U-Type
                cases += &format!("0b{} => self.{}(rd!(inst), inst >> 12),\n", pieces[2], pieces[3].to_lowercase());
            }
            // imm[11:0] rs1 000 rd 0010011 ADDI
            "imm[11:0]" => {
                let funct3 = itype.entry(pieces[4].into()).or_default();
                funct3.insert(pieces[2].into(), pieces[5].into());
            }
            _ => {}
        }
    }

    for (opcode, subcodes) in itype {
        cases += &format!("0b{} => {{\n", opcode);
        cases += &format!("let funct3: u32 = funct3!(inst);\n");
        cases += "match funct3 {";
        for (funct3, op) in subcodes {
            cases += &format!("0b{} => self.{}(rd!(inst), rs1!(inst), inst >> 20),\n", funct3, op.to_lowercase());
        }
        cases += "_ => log::error!(\"{:x} {:08x}: unknown opcode+funct3: {:07b} {:03b}\", self.pc, inst, opcode, funct3)";
        cases += "}},";
    }

    fs::write(&dest_path, template.replace("/* CASES */", &cases)).unwrap();
    println!("cargo::rerun-if-changed=src/");
}
