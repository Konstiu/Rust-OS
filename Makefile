CARGO ?= cargo
RAMDISK_DIR ?= ramdisk
RAMDISK_TAR ?= ramdisk.tar
RAMDISK_TEST_DIR ?= ramdisk_test
RAMDISK_TEST_TAR ?= ramdisk_test.tar

.PHONY: build run test ramdisk ramdisk_test

build:
	$(CARGO) build -p rust_os --target x86_64-unknown-none

run: ramdisk
	$(CARGO) run -p rust_os --target x86_64-unknown-none -- --ramdisk $(abspath $(RAMDISK_TAR))

test: ramdisk_test
	$(CARGO) test -p rust_os --target x86_64-unknown-none -- --ramdisk $(abspath $(RAMDISK_TEST_TAR))

ramdisk:
	tar --format=ustar -C $(RAMDISK_DIR) -cf $(RAMDISK_TAR) .

ramdisk_test:
	tar --format=ustar -C $(RAMDISK_TEST_DIR) -cf $(RAMDISK_TEST_TAR) .
