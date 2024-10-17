use goblin::elf::Elf;
use std::fs::File;
use std::io::Read;

fn load_elf_data(path: &str, out: &mut [u8]) -> Result<usize, Box<dyn std::error::Error>> {
    let mut exec_start = 0usize;

    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let elf = Elf::parse(&buf)?;

    for section in &elf.section_headers {
        if section.is_alloc() {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf: [u8; 1 << 20] = [0u8; 1 << 20];
    let mut pc = load_elf_data("hello", &mut buf)?;
    println!("PC: 0x{pc:x}");

    while buf[pc] != 0 {
        let inst = u32::from_le_bytes(
            buf[pc..pc + 4]
                .try_into()
                .expect("incorrect byte slice length"),
        );
        println!("{pc:x}: {inst:08x}");
        pc += 4;
    }

    Ok(())
}
