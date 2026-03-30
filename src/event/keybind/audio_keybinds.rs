use std::sync::{Arc, atomic::{AtomicBool, AtomicU32, Ordering}};

use ratatui::crossterm::event::{KeyCode, KeyEvent};



pub fn listen_for_audio_keybinds(key: KeyEvent, paused: Arc<AtomicBool>, volume: Arc<AtomicU32>) {
    let volume_level = f32::from_bits(volume.load(Ordering::Relaxed));
    if key.code == KeyCode::Char(' ') {
        if paused.load(Ordering::Relaxed) {
            paused.store(false, Ordering::Relaxed);
        }else {
            paused.store(true, Ordering::Relaxed);
        }
    }else if key.code == KeyCode::Char('-') {
        let new_vol = volume_level - 0.05;
        if new_vol >= 0.0 {
            volume.store(new_vol.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        }
    }else if key.code == KeyCode::Char('=') {
        let new_vol = volume_level + 0.10;
        if new_vol  <= 1.0 {
            volume.store(new_vol.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        }
    }
}