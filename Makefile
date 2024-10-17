default: help

check: hello  ## run rvem against assembled RISC-V program(s)
	cargo run
.PHONY: check

debug: hello  ## run rvem with debug logging enabled
	RUST_LOG=debug cargo run
.PHONY: debug

trace: hello  ## run rvem with trace logging enabled
	RUST_LOG=trace cargo run
.PHONY: trace

fib: fib.o  ## build RISC-V `fib` program
	riscv64-elf-ld -melf32lriscv -o $@ $<

fib.o: fib.s
	riscv64-elf-as -march=rv32i $< -o $@

itoa.o: itoa.s
	riscv64-elf-as -march=rv32i $< -o $@

hello: hello.o  ## build RISC-V `hello` program
	riscv64-elf-ld -melf32lriscv -o $@ $<

hello.o: hello.s
	riscv64-elf-as -march=rv32i $< -o $@

strlen: strlen.o  ## build RISC-V `strlen` program
	riscv64-elf-ld -melf32lriscv -o $@ $<

strlen.o: strlen.s
	riscv64-elf-as -march=rv32i $< -o $@

dump-fib: fib  ## disassemble executable sections of `fib`
	riscv64-elf-objdump -d $<
.PHONY: dump

dumpall-fib: fib  ## disassemble all sections of `fib`
	riscv64-elf-objdump -D $<
.PHONY: dump

dump-hello: hello  ## disassemble executable sections of `hello`
	riscv64-elf-objdump -d $<
.PHONY: dump

dumpall-hello: hello  ## disassemble all sections of `hello`
	riscv64-elf-objdump -D $<
.PHONY: dump

format:  # beautify all rust code
	cargo fmt
.PHONY: format

clean:  ## remove intermediate object files
	rm -f *.o
.PHONY: clean

binclean: clean  ## remove object files and assembled RISC-V programs
	rm -f hello
.PHONY: binclean

realclean: clean binclean  ## remove everything but source code
	rm -rf target
.PHONY: realclean

help: ## show this help
	@echo "\nSpecify a command. The choices are:\n"
	@grep -E '^[0-9a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[0;36m%-14s\033[m %s\n", $$1, $$2}'
	@echo ""
.PHONY: help
