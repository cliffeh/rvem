default: check

check: hello
	cargo run
.PHONY: check

hello: hello.o
	riscv64-elf-ld -melf32lriscv -o $@ $<

hello.o: hello.s
	riscv64-elf-as -march=rv32i $< -o $@

clean:
	rm -f *.o
.PHONY: clean

realclean: clean
	rm -f hello
	rm -rf target
.PHONY: realclean
