PROGS=hello complexMul fac fib strlen  # helloc

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

helloc:
	touch helloc

$(PROGS): %: %.o
	$(ASPREFIX)-ld -melf32lriscv -o $@ $<

%.o: %.s
	$(ASPREFIX)-as -march=rv32im $< -o $@
