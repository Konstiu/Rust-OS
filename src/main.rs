fn main() {

    let bios_path = env!("BIOS_PATH");

    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    cmd.args(
        [
            "-drive",
            &format!("format=raw,file={bios_path}"),
            "-device",
            "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-serial",
            "stdio",
        ]
    );

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
