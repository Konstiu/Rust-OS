use crate::filesystem::{FileType, with_filesystem};
use crate::framebuffer::with_framebuffer_writer;
use crate::task::keyboard::ScanCodeStream;
use crate::{print, println};
use alloc::string::String;
use alloc::vec::Vec;
use futures_util::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

const COMMANDS: &[&str] = &[
    "help", "echo", "cat", "ls", "version", "clear", "snake", "cowsay",
];

pub async fn run() {
    let mut scancodes = ScanCodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    let mut command_buffer = String::new();
    print!("> ");

    while let Some(scancode) = scancodes.next().await {
        if crate::wasm_game::is_game_running() {
            crate::wasm_game::handle_scancode(scancode);
            continue;
        }
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => match character {
                        '\n' => {
                            println!();
                            execute_command(&command_buffer);
                            command_buffer.clear();
                            print!("> ");
                        }
                        '\x08' => {
                            // Backspace
                            if !command_buffer.is_empty() {
                                command_buffer.pop();
                                print!("{}", character); // Move cursor back
                            }
                        }
                        '\t' => {
                            autocomplete(&mut command_buffer);
                        }
                        c => {
                            command_buffer.push(c);
                            print!("{}", c);
                        }
                    },
                    DecodedKey::RawKey(_) => {}
                }
            }
        }
    }
}

fn autocomplete(buffer: &mut String) {
    let args: Vec<&str> = buffer.split_whitespace().collect();
    let is_new_arg = buffer.ends_with(' ');

    if args.is_empty() || (args.len() == 1 && !is_new_arg) {
        let prefix = if args.is_empty() { "" } else { args[0] };

        let matches: Vec<&str> = COMMANDS
            .iter()
            .copied()
            .filter(|c| c.starts_with(prefix))
            .collect();

        if matches.len() == 1 {
            let completion = matches[0];
            let remaining = &completion[prefix.len()..];
            buffer.push_str(remaining);
            buffer.push(' ');
            print!("{} ", remaining);
        } else if matches.len() > 1 {
            println!();
            for m in matches {
                print!("{} ", m);
            }
            println!();
            print!("> {}", buffer);
        }
    } else {
        let last_arg = if is_new_arg { "" } else { args.last().unwrap() };

        let (dir, file_prefix) = if let Some(idx) = last_arg.rfind('/') {
            let d = &last_arg[..idx];
            let f = &last_arg[idx + 1..];
            (if d.is_empty() { "/" } else { d }, f)
        } else {
            ("/", last_arg)
        };

        let matches = with_filesystem(|fs| {
            if let Ok(entries) = fs.read_dir(dir) {
                entries
                    .into_iter()
                    .filter(|e| e.name().starts_with(file_prefix))
                    .map(|e| (String::from(e.name()), e.file_type))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        });

        if let Some(matches) = matches {
            if matches.len() == 1 {
                let (name, ftype) = &matches[0];
                let remaining = &name[file_prefix.len()..];
                buffer.push_str(remaining);
                print!("{}", remaining);
                if *ftype == FileType::Dir {
                    buffer.push('/');
                    print!("/");
                }
            } else if matches.len() > 1 {
                println!();
                for (name, ftype) in matches {
                    print!("{}{}", name, if ftype == FileType::Dir { "/" } else { "" });
                    print!(" ");
                }
                println!();
                print!("> {}", buffer);
            }
        }
    }
}

fn execute_command(command: &str) {
    let parts: alloc::vec::Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "help" => {
            print!("Available commands: ");
            for (i, cmd) in COMMANDS.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", cmd);
            }
            println!();
        }
        "version" => println!("RustOS v0.1.0"),
        "clear" => with_framebuffer_writer(|writer| writer.clear()),
        "snake" => {
            println!("Starting Snake...");
            with_framebuffer_writer(|writer| writer.clear());
            let wasm_bytes = include_bytes!("../wasm/snake.wasm");
            crate::wasm_game::init_wasm_game(wasm_bytes);
        }
        "cowsay" => {
            println!("Starting Cowsay...");
            with_framebuffer_writer(|writer| writer.clear());
            let wasm_bytes = include_bytes!("../wasm/cowsay.wasm");
            crate::wasm_game::init_wasm_game(wasm_bytes);
        }
        "ls" => {
            let path = parts.get(1).copied().unwrap_or("/");
            cmd_ls(path);
        }
        "cat" => {
            if parts.len() < 2 {
                println!("Usage: cat <file>");
            } else {
                cmd_cat(parts[1]);
            }
        }
        s if s.starts_with("echo ") => println!("{}", &s[5..]),
        "" => {}
        cmd => println!(
            "Unknown command: '{}'. Type 'help' for a list of commands.",
            cmd
        ),
    }
}

fn cmd_ls(path: &str) {
    match with_filesystem(|fs| fs.read_dir(path)) {
        Some(Ok(entries)) => {
            if entries.is_empty() {
                println!("(empty)");
                return;
            }
            for entry in entries {
                match entry.file_type {
                    crate::filesystem::FileType::Dir => {
                        println!("{}/", entry.name());
                    }
                    crate::filesystem::FileType::File => {
                        println!("{}  ({} bytes)", entry.name(), entry.size);
                    }
                    _ => {
                        println!("{}", entry.name());
                    }
                }
            }
        }
        Some(Err(e)) => println!("ls: {}: {:?}", path, e),
        None => println!("ls: filesystem not initialized"),
    }
}

fn cmd_cat(path: &str) {
    match with_filesystem(|fs| fs.read_to_string(path)) {
        Some(Ok(content)) => print!("{}", content),
        Some(Err(e)) => println!("cat: {}: {:?}", path, e),
        None => println!("cat: filesystem not initialized"),
    }
}
