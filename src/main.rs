mod screen_state;
mod terminal_buffer;

use screen_state::ScreenState;
use terminal_buffer::TerminalBuffer;

use std::{
    fs::File,
    io::{self, Read},
    result,
    time::Duration,
};

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers},
    style::Color,
    terminal,
};

type Result<T> = result::Result<T, ()>;

const BYTES_PER_LINE: usize = 16;
const ADDRESS_OFFSET: u32 = 0x10;

fn print_usage() {
    println!("Usage: hex_editor <file_path>");
}

fn parse_file_path() -> Option<String> {
    let mut args = std::env::args();
    let _ = args.next();

    return args.next();
}

#[derive(Default)]
struct HexViewLine {
    offset: String,
    bytes: Vec<u8>,
}

impl HexViewLine {
    fn new(offset: String) -> Self {
        Self {
            offset,
            bytes: Vec::default(),
        }
    }

    fn put_data(&mut self, byte_data: u8) {
        self.bytes.push(byte_data);
    }
}

struct Cursor {
    x: usize,
    y: usize,
    is_visible: bool,
    is_left_nibble: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {x: 0, y: 0, is_visible: false, is_left_nibble: true}
    }
}

struct HexView {
    lines: Vec<HexViewLine>,
    cursor: Cursor,
}

impl HexView {
    fn new(data: &[u8]) -> Self {
        fn put_line(lines: &mut Vec<HexViewLine>, offset: &u32, data: &[u8]) {
            let mut hex_view_line = HexViewLine::new(format!("{offset:08X}"));

            for data_byte in data.iter() {
                hex_view_line.put_data(*data_byte);
            }

            lines.push(hex_view_line);
        }

        let mut hex_editor_lines = Vec::new();

        let mut line_bytes: Vec<u8> = Vec::with_capacity(BYTES_PER_LINE);
        let mut current_offset = 0x00000000;
        for (i, b) in data.iter().enumerate() {
            if i > 0 && i % BYTES_PER_LINE == 0 {
                put_line(&mut hex_editor_lines, &current_offset, &line_bytes);
                line_bytes.clear();
                current_offset += ADDRESS_OFFSET;
            }
            line_bytes.push(*b);
        }

        if !line_bytes.is_empty() {
            put_line(&mut hex_editor_lines, &current_offset, &line_bytes);
        }

        Self {
            lines: hex_editor_lines,
            cursor: Cursor::default(),
        }
    }

    fn move_cursor_left(&mut self) {
        if !self.cursor.is_visible {
            self.cursor.is_visible = true;
            return;
        }

        if !self.cursor.is_left_nibble {
            self.cursor.is_left_nibble = true;
        } else {
            if self.cursor.x == 0 && self.cursor.is_left_nibble {
                return;
            }

            self.cursor.is_left_nibble = false;
            if let Some(_) = self.cursor.x.checked_sub(1) {
                self.cursor.x -= 1;
            }
        }
    }

    fn move_cursor_right(&mut self) {
        if !self.cursor.is_visible {
            self.cursor.is_visible = true;
            return;
        }

        if self.cursor.is_left_nibble {
            self.cursor.is_left_nibble = false;
        } else {
            if self.cursor.x == (BYTES_PER_LINE - 1) && !self.cursor.is_left_nibble {
                return;
            }

            self.cursor.is_left_nibble = true;
            self.cursor.x = std::cmp::min(self.cursor.x + 1, BYTES_PER_LINE-1);
        }

    }

    fn move_cursor_up(&mut self) {
        if !self.cursor.is_visible {
            self.cursor.is_visible = true;
            return;
        }

        if let Some(_) = self.cursor.y.checked_sub(1) {
            self.cursor.y -= 1;
        }
    }

    fn move_cursor_down(&mut self) {
        if !self.cursor.is_visible {
            self.cursor.is_visible = true;
            return;
        }

        self.cursor.y = std::cmp::min(self.cursor.y + 1, self.lines.len() - 1);
    }

