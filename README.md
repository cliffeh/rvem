# rvem
a RISC-V emulator

`rvem` is an emulator that supports a subset of the RISC-V instruction set - specifically, the rv32i base instruction set and the rv32m extensions.

## Building & Running
The emulator is written in Rust. It can be compiled using `cargo build` and/or run using `cargo run`. (Do `cargo run -- --help` for a list of supported command-line options.)

The source repository also contains a handful of RISC-V assembly programs for testing. In order to build these you'll need to install a riscv64 binutils package appropriate to your environment. Known working packages include `binutils-riscv64-linux-gnu` on Ubuntu (installable via apt) and `riscv64-elf-binutils` on MacOS (installable via brew).

There is also a helpful little Makefile for running various tests, traces, binary dumps, etc. Do `make help` to for a list of supported targets.

## What Works?
All of the test programs run successfully - or at least they do on _my_ computer ;-) - although I'd like to have more testing in place (natch). I've also been trying to maintain a list of [TODOs](TODO.md) for future improvements.

# References
* [RISC-V Instruction Set Manual](https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf) - Massive PDF describing the entire spec
* [RV32I Base Integer Instruction Set](https://docs.openhwgroup.org/projects/cva6-user-manual/01_cva6_user/RISCV_Instructions_RV32I.html) - Pseudocode reference for each instruction
* [RISC-V Instruction Encoder/Decoder](https://luplab.gitlab.io/rvcodecjs/) - Nifty little online instruction encoder/decoder
* [Hello World](https://smist08.wordpress.com/2019/09/07/risc-v-assembly-language-hello-world/) - Where I got hello.s
* [Assembly Programmer's Manual](https://github.com/riscv-non-isa/riscv-asm-manual/blob/main/src/asm-manual.adoc) - Helpful assembly programming manual
* [Assembler Reference](https://michaeljclark.github.io/asm.html) - Yet another reference, contains some helpful details about the ELF format
* [RISC-V RV32I assembly with Ripes simulator](https://dantalion.nl/2022/02/25/risc-v-rv32i-assembly.html) - Gives some high-level basics and links out to a useful [simulator](https://github.com/mortbopet/Ripes)
* [Quick Reference Card](https://github.com/dylanmc/CS2-RISC-V/raw/master/Extra%20stuff/RISC-V%20quick%20ref%20card.pdf)
