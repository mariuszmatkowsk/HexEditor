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

struct ByteNibble {
    left: u8,
    right: u8,
}

impl ByteNibble {
    fn new(data: u8) -> Self {
        Self {
            left: (data >> 4) & 0x0F,
            right: data & 0x0F,
        }
    }

    fn as_byte(&self) -> u8 {
        self.left << 4 | self.right
    }
}

#[derive(Default)]
struct HexEditorLine {
    offset: String,
    hex_data: Vec<ByteNibble>,
    asci_data: Vec<u8>,
}

impl HexEditorLine {
    fn new(offset: String) -> Self {
        Self {
            offset,
            hex_data: Vec::default(),
            asci_data: Vec::default(),
        }
    }

    fn put_data(&mut self, byte_data: u8) {
        self.hex_data.push(ByteNibble::new(byte_data));
        self.asci_data.push(byte_data);
    }
}

struct HexEditor {
    lines: Vec<HexEditorLine>,
}

impl HexEditor {
    fn new(data: &[u8]) -> Self {
        fn put_line(lines: &mut Vec<HexEditorLine>, offset: &u32, data: &[u8]) {
            let mut hex_editor_line = HexEditorLine::new(format!("{offset:08X}"));

            for data_byte in data.iter() {
                hex_editor_line.put_data(*data_byte);
            }

            lines.push(hex_editor_line);
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
        }
    }

    fn get_lines(&self) -> &Vec<HexEditorLine> {
        &self.lines
    }

    fn get_data_as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for line in self.lines.iter() {
            for byte_nibble in line.hex_data.iter() {
                bytes.push(byte_nibble.as_byte());
            }
        }
        bytes
    }
}

fn render_hex_editor(buffer: &mut TerminalBuffer, hex_editor: &HexEditor) {
    for (y, hex_editor_line) in hex_editor.get_lines().iter().enumerate() {
        assert!(
            hex_editor_line.hex_data.len() == hex_editor_line.asci_data.len(),
            "Data is not alignment"
        );
        buffer.put_cells(
            0,
            y,
            &format!("{offset}:", offset = hex_editor_line.offset),
            Color::White,
            Color::Black,
        );

        let start_hex = 11;
        for (x, byte_nibble) in hex_editor_line.hex_data.iter().enumerate() {
            buffer.put_cells(
                start_hex + x * 3,
                y,
                &format!("{value:1X}", value = byte_nibble.left),
                Color::White,
                Color::Black,
            );
            buffer.put_cells(
                start_hex + 1 + x * 3,
                y,
                &format!("{value:1X}", value = byte_nibble.right),
                Color::White,
                Color::Black,
            );
        }

        let start_asci = 11 + 3 * BYTES_PER_LINE - 1 + 2;
        for (x, asci) in hex_editor_line.asci_data.iter().enumerate() {
            if { '!'..'~' }.contains(&(*asci as char)) {
                buffer.put_cell(start_asci + x, y, *asci as char, Color::White, Color::Black);
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

    let hex_editor = HexEditor::new(&data);

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
                                        todo!("Handle left move");
                                    }
                                    'l' => {
                                        todo!("Handle right move")
                                    }
                                    'j' => {
                                        todo!("Handle down move")
                                    }
                                    'k' => {
                                        todo!("Handle up move")
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
        // buffer.flush(&mut stdout).map_err(|err| {
        //     eprintln!("Could not flush buffer: {err}");
        // })?;
    }

    Ok(())
}
