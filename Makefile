PROGS=hello fib1 fib2 strlen
DEFAULT_PROG=hello
PROG?=$(DEFAULT_PROG)

# Detect the platform
UNAME_S := $(shell uname -s)

# Set the toolchain prefix based on the platform
ifeq ($(UNAME_S), Linux)
    ASPREFIX = riscv64-linux-gnu
else ifeq ($(UNAME_S), Darwin)
    ASPREFIX = riscv64-elf
else
    $(error Unsupported platform: $(UNAME_S))
endif

default: help

run: $(PROG)  ## run rvem
	cargo run -- tests/data/$<
.PHONY: hello

debug: $(PROG)  ## run rvem with debug logging enabled
	RUST_LOG=debug cargo run -- tests/data/$<
.PHONY: debug

trace: $(PROG)  ## run rvem with trace logging enabled
	RUST_LOG=trace cargo run -- tests/data/$<
.PHONY: trace

dump: $(PROG)  ## disassemble executable sections
	$(ASPREFIX)-objdump -d tests/data/$<
.PHONY: dump

dumpall: $(PROG)  ## disassemble all sections
	$(ASPREFIX)-objdump -D tests/data/$<
.PHONY: dump


### targets that actually build things
$(PROGS): %: tests/data/%.o
	$(ASPREFIX)-ld -melf32lriscv -o tests/data/$@ $<

%.o: %.s
	$(ASPREFIX)-as -march=rv32i $< -o $@


format:  ## beautify rust code
	cargo fmt
.PHONY: format

fmt: format  ## alias for format

clean:  ## remove intermediate object files
	rm -f $(patsubst %, tests/data/%.o, $(PROGS))
.PHONY: clean

binclean: clean  ## remove assembled RISC-V programs
	rm -f $(patsubst %, tests/data/%, $(PROGS))
.PHONY: binclean

realclean: clean binclean  ## remove everything but source code
	rm -rf target
.PHONY: realclean

help: ## show this help
	@echo
	@echo "Specify a command. The choices are:"
	@echo
	@grep -E '^[0-9a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[0;36m%-10s\033[m %s\n", $$1, $$2}'
	@echo
	@echo "Available environment variables:"
	@echo
	@printf "  \033[0;36m%-10s\033[m %s\n" RUST_LOG "sets log level (debug, trace, error)"
	@printf "  \033[0;36m%-10s\033[m %s\n" PROG "set the program to run"
	@echo
	@echo "Available programs:"
	@echo
	@printf "  \033[0;36m%-10s\033[m %s\n" hello "(default) your bog standard \"Hello, World!\" program"
	@printf "  \033[0;36m%-10s\033[m %s\n" fib1 "computes the Fibonacci sequence up to fib(42)"
	@printf "  \033[0;36m%-10s\033[m %s\n" fib2 "computes the Fibonacci sequence to a number of your choice"
	@printf "  \033[0;36m%-10s\033[m %s\n" strlen "computes the length of \"The quick brown fox jumps over the lazy dog.\""
	@echo ""
	@echo "Examples:"
	@echo
	@printf "  \033[0;36m%-22s\033[m %s\n" "make run" "builds and runs 'hello'"
	@printf "  \033[0;36m%-22s\033[m %s\n" "PROG=fib1 make trace" "builds and runs 'fib' with trace logging turned on"
	@printf "  \033[0;36m%-22s\033[m %s\n" "PROG=strlen make dump" "dumps the executable section of \`strlen\`"
.PHONY: help
