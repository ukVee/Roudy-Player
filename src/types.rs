use ratatui::crossterm::event::KeyEvent;

pub enum PollEvent {
    Input(KeyEvent),
    Tick,
}