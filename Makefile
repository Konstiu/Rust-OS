CARGO ?= cargo

.PHONY: run test
run:
	$(CARGO) run -p rust_os --target x86_64-unknown-none

test:
	$(CARGO) test -p rust_os --target x86_64-unknown-none
