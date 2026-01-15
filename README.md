
# Rust OS — LL 2025W

A Rust-based experimental operating system developed as part of the
**Low-Level Programming (LL) 2025W** course.

The project builds a minimal **x86_64 kernel**, runs it under **QEMU**,
mounts a **TAR-based RAM disk**, and provides an interactive shell with
**WASM-powered demos**.

---

## Quick start

```bash
# Build the kernel
make build

# Run in QEMU (auto-builds RAM disk)
make run

# Run headless tests in QEMU
make test

# Clean all artifacts
make clean
````

### Requirements

- Nightly Rust toolchain
- QEMU
- `tar` (GNU tar via coreutils)

QEMU execution is handled by the host-side runner in
[`qemu_runner`](qemu_runner).

---

## What’s included

- **`rust_os`**
  A `no_std` x86_64 kernel inspired by *Writing an OS in Rust*.
  Sets up GDT/IDT, paging, heap allocation, interrupts, and async tasks.

- **`qemu_runner`**
  Host-side utility that builds a bootable disk image, wires in the RAM
  disk, and runs QEMU for both `cargo run` and `cargo test`.

- **RAM disk**
  TAR-backed readonly filesystem:

  - Normal boots: [`ramdisk`](ramdisk)
  - Tests: [`ramdisk_test`](ramdisk_test)

- **Shell**
  Interactive shell with commands:
  `help`, `echo`, `cat`, `ls`, `version`, `clear`, `exec`
  Includes tab completion for commands and paths.

- **WASM support**
  The shell can load and execute `.wasm` programs from the RAM disk.
  Host functions expose framebuffer drawing and keyboard input
  (see [`rust_os/src/wasm_game.rs`](rust_os/src/wasm_game.rs)).

---

## Repository layout

- [`rust_os`](rust_os) — kernel crate and entry point
- [`qemu_runner`](qemu_runner) — QEMU runner and disk-image builder
- [`ramdisk`](ramdisk) — files packaged for normal boots
- [`ramdisk_test`](ramdisk_test) — test fixtures for headless runs
- [`Makefile`](Makefile) — build, run, test, and RAM disk packaging

Key components:

- TAR filesystem:
  [`rust_os/src/filesystem`](rust_os/src/filesystem)

- Async executor + keyboard stream:
  [`rust_os/src/task`](rust_os/src/task)

- WASM host integration:
  [`rust_os/src/wasm_game.rs`](rust_os/src/wasm_game.rs)

---

## RAM disk

- Built automatically via `make run` / `make test`
- Mounted as a readonly filesystem at `/` in the guest
- Example contents:

  ```
  /
  ├── hello.txt
  ├── readme.txt
  ├── etc/config.txt
  ├── apps/        # WASM binaries
  └── tmp/
  ```

Add your own `.wasm` files under `ramdisk/apps/`.

---

## Shell usage

- The shell starts automatically after boot
- Commands:

  ```
  help
  version
  clear
  ls <path>
  cat <file>
  echo <text>
  exec <program>.wasm
  ```

- Tab completion works for commands and filesystem paths
- `exec` clears the framebuffer and runs a WASM program
- Press `Esc` to return from WASM execution to the shell

---

## Building and running manually

### 1) Package the RAM disk

```bash
tar --format=ustar -C ramdisk -cf ramdisk.tar .
```

### 2) Run the kernel

```bash
cargo run -p rust_os \
  --target x86_64-unknown-none \
  -- --ramdisk $(pwd)/ramdisk.tar
```

---

## Creating a bootable disk image

The runner can produce a **BIOS-bootable raw disk image** for QEMU
and real hardware testing.

```bash
cargo build -p rust_os --target x86_64-unknown-none
tar --format=ustar -C ramdisk -cf ramdisk.tar .
cargo run -p qemu_runner -- \
  target/x86_64-unknown-none/debug/rust_os \
  --ramdisk $(pwd)/ramdisk.tar
```

Output:

```
target/x86_64-unknown-none/debug/rust_os.img
```

### Booting on real hardware (⚠ destructive)

```bash
sudo dd if=target/x86_64-unknown-none/debug/rust_os.img \
        of=/dev/sdX bs=4M status=progress conv=fsync
```

Boot in **legacy / CSM mode**.
UEFI boot is **not supported**.

---

## Keyboard on real hardware (UEFI vs BIOS)

The kernel reads keyboard input from the **legacy PS/2 controller**
(`i8042`, port `0x60`, IRQ1).

- QEMU emulates this device → works out of the box
- Modern UEFI systems may **not expose PS/2**

### Possible fixes

- Enable **USB Legacy Support**
- Enable **CSM / Legacy Boot**
- Disable Secure Boot if required

Native USB HID input is **not implemented**.

---

## Testing

```bash
make test
```

Runs headless QEMU tests using `isa-debug-exit`.
Test fixtures are packaged from [`ramdisk_test`](ramdisk_test).

---

## Learning Resources

This project closely follows some concepts from the excellent
**Writing an OS in Rust** series by Philipp Oppermann:

- <https://os.phil-opp.com/>

Another repository for the same course project can be found at:

- <https://github.com/rettetdemdativ/rust-os>

---

## Notes

- Initializes GDT/IDT, paging, heap allocators, PIT timer, and interrupts
- Uses the bootloader’s linear framebuffer for graphics output
- VGA text mode remains for reference
- WASM host exposes framebuffer helpers and keyboard input

---

## License

This project is licensed under the **MIT License**.
See the [LICENSE](LICENSE) file for details.
