use ::rvem::Emulator;
use clap::Parser;
use rvem::{EmulatorError, DEFAULT_MEMORY_SIZE};
use std::{env, process};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Dump the program and exit
    #[arg(short = 'D', long, default_value_t = false)]
    dump: bool,

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

fn main() -> Result<(), EmulatorError> {
    let args = Args::parse();

    if let Some(log_level) = args.log_level {
        env::set_var("RUST_LOG", log_level);
    }

    env_logger::init();

    let mut em: Emulator = Emulator::load_from(&args.file, Some(args.memory))?;

    if args.dump {
        println!("{em:#?}");
        process::exit(0);
    } else if log::log_enabled!(log::Level::Trace) {
        log::trace!("{:#?}", em);
    }

    em.run()
}
