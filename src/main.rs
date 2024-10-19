use ::rvem::VirtualMachine;
use clap::Parser;
use rvem::DEFAULT_MEMORY_SIZE;
use std::env;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set log level (overrides RUST_LOG environment variable)
    #[arg(short, long)]
    log_level: Option<String>,

    /// Memory to allocate for the emulator
    #[arg(short, long, default_value_t = DEFAULT_MEMORY_SIZE)]
    memory: usize,

    /// Initialize the stack pointer [default: beginning of .text section]
    #[arg(long)]
    sp: Option<usize>,

    /// RISC-V program to emulate
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if let Some(log_level) = args.log_level {
        env::set_var("RUST_LOG", log_level);
    }

    env_logger::init();

    let mut vm: VirtualMachine = VirtualMachine::load_from(&args.file, args.memory)?;
    log::debug!("start address: 0x{:x}", vm.pc);

    if let Ok(value) = env::var("RUST_LOG") {
        if value.to_lowercase() == "trace".to_string() {
            vm.memdump("memdump: ");
        }
    }

    vm.run();

    Ok(())
}
