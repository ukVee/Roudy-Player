use std::sync::mpsc::Sender;

use crate::audio::{decoder::decode_stream, player::player};



pub enum AudioCommand {
    Play(Vec<u8>),
    Pause,
    Resume,
    Shutdown
}

pub struct AudioHandler {
    pub audio_messeneger: Sender<AudioCommand>,
}

impl AudioHandler {
    pub fn mount() -> Self {
        let (audio_tx, audio_rx) = std::sync::mpsc::channel::<AudioCommand>();

        std::thread::spawn(move || {
            let mut _audio_stream = None;
            loop {
                if let Ok(event) = audio_rx.recv() {
                    match event {
                        AudioCommand::Shutdown => {
                            break;
                        }
                        AudioCommand::Pause => {

                        }
                        AudioCommand::Resume => {

                        }
                        AudioCommand::Play(bytes) => {
                            match decode_stream(bytes) {
                                Ok(stream) => {
                                    _audio_stream = Some(player(stream.stream_samples, stream.channels as u16, stream.sample_rate ));
                                }
                                Err(e) => {

                                }
                            }

                        }
                    }
                }
            }
        });

        Self {
            audio_messeneger: audio_tx,
        }
    }
}