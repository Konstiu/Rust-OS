use crate::task::keyboard::ScanCodeStream;
use crate::{print, println};
use alloc::string::String;
use futures_util::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

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

fn execute_command(command: &str) {
    match command.trim() {
        "help" => println!("Available commands: help, echo, version, clear, snake, cowsay"),
        "version" => println!("RustOS v0.1.0"),
        // FIXME: implement rendering for this
        //"clear" => print!("\x1b[2J"),
        // FIXME: allocation does not work?
        "snake" => {
            println!("Starting Snake...");
            let wasm_bytes = include_bytes!("../wasm/snake.wasm");
            crate::wasm_game::init_wasm_game(wasm_bytes);
        }
        "cowsay" => {
            println!("Starting Cowsay...");
            let wasm_bytes = include_bytes!("../wasm/cowsay.wasm");
            crate::wasm_game::init_wasm_game(wasm_bytes);
        }
        s if s.starts_with("echo ") => println!("{}", &s[5..]),
        "" => {}
        cmd => println!(
            "Unknown command: '{}'. Type 'help' for a list of commands.",
            cmd
        ),
    }
}
