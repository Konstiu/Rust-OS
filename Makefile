CARGO ?= cargo
RAMDISK_DIR ?= ramdisk
RAMDISK_TAR ?= ramdisk.tar

.PHONY: build run test ramdisk

build:
	$(CARGO) build -p rust_os --target x86_64-unknown-none

run: ramdisk
	$(CARGO) run -p rust_os --target x86_64-unknown-none -- --ramdisk $(RAMDISK_TAR)

test:
	$(CARGO) test -p rust_os --target x86_64-unknown-none

ramdisk:
	tar --format=ustar -C $(RAMDISK_DIR) -cf $(RAMDISK_TAR) .
