PROGS=hello complexMul fac fib strlen
DEFAULT_PROG=hello
PROG?=$(DEFAULT_PROG)

default: help

build: target/debug/rvem  # builds the RISC-V emulator
	cargo build
.PHONY: hello

run: $(PROG)  ## emulate a RISC-V program
	cargo run -- tests/data/$<
.PHONY: hello

debug: $(PROG)  ## run rvem with debug logging enabled
	RUST_LOG=debug cargo run -- tests/data/$<
.PHONY: debug

trace: $(PROG)  ## run rvem with trace logging enabled
	RUST_LOG=trace cargo run -- tests/data/$<
.PHONY: trace

dump: $(PROG)  ## disassemble all sections using rvem
	cargo run -- -D tests/data/$<
.phony: dump

objdump: $(PROG)  ## disassemble executable sections using objdump
	$(ASPREFIX)-objdump -d tests/data/$<
.PHONY: dump

objdump-all: $(PROG)  ## disassemble all sections using objdump
	$(ASPREFIX)-objdump -D tests/data/$<
.PHONY: dump

readelf: $(PROG)  ## display ELF information
	$(ASPREFIX)-readelf -a tests/data/$<
.PHONY: readelf

check: $(PROGS)  ## emulate all programs and test their output
	cargo test
.PHONY: check

$(PROGS):
	make -C tests/data $@

test: check  ## alias for check

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
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[0;36m%-22s\033[m %s\n", $$1, $$2}'
	@echo
	@echo "Available environment variables:"
	@echo
	@printf "  \033[0;36m%-22s\033[m %s\n" RUST_LOG "sets log level (debug, trace, error)"
	@printf "  \033[0;36m%-22s\033[m %s\n" PROG "sets the program to run"
	@echo
	@echo "Available programs:"
	@echo
	@printf "  \033[0;36m%-22s\033[m %s\n" hello "(default) your bog standard \"Hello, World!\" program"
	@printf "  \033[0;36m%-22s\033[m %s\n" complexMul "computes (1 + 3i) * (5 + 4i)"
	@printf "  \033[0;36m%-22s\033[m %s\n" fac "computes 5!"
	@printf "  \033[0;36m%-22s\033[m %s\n" fib "computes the Fibonacci sequence up to fib(42)"
	@printf "  \033[0;36m%-22s\033[m %s\n" strlen "computes the length of \"The quick brown fox jumps over the lazy dog.\""
	@echo ""
	@echo "Examples:"
	@echo
	@printf "  \033[0;36m%-22s\033[m %s\n" "make run" "builds and runs 'hello'"
	@printf "  \033[0;36m%-22s\033[m %s\n" "PROG=fib1 make trace" "builds and runs 'fib1' with trace logging turned on"
	@printf "  \033[0;36m%-22s\033[m %s\n" "PROG=strlen make dump" "dumps the program \`strlen\`"
.PHONY: help
