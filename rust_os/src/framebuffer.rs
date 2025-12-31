use core::{fmt::{self, Write}, ptr};

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar, get_raster, get_raster_width};
use spin::Mutex;
use conquer_once::spin::OnceCell;
use x86_64::instructions::interrupts::without_interrupts;

// ============================================================================
// Constants
// ============================================================================

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;
const BORDER_PADDING: usize = 1;
const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);
const FALLBACK_CHAR: char = '�';
const FONT_WEIGHT: FontWeight = FontWeight::Regular;

// ============================================================================
// Global State
// ============================================================================

static WRITER: OnceCell<Mutex<FrameBufferWriter>> = OnceCell::uninit();
static CELL_SIZE: OnceCell<usize> = OnceCell::uninit();

// ============================================================================
// RGB Color Type
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 160, b: 0 };
    pub const BRIGHT_GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const DARK_GRAY: Self = Self { r: 32, g: 32, b: 32 };
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Initialize the global framebuffer writer
pub fn init_framebuffer_writer(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();
    WRITER.init_once(|| Mutex::new(FrameBufferWriter::new(buffer, info)));
}

pub fn clear_color(color: Rgb) {
    with_framebuffer_writer(|writer| writer.clear_color(color));
}

pub fn reset_cursor() {
    with_framebuffer_writer(|writer| {
        writer.x_pos = BORDER_PADDING;
        writer.y_pos = BORDER_PADDING;
    });
}

/// Set the logical cell size used for grid calculations
pub fn init_cell_size(cell_size: usize) {
    CELL_SIZE.get_or_init(|| cell_size);
}

/// Execute a function with access to the framebuffer writer
pub fn with_framebuffer_writer<R>(f: impl FnOnce(&mut FrameBufferWriter) -> R) -> R {
    without_interrupts(|| {
        let mut writer = WRITER
            .get()
            .expect("FrameBufferWriter has not been initialized")
            .lock();
        f(&mut writer)
    })
}

/// Get the framebuffer pixel dimensions
pub fn framebuffer_size() -> (usize, usize) {
    with_framebuffer_writer(|writer| writer.dimensions())
}

/// Get the grid size based on the configured cell size
pub fn grid_size() -> Option<(usize, usize)> {
    CELL_SIZE.get().map(|cell| {
        with_framebuffer_writer(|writer| writer.grid_dimensions(*cell))
    })
}

/// Get the framebuffer dimensions and grid dimensions for a given cell size
pub fn framebuffer_dimensions(cell_size: usize) -> ((usize, usize), (usize, usize)) {
    with_framebuffer_writer(|writer| {
        (writer.dimensions(), writer.grid_dimensions(cell_size))
    })
}

/// Write a single pixel with the provided RGB color
pub fn put_pixel(x: usize, y: usize, color: Rgb) {
    with_framebuffer_writer(|writer| writer.put_pixel_rgb(x, y, color));
}

/// Draw a filled cell at grid coordinates
pub fn draw_cell(cx: usize, cy: usize, cell_size: usize, color: Rgb) {
    let px = cx * cell_size;
    let py = cy * cell_size;

    for y in py..py + cell_size {
        for x in px..px + cell_size {
            put_pixel(x, y, color);
        }
    }
}

// ============================================================================
// Macros for Printing
// ============================================================================

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    with_framebuffer_writer(|writer| {
        writer
            .write_fmt(args)
            .expect("Writing to framebuffer failed")
    })
}

// ============================================================================
// FrameBufferWriter Implementation
// ============================================================================

pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl FrameBufferWriter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut writer = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        writer.clear();
        writer
    }

    // ------------------------------------------------------------------------
    // Dimension Queries
    // ------------------------------------------------------------------------

    pub fn width(&self) -> usize {
        self.info.width
    }

    pub fn height(&self) -> usize {
        self.info.height
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.info.width, self.info.height)
    }

    pub fn grid_dimensions(&self, cell_size: usize) -> (usize, usize) {
        (self.info.width / cell_size, self.info.height / cell_size)
    }

    // ------------------------------------------------------------------------
    // Text Rendering
    // ------------------------------------------------------------------------

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + CHAR_RASTER_WIDTH;
                
                if new_xpos >= self.width() {
                    self.newline();
                }

                let new_ypos = self.y_pos + CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }

                self.write_rendered_char(get_rasterized_char(c))
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn newline(&mut self) {
        self.y_pos += CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING
    }

    // ------------------------------------------------------------------------
    // Low-Level Pixel Operations
    // ------------------------------------------------------------------------

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
            PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {other:?} not supported in FrameBufferWriter")
            }
        };
        
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }

    pub fn put_pixel_rgb(&mut self, x: usize, y: usize, c: Rgb) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }
        
        let pixel_offset = y * self.info.stride + x;
        let byte_offset = pixel_offset * self.info.bytes_per_pixel;
        
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [c.r, c.g, c.b, 0],
            PixelFormat::Bgr => [c.b, c.g, c.r, 0],
            PixelFormat::U8 => {
                let v = if (c.r as u16 + c.g as u16 + c.b as u16) > 0 { 0xF } else { 0x0 };
                [v, 0, 0, 0]
            }
            other => panic!("pixel format {other:?} not supported"),
        };
        
        let bytes_per_pixel = self.info.bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        
        unsafe {
            core::ptr::read_volatile(&self.framebuffer[byte_offset]);
        }
    }

    // ------------------------------------------------------------------------
    // Graphics Operations
    // ------------------------------------------------------------------------

    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    pub fn clear_color(&mut self, c: Rgb) {
        for y in 0..self.info.height {
            for x in 0..self.info.width {
                self.put_pixel_rgb(x, y, c);
            }
        }
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, c: Rgb) {
        let x_end = (x + w).min(self.info.width);
        let y_end = (y + h).min(self.info.height);
        
        for yy in y..y_end {
            for xx in x..x_end {
                self.put_pixel_rgb(xx, yy, c);
            }
        }
    }

    pub fn draw_cell(&mut self, cx: usize, cy: usize, cell_size: usize, color: Rgb) {
        let px = cx * cell_size;
        let py = cy * cell_size;
        self.fill_rect(px, py, cell_size, cell_size, color);
    }

    pub fn draw_cell_inset(&mut self, cx: usize, cy: usize, cell_size: usize, inset: usize, color: Rgb) {
        let px = cx * cell_size + inset;
        let py = cy * cell_size + inset;
        let size = cell_size.saturating_sub(inset * 2);
        self.fill_rect(px, py, size, size, color);
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_rasterized_char(c: char) -> RasterizedChar {
    fn _get(c: char) -> Option<RasterizedChar> {
        get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
    }

    _get(c).unwrap_or_else(|| {
        _get(FALLBACK_CHAR).expect("Failed to get rasterized version of backup char")
    })
}
