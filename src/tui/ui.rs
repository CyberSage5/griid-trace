use crate::tui::App;
use ratatui::{backend::Backend, Frame, Terminal};

/// Main UI rendering function
pub fn ui<B: Backend>(f: &mut Frame, app: &mut App) {
    app.draw(f);
}

/// Initialize the terminal
pub fn init<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
    terminal.clear()?;
    Ok(())
}

/// Restore the terminal to its original state
pub fn restore<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
    terminal.clear()?;
    Ok(())
}
