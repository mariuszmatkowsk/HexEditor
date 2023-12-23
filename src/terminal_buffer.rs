use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};

pub struct TerminalBuffer {
    cells: Vec<Cell>,
    w: usize,
    h: usize,
}

#[derive(Clone)]
struct Cell {
    ch: char,
    fg: Color,
    bg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::White,
            bg: Color::Black,
        }
    }
}

impl TerminalBuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            cells: vec![Cell::default(); w * h],
            w,
            h,
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn put_cell(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
        let index = y * self.w + x;

        if let Some(cell) = self.cells.get_mut(index) {
            *cell = Cell { ch, fg, bg };
        }
    }

    pub fn put_cells(&mut self, x: usize, y: usize, chs: &str, fg: Color, bg: Color) {
        let start_index = y * self.w + x;
        for (i, ch) in chs.chars().enumerate() {
            if let Some(cell) = self.cells.get_mut(start_index + i) {
                *cell = Cell { ch, fg, bg }
            }
        }
    }

    pub fn flush(&self, qc: &mut impl Write) -> io::Result<()> {
        let mut curr_fg_color = Color::White;
        let mut curr_bg_color = Color::Black;
        qc.queue(Clear(ClearType::All))?;
        qc.queue(SetForegroundColor(curr_fg_color))?;
        qc.queue(SetBackgroundColor(curr_bg_color))?;
        qc.queue(MoveTo(0, 0))?;

        for Cell { ch, fg, bg } in self.cells.iter() {
            if curr_fg_color != *fg {
                curr_fg_color = *fg;
                qc.queue(SetForegroundColor(curr_fg_color))?;
            }

            if curr_bg_color != *bg {
                curr_bg_color = *bg;
                qc.queue(SetBackgroundColor(curr_bg_color))?;
            }

            qc.queue(Print(ch))?;
        }

        qc.flush()?;

        Ok(())
    }
}
