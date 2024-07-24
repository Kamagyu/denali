use std::{error::Error, io};
use ratatui::{
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
    },
    prelude::*
};

mod ui;
mod parser;

fn main() -> Result<(), Box<dyn Error>> {

    for file in std::fs::read_dir("tmp/")? {
        let file = file?;
        let file_path = file.path();
        std::fs::remove_file(file_path)?;
        
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it    
    let mut app = ui::App::new();
    app.mc_version = "1.20.1".to_string();
    let res = ui::run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}



