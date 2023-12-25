use std::io;

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct ScreenState;

impl ScreenState {
    pub fn enable() -> io::Result<ScreenState> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        execute!(io::stdout(), Hide)?;
        Ok(Self)
    }
}

impl Drop for ScreenState {
    fn drop(&mut self) {
        execute!(io::stdout(), Show).unwrap();
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
    }
}
