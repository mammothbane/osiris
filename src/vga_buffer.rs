use core::fmt;
use core::ptr::Unique;
use memory::VGA_BASE;
use memory::VirtualAddr;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScreenChar {
    ascii_char: u8,
    color: ColorCode,
}

impl ScreenChar {
    const BLANK: ScreenChar = ScreenChar {
        ascii_char: b' ',
        color: ColorCode::new(Color::Black, Color::Black),
    };
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color: ColorCode,
    buf: Unique<Buffer>,
}

impl Writer {
    pub fn write(&mut self, b: u8) {
        match b {
            b'\n' => self.newline(),
            b => {
                if self.column_position >= BUFFER_WIDTH {
                    self.newline();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color = self.color;
                self.buffer().chars[row][col].write(ScreenChar {
                    ascii_char: b,
                    color,
                });

                self.column_position += 1;
            }
        }

    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buf.as_mut() }
    }

    fn newline(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let buf = self.buffer();
                let chr = buf.chars[row][col].read();
                buf.chars[row - 1][col].write(chr);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buffer().chars[row][col].write(ScreenChar::BLANK);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.bytes().for_each(|b| self.write(b));
        Ok(())
    }
}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color: ColorCode::new(Color::LightGreen, Color::Black),
    buf: unsafe { Unique::new_unchecked(VGA_BASE as *mut _) },
});

pub unsafe fn update_vga_base(addr: VirtualAddr) {
    WRITER.lock().buf = Unique::new_unchecked(addr as *mut _);
}

macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga_buffer::print(format_args!($($arg)*));
    });
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

pub fn clear_screen() {
    for _ in 0..BUFFER_HEIGHT {
        println!();
    }
}
