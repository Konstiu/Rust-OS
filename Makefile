CARGO ?= cargo

.PHONY: build run test

build:
	$(CARGO) build -p rust_os --target x86_64-unknown-none

run:
	$(CARGO) run -p rust_os --target x86_64-unknown-none

test:
	$(CARGO) test -p rust_os --target x86_64-unknown-none
