use ::rvem::Emulator;
use clap::Parser;
use rvem::{EmulatorError, DEFAULT_MEMORY_SIZE};
use std::{env, process};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
/// A RISC-V emulator.
///
/// rvem is an emulator that supports a subset of the RISC-V instruction set -
/// specifically, the rv32i base instruction set and the rv32m extensions.
struct Args {
    /// Dump the program and exit
    #[arg(short = 'D', long, default_value_t = false)]
    dump: bool,

    /// Set log level (overrides RUST_LOG environment variable)
    ///
    /// Available options include: error (default), warn, info, debug,
    /// trace (most verbose).
    #[arg(short, long)]
    log_level: Option<String>,

    /// Memory to allocate for the emulator
    #[arg(short, long, value_name = "BYTES", default_value_t = DEFAULT_MEMORY_SIZE)]
    memory: usize,

    /// RISC-V program to emulate
    file: String,
}

fn emulate(args: Args) -> Result<(), EmulatorError> {
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

fn main() -> Result<(), EmulatorError> {
    let args = Args::parse();
    emulate(args)
}
