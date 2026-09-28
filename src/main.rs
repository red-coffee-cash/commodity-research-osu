mod app;
mod help_content;
mod methods;
mod problem;
mod session;
mod soroban;
mod stats;
mod ui;

use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use app::App;

fn main() -> Result<()> {
    let mut app = App::new();
    // ratatui::init also installs a panic hook that restores the terminal.
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.quit {
        terminal.draw(|f| ui::draw(f, app, Instant::now()))?;
        // Short poll so countdowns and flashes update smoothly.
        if event::poll(Duration::from_millis(30))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.on_key(key, Instant::now());
        }
        app.tick(Instant::now());
    }
    Ok(())
}
