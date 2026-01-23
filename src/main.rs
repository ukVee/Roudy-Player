use ratatui::crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend
};

use std::panic::{
    PanicHookInfo,
    set_hook
};
use std::io::stdout;

use dotenv::dotenv;

mod api;
mod event;
mod types;
mod layout;
mod global_state;
mod helpers;


use crate::event::eloop::event_loop;

fn restore_terminal() -> std::io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn cleanup() -> Box<dyn for<'a, 'b> Fn(&'a PanicHookInfo<'b>) + Send + Sync> {
    Box::new(|error| {
        restore_terminal().expect("Failed to restore terminal.");
        println!("Oh no: \n{}", error);
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    enable_raw_mode()?;
    set_hook(cleanup());
    execute!(stdout(), EnterAlternateScreen)?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;


    let (mut terminal, shutdown_server) = event_loop(terminal).await?;

    restore_terminal()?;
    terminal.show_cursor()?;
    if let Some(shutdown) = shutdown_server {
        let _ = shutdown.send(());
    }
    Ok(())
}