    fn get_lines(&self) -> &Vec<HexViewLine> {
        &self.lines
    }

    fn get_data_as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for line in self.lines.iter() {
            for byte_data in line.bytes.iter() {
                bytes.push(*byte_data);
            }
        }
        bytes
    }
}

fn render_hex_editor(buffer: &mut TerminalBuffer, hex_editor: &HexView) {
    for (y, hex_editor_line) in hex_editor.get_lines().iter().enumerate() {
        buffer.put_cells(
            0,
            y,
            &format!("{offset}:", offset = hex_editor_line.offset),
            Color::White,
            Color::Black,
        );

        let start_hex = 11;
        for (x, byte_data) in hex_editor_line.bytes.iter().enumerate() {
            let mut left_nibble_fg = Color::White;
            let mut left_nibble_bg = Color::Black;

            let mut right_nibble_fg = Color::White;
            let mut right_nibble_bg = Color::Black;

            if hex_editor.cursor.is_visible {
                if hex_editor.cursor.y == y && hex_editor.cursor.x == x{
                    if hex_editor.cursor.is_left_nibble {
                        left_nibble_fg = Color::Black;
                        left_nibble_bg = Color::White;
                    } else {
                        right_nibble_fg = Color::Black;
                        right_nibble_bg = Color::White;
                    }
                }
            }
            buffer.put_cells(
                start_hex + x * 3,
                y,
                &format!("{value:1X}", value = (byte_data >> 4) & 0xf),
                left_nibble_fg,
                left_nibble_bg,
            );
            buffer.put_cells(
                start_hex + 1 + x * 3,
                y,
                &format!("{value:1X}", value = byte_data & 0xf),
                right_nibble_fg,
                right_nibble_bg,
            );
        }

        let start_asci = 11 + 3 * BYTES_PER_LINE - 1 + 2;
        for (x, byte_data) in hex_editor_line.bytes.iter().enumerate() {
            if { '!'..'~' }.contains(&(*byte_data as char)) {
                buffer.put_cell(start_asci + x, y, *byte_data as char, Color::White, Color::Black);
            } else {
                buffer.put_cell(start_asci + x, y, '.', Color::White, Color::Black);
            }
        }
    }
}

fn main() -> Result<()> {
    let mut stdout = io::stdout();

    let file_path = match parse_file_path() {
        Some(file_path) => file_path,
        None => {
            print_usage();
            return Err(());
        }
    };

    let mut file = File::open(file_path.clone()).map_err(|err| {
        eprintln!("Could not open file: {file_path}: {err}");
    })?;

    let mut data = Vec::new();

    file.read_to_end(&mut data).map_err(|err| {
        eprintln!("Could not read file into buffer: {err}");
    })?;

    let _screen_state = ScreenState::enable().map_err(|err| {
        eprintln!("Could not enter screen state: {err}");
    })?;

    let (width, height) = terminal::size().map_err(|err| {
        eprintln!("Culd not get terminal size: {err}");
    })?;

    let mut buffer = TerminalBuffer::new(width.into(), height.into());

    let mut hex_editor = HexView::new(&data);

    render_hex_editor(&mut buffer, &hex_editor);

    buffer.flush(&mut stdout).map_err(|err| {
        eprintln!("Could not flush buffer: {err}");
    })?;

    let mut quit = false;

    while !quit {
        if poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char(key) if key_event.modifiers == KeyModifiers::CONTROL => {
                                match key {
                                    'c' => quit = true,
                                    'h' => {
                                        hex_editor.move_cursor_left();
                                    }
                                    'l' => {
                                        hex_editor.move_cursor_right();
                                    }
                                    'j' => {
                                        hex_editor.move_cursor_down();
                                    }
                                    'k' => {
                                        hex_editor.move_cursor_up();
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Char(_key) => {}
                            KeyCode::Enter => {}
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        render_hex_editor(&mut buffer, &hex_editor);

        buffer.flush(&mut stdout).map_err(|err| {
            eprintln!("Could not flush buffer: {err}");
        })?;
    }

    Ok(())
}
