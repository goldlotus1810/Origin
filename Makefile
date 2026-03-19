# HomeOS — origin.olang build system
# Usage:
#   make              — build origin.olang
#   make test         — run all Rust tests
#   make clean        — remove build artifacts
#   make vm           — assemble + link VM only

AS       = as
LD       = ld
CARGO    = cargo

VM_SRC   = vm/x86_64/vm_x86_64.S
VM_OBJ   = vm/x86_64/vm_x86_64.o
VM_BIN   = vm/x86_64/vm_x86_64
STDLIB   = stdlib
KNOWLEDGE = origin.olang
OUTPUT   = origin_new.olang

.PHONY: all vm build test clean clippy

all: build

# Assemble VM
$(VM_OBJ): $(VM_SRC)
	$(AS) --64 -o $@ $<

# Link VM (static, no libc)
$(VM_BIN): $(VM_OBJ)
	$(LD) -static -nostdlib -o $@ $<

vm: $(VM_BIN)

# Build origin.olang
build: $(VM_BIN)
	$(CARGO) run -p builder -- \
		--vm $(VM_BIN) --wrap \
		--stdlib $(STDLIB) \
		--knowledge $(KNOWLEDGE) \
		--codegen \
		-o $(OUTPUT)
	chmod +x $(OUTPUT)
	@echo "Built: $(OUTPUT) ($$(stat -c%s $(OUTPUT)) bytes)"

# Run tests
test:
	$(CARGO) test --workspace

# Clippy
clippy:
	$(CARGO) clippy --workspace

# Clean
clean:
	$(CARGO) clean
	rm -f $(VM_OBJ) $(VM_BIN) $(OUTPUT)
