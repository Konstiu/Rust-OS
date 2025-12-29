# Rust OS

Current Stand, 

`cargo build` build the executable.
`cargo install bootimage` to install bootimage tool.
`cargo bootimage` creates a bootable disk image and it tells you where it is.
```qemu-system-x86_64 -drive format=raw,file=target/x86_64-LL-2025/debug/bootimage-LowLevelProgrammingProject.bin the name of the binary file may be different based on the project name. Ours is calles LowLevelProgrammingProject. so the bin is called bootimage-LowLevelProgrammingProject.bin```
This can also be executed via `cargo run` because of the target specification in .cargo/config.toml.





Based on [Writing an OS in Rust](https://os.phil-opp.com/)
