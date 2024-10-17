// build.rs

use std::env;
use std::fs;
use std::fs::read_to_string;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("rv32i.rs");

    let template_path = Path::new("src/run.in");
    let mut template = read_to_string(template_path).unwrap();

    let mut cases = String::new();

    // cases.repl

    for line in read_to_string("src/rv32i.tab").unwrap().lines() {
        let pieces: Vec<&str> = line.split(&[' ', '\t', '\r', '\n']).collect();

        match pieces[0] {
            "imm[31:12]" => {
                // U-Type
                cases += format!(
                    "0b{} => self.{}(rd!(inst), inst >> 12),\n",
                    pieces[2],
                    pieces[3].to_lowercase()
                )
                .as_str();
            }
            _ => {}
        }
    }

    fs::write(&dest_path, template.replace("/* CASES */", &cases)).unwrap();
    println!("cargo::rerun-if-changed=src/");
}
