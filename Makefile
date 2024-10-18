PROGS=hello fib strlen
DEFAULT_PROG=hello
PROG?=$(DEFAULT_PROG)

default: help

run: $(PROG)  ## run rvem
	cargo run -- $<
.PHONY: hello

debug: $(PROG)  ## run rvem with debug logging enabled
	RUST_LOG=debug cargo run -- $<
.PHONY: debug

trace: $(PROG)  ## run rvem with trace logging enabled
	RUST_LOG=trace cargo run -- $<
.PHONY: trace

dump: $(PROG)  ## disassemble executable sections
	riscv64-elf-objdump -d $<
.PHONY: dump

dumpall: $(PROG)  ## disassemble all sections
	riscv64-elf-objdump -D $<
.PHONY: dump

format:  ## beautify rust code
	cargo fmt
.PHONY: format

clean:  ## remove intermediate object files
	rm -f *.o
.PHONY: clean

binclean:  ## remove assembled RISC-V programs
	rm -f $(PROGS)
.PHONY: binclean

realclean: clean binclean  ## remove everything but source code
	rm -rf target
.PHONY: realclean

help: ## show this help
	@echo
	@echo "Specify a command. The choices are:"
	@echo
	@grep -E '^[0-9a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[0;36m%-14s\033[m %s\n", $$1, $$2}'
	@echo
	@echo "Available environment variables:"
	@echo
	@printf "  \033[0;36m%-14s\033[m %s\n" RUST_LOG "sets log level (debug, trace, error, etc.)"
	@printf "  \033[0;36m%-14s\033[m %s\n" PROG "set the program to run"
	@echo
	@echo "Available programs:"
	@echo
	@printf "  \033[0;36m%-14s\033[m %s\n" hello "(default) your bog standard 'Hello, World!' program"
	@printf "  \033[0;36m%-14s\033[m %s\n" fib "computes the Fibonacci sequence up to fib(11)"
	@printf "  \033[0;36m%-14s\033[m %s\n" strlen "computes the string length of 'The quick brown fox jumps over the lazy dog.'"
	@echo ""
	@echo "Examples:"
	@echo
	@printf "  \033[0;36m%-22s\033[m %s\n" "make run" "builds and runs 'hello'"
	@printf "  \033[0;36m%-22s\033[m %s\n" "PROG=fib make trace" "builds and runs 'fib' with trace logging turned on"
	@printf "  \033[0;36m%-22s\033[m %s\n" "PROG=strlen make dump" "dumps the executable section of \`strlen\`"
.PHONY: help

### targets that actually build things
fib: fib.o
	riscv64-elf-ld -melf32lriscv -o $@ $<

fib.o: fib.s
	riscv64-elf-as -march=rv32i $< -o $@

hello: hello.o
	riscv64-elf-ld -melf32lriscv -o $@ $<

hello.o: hello.s
	riscv64-elf-as -march=rv32i $< -o $@

strlen: strlen.o
	riscv64-elf-ld -melf32lriscv -o $@ $<

strlen.o: strlen.s
	riscv64-elf-as -march=rv32i $< -o $@
