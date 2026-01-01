# Rust OS Workspace

Current Stand,

- `make build` builds the `rust_os` kernel 
- `make run` builds the `rust_os` kernel and boots it in QEMU (via the runner in `.cargo/config.toml`).
- `make test` runs the kernel tests in QEMU (headless).

Crates,

- `rust_os` contains the no_std kernel, following the flow from [Writing an OS in Rust](https://os.phil-opp.com/).
- `qemu_runner` is a host-side helper that turns the kernel into a bootable disk image and launches QEMU for `cargo run`/`cargo test`.
