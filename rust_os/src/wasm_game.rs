use wasmi::{Caller, Linker, Store, Engine, Module, Func};
use crate::framebuffer::{self, Rgb};
use spin::Mutex;
use conquer_once::spin::OnceCell;
use alloc::vec;

static WASM_GAME: OnceCell<Mutex<WasmGame>> = OnceCell::uninit();

pub struct WasmGame {
    store: Store<()>,
    game_update: Func,
    game_render: Func,
    set_direction: Func,
    handle_key: Func,
}

/// Initialize and start the WASM Snake game
pub fn init_wasm_game(wasm_bytes: &'static [u8]) {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm_bytes).expect("Failed to parse WASM module");

    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    // Register framebuffer host functions
    register_framebuffer_functions(&mut linker);

    // Instantiate the module
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .expect("Failed to instantiate WASM module");

    // Get exported functions
    let game_init = instance
        .get_func(&store, "game_init")
        .expect("game_init not found")
        .typed::<(), ()>(&store)
        .expect("game_init has wrong signature");

    let game_update = instance
        .get_func(&store, "game_update")
        .expect("game_update not found");

    let game_render = instance
        .get_func(&store, "game_render")
        .expect("game_render not found");

    let set_direction = instance
        .get_func(&store, "set_direction")
        .expect("set_direction not found");

    let handle_key = instance
        .get_func(&store, "handle_key")
        .expect("handle_key not found");

    // Initialize the game
    game_init.call(&mut store, ()).expect("game_init failed");

    // Store the game state
    let game = WasmGame {
        store,
        game_update,
        game_render,
        set_direction,
        handle_key,
    };

    WASM_GAME.init_once(|| Mutex::new(game));
}

/// Update the game (call from timer interrupt)
pub fn update_game() {
    if let Some(game_mutex) = WASM_GAME.get() {
        let mut game = game_mutex.lock();
        let update_fn = game
            .game_update
            .typed::<(), ()>(&game.store)
            .expect("game_update has wrong signature");
        update_fn.call(&mut game.store, ()).ok();
    }
}

/// Handle keyboard input - pass raw key code to WASM
pub fn handle_key(key_code: u8) {
    if let Some(game_mutex) = WASM_GAME.get() {
        let mut game = game_mutex.lock();
        let handle_key_fn = game
            .handle_key
            .typed::<i32, ()>(&game.store)
            .expect("handle_key has wrong signature");
        handle_key_fn.call(&mut game.store, key_code as i32).ok();
    }
}

/// Render the game (call from timer interrupt after update)
pub fn render_game() {
    if let Some(game_mutex) = WASM_GAME.get() {
        let mut game = game_mutex.lock();
        let render_fn = game
            .game_render
            .typed::<(), ()>(&game.store)
            .expect("game_render has wrong signature");
        render_fn.call(&mut game.store, ()).ok();
    }
}

/// Handle keyboard input for the game
/// Direction: 0=right, 1=down, 2=left, 3=up
pub fn handle_direction(direction: i32) {
    if let Some(game_mutex) = WASM_GAME.get() {
        let mut game = game_mutex.lock();
        let set_dir_fn = game
            .set_direction
            .typed::<i32, ()>(&game.store)
            .expect("set_direction has wrong signature");
        set_dir_fn.call(&mut game.store, direction).ok();
    }
}

/// Register all framebuffer functions as WASM host functions
fn register_framebuffer_functions<T>(linker: &mut Linker<T>) {
    // put_pixel(x: i32, y: i32, r: i32, g: i32, b: i32)
    linker
        .func_wrap(
            "env",
            "put_pixel",
            |_caller: Caller<T>, x: i32, y: i32, r: i32, g: i32, b: i32| {
                let color = Rgb {
                    r: r as u8,
                    g: g as u8,
                    b: b as u8,
                };
                framebuffer::put_pixel(x as usize, y as usize, color);
            },
        )
        .unwrap();

    // draw_cell(cx: i32, cy: i32, cell_size: i32, r: i32, g: i32, b: i32)
    linker
        .func_wrap(
            "env",
            "draw_cell",
            |_caller: Caller<T>, cx: i32, cy: i32, cell_size: i32, r: i32, g: i32, b: i32| {
                let color = Rgb {
                    r: r as u8,
                    g: g as u8,
                    b: b as u8,
                };
                framebuffer::draw_cell(cx as usize, cy as usize, cell_size as usize, color);
            },
        )
        .unwrap();

    // clear_color(r: i32, g: i32, b: i32)
    linker
        .func_wrap(
            "env",
            "clear_color",
            |_caller: Caller<T>, r: i32, g: i32, b: i32| {
                let color = Rgb {
                    r: r as u8,
                    g: g as u8,
                    b: b as u8,
                };
                framebuffer::clear_color(color);
            },
        )
        .unwrap();

    // get_framebuffer_width() -> i32
    linker
        .func_wrap(
            "env",
            "get_framebuffer_width",
            |_caller: Caller<T>| -> i32 {
                let (width, _) = framebuffer::framebuffer_size();
                width as i32
            },
        )
        .unwrap();

    // get_framebuffer_height() -> i32
    linker
        .func_wrap(
            "env",
            "get_framebuffer_height",
            |_caller: Caller<T>| -> i32 {
                let (_, height) = framebuffer::framebuffer_size();
                height as i32
            },
        )
        .unwrap();

    // get_grid_width() -> i32
    linker
        .func_wrap("env", "get_grid_width", |_caller: Caller<T>| -> i32 {
            framebuffer::grid_size().map(|(w, _)| w as i32).unwrap_or(0)
        })
        .unwrap();

    // get_grid_height() -> i32
    linker
        .func_wrap("env", "get_grid_height", |_caller: Caller<T>| -> i32 {
            framebuffer::grid_size().map(|(_, h)| h as i32).unwrap_or(0)
        })
        .unwrap();

    // init_cell_size(size: i32)
    linker
        .func_wrap("env", "init_cell_size", |_caller: Caller<T>, size: i32| {
            framebuffer::init_cell_size(size as usize);
        })
        .unwrap();

    // reset_cursor()
    linker
        .func_wrap("env", "reset_cursor", |_caller: Caller<T>| {
            framebuffer::reset_cursor();
        })
        .unwrap();

    // println(ptr: i32, len: i32)
    linker
        .func_wrap("env", "println", |caller: Caller<T>, ptr: i32, len: i32| {
            // Get WASM memory
            let memory = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("Failed to get WASM memory");

            // Read the string bytes from WASM memory
            let mut buffer = vec![0u8; len as usize];
            memory
                .read(&caller, ptr as usize, &mut buffer)
                .expect("Failed to read memory");

            // Convert to string and print using your println! macro
            if let Ok(s) = core::str::from_utf8(&buffer) {
                crate::serial_println!("WASM println: ptr={}, len={}, str='{}'", ptr, len, s);
                crate::println!("{}", s);
            }
        })
        .unwrap();
}
