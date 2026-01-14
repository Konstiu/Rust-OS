RUST OS - Low-Level Programming 2025W
=====================================
A minimal x86_64 kernel built in Rust, bootable on real hardware (BIOS/legacy) and under QEMU.

QUICK START
===========
make run - Build and boot in QEMU | make test - Run headless tests | make build - Build kernel only

SHELL COMMANDS
==============
help - Show available commands | version - Print OS version | clear - Clear screen | ls <path> - List directory
cat <file> - Read file | echo <text> - Print text | exec <file> - Load and run a .wasm program
Tab completion works for commands and filesystem paths. Press ESC while running a WASM program to return.

CREATING A BOOTABLE USB
=======================
1. cargo build -p rust_os --target x86_64-unknown-none
2. tar --format=ustar -C ramdisk -cf ramdisk.tar .
3. cargo run -p qemu_runner -- target/x86_64-unknown-none/debug/rust_os --ramdisk $(pwd)/ramdisk.tar
4. sudo dd if=target/x86_64-unknown-none/debug/rust_os.img of=/dev/sdX bs=4M status=progress conv=fsync
5. Boot in BIOS/legacy mode (CSM). UEFI not supported.

ARCHITECTURE
============
rust_os/ - Kernel with GDT, IDT, paging, heap, framebuffer, interrupts, async executor, TAR filesystem, WASM.
qemu_runner/ - Host utility to build bootable disk images and launch QEMU.

RAM DISK
========
/readme.txt, /hello.txt - Example files | /etc/config.txt - Configuration | /tmp/, /apps/, /bin/ - Directories

NOTES
=====
Bootloader: bootloader crate v0.11.3 (BIOS/MBR only) | Platform: x86_64, no_std | TAR: USTAR format via tarfs