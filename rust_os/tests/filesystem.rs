#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{panic::PanicInfo, slice};

use bootloader_api::BootInfo;
use conquer_once::spin::OnceCell;
use rust_os::{
    default_entry_point,
    filesystem::{Error, FileSystem, FileType},
    hlt_loop,
    init_kernel
};

default_entry_point!(main);

static RAMDISK: OnceCell<&'static [u8]> = OnceCell::uninit();

fn main(boot_info: &'static mut BootInfo) -> ! {
    let ramdisk_addr = boot_info
        .ramdisk_addr
        .into_option()
        .expect("Could not get ramdisk address from boot info. Ramdisk may not have been included");

    let ramdisk = unsafe {
        slice::from_raw_parts(
            ramdisk_addr as *const u8,
            boot_info.ramdisk_len as usize
        )
    };

    RAMDISK.init_once(|| ramdisk);

    init_kernel(boot_info);
    test_main();
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}


#[test_case]
fn test_file_read() {
    let mut fs = create_fs();
    let expected = "Test: Hello World!\n".as_bytes().to_vec();
    let actual = fs.read("test.txt").unwrap();
    assert_eq!(expected, actual);
}

#[test_case]
fn test_file_read_to_string() {
    let mut fs = create_fs();
    let expected = "Test: Hello World!\n";
    let actual = fs.read_to_string("test.txt").unwrap();
    assert_eq!(expected, actual);
}

#[test_case]
fn test_read_dir_root_entries() {
    let fs = create_fs();
    let entries = fs.read_dir("/").unwrap();

    let mut saw_dir = false;
    let mut saw_test_txt = false;

    for entry in entries {
        match entry.name() {
            "dir" => {
                assert_eq!(entry.file_type, FileType::Dir);
                saw_dir = true;
            }
            "test.txt" => {
                assert_eq!(entry.file_type, FileType::File);
                saw_test_txt = true;
            }
            other => panic!("Unexpected entry in root dir: {other}"),
        }
    }

    assert!(saw_dir, "Missing dir entry");
    assert!(saw_test_txt, "Missing test.txt entry");
}

#[test_case]
fn test_read_dir_subdir_entries() {
    let mut fs = create_fs();
    let entries = fs.read_dir("dir").unwrap();

    let mut saw_file1 = false;
    let mut saw_file2 = false;

    for entry in entries {
        match entry.name() {
            "file1.txt" => {
                assert_eq!(entry.path.as_str(), "dir/file1.txt");
                assert_eq!(entry.file_type, FileType::File);
                let content = fs.read_to_string(entry.path.as_str()).unwrap();
                assert_eq!(content, "File 1 content\n");
                saw_file1 = true;
            }
            "file2.txt" => {
                assert_eq!(entry.path.as_str(), "dir/file2.txt");
                assert_eq!(entry.file_type, FileType::File);
                let content = fs.read_to_string(entry.path.as_str()).unwrap();
                assert_eq!(content, "File 2 content\n");
                saw_file2 = true;
            }
            other => panic!("Unexpected entry in dir: {other}"),
        }
    }

    assert!(saw_file1, "Missing file1.txt entry");
    assert!(saw_file2, "Missing file2.txt entry");
}

#[test_case]
fn test_read_into_with_offset() {
    let mut fs = create_fs();
    let mut buffer = [0u8; 5];
    let bytes_read = fs.read_into("test.txt", 6, &mut buffer).unwrap();
    assert_eq!(bytes_read, buffer.len());
    assert_eq!(&buffer[..bytes_read], b"Hello");
}

#[test_case]
fn test_invalid_path_traversal() {
    let mut fs = create_fs();
    let err = fs.read("./dir/../dir/file1.txt").unwrap_err();
    assert_eq!(err, Error::InvalidPathTraversal);

    let content = fs.read_to_string("./dir/file1.txt").unwrap();
    assert_eq!(content, "File 1 content\n");
}

#[test_case]
fn test_read_dir_on_file_path_is_empty() {
    let fs = create_fs();
    let entries = fs.read_dir("test.txt").unwrap();
    assert!(entries.is_empty());
}

#[test_case]
fn test_metadata_sizes_root_entries() {
    let fs = create_fs();
    let entries = fs.read_dir("/").unwrap();

    let mut saw_dir = false;
    let mut saw_test_txt = false;

    for entry in entries {
        match entry.name() {
            "dir" => {
                assert_eq!(entry.file_type, FileType::Dir);
                assert_eq!(entry.size, 0);
                saw_dir = true;
            }
            "test.txt" => {
                assert_eq!(entry.file_type, FileType::File);
                assert_eq!(entry.size, "Test: Hello World!\n".len());
                saw_test_txt = true;
            }
            _ => {}
        }
    }

    assert!(saw_dir, "Missing dir entry");
    assert!(saw_test_txt, "Missing test.txt entry");
}

#[test_case]
fn test_file_not_found() {
    let mut fs = create_fs();
    let err = fs.read("missing.txt").unwrap_err();
    assert_eq!(err, Error::NotFound);
}

fn create_fs() -> FileSystem {
    let ramdisk = *RAMDISK.get().unwrap();
    FileSystem::from_tar(ramdisk.into())
        .unwrap_or_else(|e| {
        panic!("Failed to create filesystem: {e}")
    })
}
