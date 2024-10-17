default: debug

check: hello
	cargo run
.PHONY: check

debug: hello
	RUST_LOG=debug cargo run
.PHONY: debug

trace: hello
	RUST_LOG=trace cargo run
.PHONY: trace

hello: hello.o
	riscv64-elf-ld -melf32lriscv -o $@ $<

hello.o: hello.s
	riscv64-elf-as -march=rv32i $< -o $@

dump: hello
	riscv64-elf-objdump -d $<
.PHONY: dump

clean:
	rm -f *.o
.PHONY: clean

realclean: clean
	rm -f hello
	rm -rf target
.PHONY: realclean
