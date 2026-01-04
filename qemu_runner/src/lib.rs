use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    thread,
    time::{Duration, Instant},
};

use bootloader::DiskImageBuilder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QemuMode {
    Run,
    Test,
}

pub fn run_qemu_with_kernel<P, R>(
    kernel_path: P,
    ramdisk_path: Option<R>,
    mode: QemuMode,
) -> Result<ExitStatus, Box<dyn Error>>
where
    P: AsRef<Path>,
    R: AsRef<Path>,
{
    let image_path = create_bios_disk_image(
        kernel_path.as_ref(),
        ramdisk_path.as_ref().map(|p| p.as_ref()),
    )?;
    run_qemu_with_image(&image_path, mode)
}

fn run_qemu_with_image(image_path: &Path, mode: QemuMode) -> Result<ExitStatus, Box<dyn Error>> {
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args([
        "-drive",
        &format!("format=raw,file={}", image_path.display()),
        "-device",
        "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-serial",
        "stdio",
    ]);
    if mode == QemuMode::Test {
        cmd.arg("-display").arg("none");
    }

    let mut child = cmd.spawn()?;

    match mode {
        QemuMode::Test => wait_with_timeout(&mut child, Duration::from_secs(300)),
        QemuMode::Run => Ok(child.wait()?),
    }
}

fn create_bios_disk_image(
    kernel_path: &Path,
    ramdisk_path: Option<&Path>,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut disk_image_builder = DiskImageBuilder::new(kernel_path.to_path_buf());

    if let Some(ramdisk) = ramdisk_path {
        disk_image_builder.set_ramdisk(ramdisk.to_path_buf());
    }

    let mut image_path = kernel_path.to_path_buf();
    image_path.set_extension("img");

    disk_image_builder.create_bios_image(&image_path)?;
    Ok(image_path)
}

fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<ExitStatus, Box<dyn Error>> {
    let deadline = Instant::now() + timeout;

    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }

        let now = Instant::now();
        if now >= deadline {
            if let Err(e) = child.kill() {
                eprintln!("Failed to kill process after timeout: {e}");
            }

            if let Err(e) = child.wait() {
                eprintln!("Waiting on process after timeout failed: {e}");
            }

            return Err(Box::new(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("qemu timed out after {}s", timeout.as_secs()),
            )));
        }

        let time_remaining = deadline.saturating_duration_since(now);
        thread::sleep(time_remaining.min(Duration::from_millis(100)));
    }
}
