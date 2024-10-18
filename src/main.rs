use ::rvem::VirtualMachine;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // TODO cli args!
    let mut vm: VirtualMachine = VirtualMachine::load_from("fib")?;
    log::debug!("start address: 0x{:x}", vm.pc);

    if let Ok(value) = env::var("RUST_LOG") {
        if value.to_lowercase() == "trace".to_string() {
            vm.memdump("memdump: ");
        }
    }

    vm.run();

    Ok(())
}
