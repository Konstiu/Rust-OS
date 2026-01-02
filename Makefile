CARGO ?= cargo

.PHONY: build run test clean

build:
	$(CARGO) build -p rust_os --target x86_64-unknown-none

run:
	$(CARGO) run -p rust_os --target x86_64-unknown-none

test:
	$(CARGO) test -p rust_os --target x86_64-unknown-none

clean:
	$(CARGO) clean -p rust_os
