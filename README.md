Rust OS workspace for the LL 2025W course. It builds a minimal x86_64 kernel, runs it under QEMU, mounts a TAR-based RAM disk, and offers a small shell with WASM-powered demos.

## Quick start

- Build: `make build`
- Run in QEMU: `make run` (auto-builds the RAM disk and passes it to the runner)
- Test in QEMU (headless): `make test`
- Clean artifacts: `make clean`

Requirements: nightly Rust toolchain, QEMU, and a `tar` tool (GNU tar via coreutils). The QEMU invocation is handled by the host-side runner in [qemu_runner](qemu_runner).

## What’s included

- `rust_os`: no_std kernel (x86_64) inspired by the “Writing an OS in Rust” series; sets up GDT/IDT, paging, heap allocator, and interrupts.
- `qemu_runner`: host utility that wraps QEMU, builds a bootable image, and wires in the RAM disk for both `cargo run` and `cargo test`.
- RAM disk: simple TAR filesystem packaged from [ramdisk](ramdisk) for normal runs and [ramdisk_test](ramdisk_test) for tests.
- Shell: interactive prompt with basic commands (`help`, `echo`, `cat`, `ls`, `version`, `clear`, `exec`) plus tab completion for commands and paths.
- WASM support: the shell can load and run `.wasm` programs from the RAM disk; host functions expose framebuffer drawing and keyboard input (see [rust_os/src/wasm_game.rs](rust_os/src/wasm_game.rs#L1-L210)).

## Repository layout

- [rust_os](rust_os): kernel crate and entrypoint (see [rust_os/src/main.rs](rust_os/src/main.rs#L1-L60))
- [qemu_runner](qemu_runner): runner binary invoked via `cargo run -p rust_os`/`cargo test -p rust_os`
- [ramdisk](ramdisk): files packed for normal boots; contents land at `/` in the guest
- [ramdisk_test](ramdisk_test): fixtures for headless tests
- [Makefile](Makefile): shortcuts for build/run/test, including TAR packaging
- [rust_os/src/filesystem](rust_os/src/filesystem/mod.rs#L1-L80): TAR-backed readonly FS with canonical path handling
- [rust_os/src/task](rust_os/src/task/executor.rs#L1-L120): minimal async task executor and keyboard event stream
- [rust_os/src/wasm_game.rs](rust_os/src/wasm_game.rs#L1-L210): WASM host integration (framebuffer + keyboard bindings)

## RAM disk

- Built via `make ramdisk` (implicit for run) or `make ramdisk_test` (implicit for test).
- The runner passes the TAR blob to the kernel; the kernel mounts it as a readonly filesystem.
- Example contents (see [ramdisk](ramdisk)):
 	- `hello.txt`, `readme.txt`
 	- `etc/config.txt`
 	- `apps/` for WASM binaries (add your `.wasm` files here)
 	- `tmp/` scratch files

## Shell usage

- Prompt starts after boot; commands: `help`, `version`, `clear`, `ls <path>`, `cat <file>`, `echo <text>`, `exec <path>.wasm`.
- Tab completion works for both commands and filesystem paths; directories complete with a trailing `/`.
- `exec` clears the framebuffer and starts a WASM program from the RAM disk; `Esc` returns to the shell.

## Building and running manually

If you prefer explicit commands instead of the Makefile:

1) Package the RAM disk

```bash
tar --format=ustar -C ramdisk -cf ramdisk.tar .
```

1) Run the kernel with the runner

```bash
cargo run -p rust_os --target x86_64-unknown-none -- --ramdisk $(pwd)/ramdisk.tar
```

## Creating a bootable disk image

The runner builds a BIOS-bootable raw disk image (for QEMU testing and real hardware).

```bash
cargo build -p rust_os --target x86_64-unknown-none
tar --format=ustar -C ramdisk -cf ramdisk.tar .
cargo run -p qemu_runner -- target/x86_64-unknown-none/debug/rust_os --ramdisk $(pwd)/ramdisk.tar
# Produces target/x86_64-unknown-none/debug/rust_os.img (BIOS bootable)
```

To boot on real hardware, write the `.img` to a USB drive (double-check the device path, this is destructive):

```bash
sudo dd if=target/x86_64-unknown-none/debug/rust_os.img of=/dev/sdX bs=4M status=progress conv=fsync
```

Then boot the machine in **legacy/CSM mode** from that drive. The image is BIOS-only; UEFI boot is not currently supported (UEFI firmware does not reliably expose the PS/2 keyboard controller for legacy I/O access).

## Keyboard on real hardware (UEFI vs BIOS)

This kernel reads keyboard scancodes from the legacy PS/2 controller (`i8042`, port `0x60`) via IRQ1. QEMU emulates this device, so input works out of the box. On modern laptops booted via UEFI, the firmware may not expose a legacy PS/2 controller, so the shell appears but the keyboard is unresponsive.

Options to fix:

- Enable firmware settings that provide legacy input:
	- Turn on “USB Legacy Support” (routes USB keyboard through SMM to PS/2).
	- Enable “CSM”/“Legacy Boot” mode; disable Secure Boot if required.
- Use the BIOS/legacy image instead of UEFI (see “Creating a bootable disk image”). BIOS typically provides the PS/2 controller path used by this kernel.

Note: Native USB HID input is not implemented in this kernel. If your device lacks PS/2 emulation under UEFI, keyboard input will not work without additional drivers.

## Testing

- `make test` builds the kernel, packages [ramdisk_test](ramdisk_test) into `ramdisk_test.tar`, and runs the headless QEMU test binary. Success is signaled via `isa-debug-exit`.

## Notes

- The kernel sets up GDT/IDT, paging, a bump/linked-list heap allocator, timer + keyboard interrupts, and an async executor. See [rust_os/src/lib.rs](rust_os/src/lib.rs#L1-L120) for the initialization flow.
- Framebuffer text/graphics output uses the bootloader’s linear framebuffer; VGA text mode is still present in [rust_os/src/vga_buffer.rs](rust_os/src/vga_buffer.rs) for reference.
- WASM host functions expose framebuffer drawing, clearing, and grid helpers; keyboard input is forwarded to WASM and the shell.

Based on and adapted from the “Writing an OS in Rust” series.
