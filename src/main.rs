use ::rvem::VirtualMachine;
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Adjust log level (overrides RUST_LOG environment variable)
    #[arg(short, long)]
    log_level: Option<String>,

    /// RISC-V program to emulate
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if let Some(level) = args.log_level {
        env::set_var("RUST_LOG", level);
    }

    env_logger::init();

    let mut vm: VirtualMachine = VirtualMachine::load_from(&args.file)?;
    log::debug!("start address: 0x{:x}", vm.pc);

    if let Ok(value) = env::var("RUST_LOG") {
        if value.to_lowercase() == "trace".to_string() {
            vm.memdump("memdump: ");
        }
    }

    vm.run();

    Ok(())
}
