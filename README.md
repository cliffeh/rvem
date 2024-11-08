# rvem
A RISC-V emulator

`rvem` is an emulator that supports a subset of the RISC-V instruction set -
specifically, the rv32i base instruction set and rv32m (multiplication/division)
extensions.

## Building & Running
The emulator is written in Rust. It can be compiled using `cargo build` and/or
run using `cargo run`. (Do `cargo run -- --help` for a list of supported
options.) There is a helpful little Makefile for running various tests, traces,
binary dumps, etc. Do `make help` for a list of supported targets.

The repository also contains a handful of RISC-V assembly programs for testing,
as well as compiled and linked binaries. If you want to build these yourself
and/or use things like the `objdump` and `readelf` make targets, you'll need to
install a riscv64 binutils package appropriate to your environment. Known
working packages include `binutils-riscv64-linux-gnu` on Ubuntu (installable via
apt) and `riscv64-elf-binutils` on MacOS (installable via brew).

All of the test programs can be emulated successfully - or at least they can on
_my_ computer ;-) - although I'd like to have more testing in place (natch).
I've been trying to maintain a list of [TODOs](TODO.md) for future improvements.

## Caveats
This emulator only supports running statically-linked binaries, and (probably)
only those assembled from source; i.e., I wouldn't expect a program
(cross-)compiled with GCC and dynamically linked to libc to work. It also only
supports the base instruction set and multiplication extensions, and only has a
handful of syscalls implemented.

## Toolchains
One of the testing challenges is finding a toolchain that will cross-compile for
a RISC-V architecture, including the necessary ABI. For assembling and linking
some platforms have available packages (see above), but at least for the
`helloc` test program (cross-compiled from C code) I had to build my own
toolchain. I don't think that is a reasonable expectation for someone else to
do, buit for posterity this is roughly how I went about it:

```shell
# NB much repo, many clone
git clone --recursive https://github.com/riscv/riscv-gnu-toolchain
riscv-gnu-toolchain
./configure --prefix=/opt/riscv --with-arch=rv32im --with-abi=ilp32 --enable-multilib
# NB this takes (roughly) forever to build
sudo make
# `hello.c` source code left as an exercise for the reader ;-)
/opt/riscv/bin/riscv32-unknown-elf-gcc -march=rv32im -mabi=ilp32 -o hello hello.c
```

Note that the upshot of this is that if you've installed a pre-packaged
toolchain you likely won't be able to use it for things like `make objdump`
on `helloc`.

## References
* [RISC-V Instruction Set Manual](https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf) - Massive PDF describing the entire spec
* [RV32I Base Integer Instruction Set](https://docs.openhwgroup.org/projects/cva6-user-manual/01_cva6_user/RISCV_Instructions_RV32I.html) - Pseudocode reference for each instruction
* [RISC-V Instruction Encoder/Decoder](https://luplab.gitlab.io/rvcodecjs/) - Nifty little online instruction encoder/decoder
* [Hello World](https://smist08.wordpress.com/2019/09/07/risc-v-assembly-language-hello-world/) - Where I got hello.s
* [Assembly Programmer's Manual](https://github.com/riscv-non-isa/riscv-asm-manual/blob/main/src/asm-manual.adoc) - Helpful assembly programming manual
* [Assembler Reference](https://michaeljclark.github.io/asm.html) - Yet another reference, contains some helpful details about the ELF format
* [RISC-V RV32I assembly with Ripes simulator](https://dantalion.nl/2022/02/25/risc-v-rv32i-assembly.html) - Gives some high-level basics and links out to a useful [simulator](https://github.com/mortbopet/Ripes)
* [Quick Reference Card](https://github.com/dylanmc/CS2-RISC-V/raw/master/Extra%20stuff/RISC-V%20quick%20ref%20card.pdf)
* [Rust Cross-Compilation](https://danielmangum.com/posts/risc-v-bytes-rust-cross-compilation/) - Experimenting with this a bit
