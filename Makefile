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

.PHONY: all vm build test intg clean clippy eval smoke smoke-binary verify demo check-all

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

# Integration tests only
intg:
	$(CARGO) test -p intg -- --show-output

# Clippy
clippy:
	$(CARGO) clippy --workspace

# Eval mode — pipe input to server
eval:
	@$(CARGO) run -p server -- --eval

# Quick smoke test
smoke:
	@echo 'hello' | $(CARGO) run -p server -- --eval

# Smoke test: native binary boots and shows prompt
smoke-binary: build
	@echo "Testing origin_new.olang boots..." && \
	OUTPUT=$$(echo '' | ./$(OUTPUT) 2>&1) && \
	echo "$$OUTPUT" | grep -q "origin.olang" && \
	echo "PASS: origin_new.olang boots OK" || \
	(echo "FAIL: origin_new.olang did not boot"; exit 1)

# E2E demo — 10 scenarios (requires tools/demo/scenarios.sh)
demo:
	@bash tools/demo/scenarios.sh

# E2E automated verification (Rust tests)
verify:
	@$(CARGO) test -p intg --test t16_e2e_demo -- --show-output

# Full check: unit + integration + E2E + binary boot
check-all: test intg verify smoke-binary
	@echo "ALL CHECKS PASSED"

# Clean
clean:
	$(CARGO) clean
	rm -f $(VM_OBJ) $(VM_BIN) $(OUTPUT)
