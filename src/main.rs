mod app;
mod config;
mod event;
mod jira;
mod widgets;

// mod log;

use crate::event::event::Event;
use app::App;
use crossterm::{
    cursor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
// use log::{debug, error, info, log_enabled, Level};
// use serde::{Deserialize, Serialize};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = config::Config::new().unwrap();

    setup_terminal()?;

    // setup panic handler to restore terminal before exiting
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        shutdown_terminal();
        original_hook(panic);
    }));

    let mut app: App = App::new(config.clone()).await?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::event::Events::new(250);

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            if let Err(err) = app.draw(f) {
                shutdown_terminal();
                eprintln!("Error: {err:?}");
                std::process::exit(1);
            }
        })?;
        match events.next()? {
            Event::Input(key) => match app.event(key).await {
                Ok(state) => {
                    if !state.is_consumed()
                        && (key == app.config.key_config.quit || key == app.config.key_config.exit)
                    {
                        break;
                    }
                }
                Err(err) => app.error.set(err.to_string())?,
                // Err(_err) => {}
            },
            Event::Tick => (),
        }
    }

    shutdown_terminal();

    Ok(())
}

fn setup_terminal() -> anyhow::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

fn shutdown_terminal() {
    let leave_screen = io::stdout().execute(LeaveAlternateScreen).map(|_f| ());

    if let Err(e) = leave_screen {
        eprintln!("leave_screen failed:\n{}", e);
    }

    let leave_raw_mode = disable_raw_mode();

    if let Err(e) = leave_raw_mode {
        eprintln!("leave_raw_mode failed:\n{}", e);
    }

    let show_cursor = io::stdout().execute(cursor::Show).map(|_| ());

    if let Err(e) = show_cursor {
        eprintln!("show_cursor failed:\n{}", e);
    }
}
