use ratatui::crossterm::event::{Event, poll};
use tokio::{
    sync::mpsc::Receiver,
    time::Duration,
};
use crate::types::PollEvent;

pub fn setup_event_polling() -> Receiver<PollEvent> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let tick_rate = Duration::from_millis(100);

    
    tokio::spawn(async move {
        loop {
            if tx.is_closed() {
                break;
            }
            if poll(tick_rate).unwrap() {
                if let Event::Key(key) = ratatui::crossterm::event::read().unwrap() {
                    if let Err(error) = tx.send(PollEvent::Input(key)).await {
                        println!("Send Input: \n {error}");
                    }
                }
            } else if !tx.is_closed() {
                if let Err(error) = tx.send(PollEvent::Tick).await {
                    println!("Send Tick: \n {error}");
                }
            }
        }
    });
    rx
}
