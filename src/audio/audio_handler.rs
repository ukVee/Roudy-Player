use std::sync::mpsc::Sender;

use cpal::{StreamConfig, default_host, traits::{DeviceTrait, HostTrait, StreamTrait}};
use m3u8_rs::MediaPlaylist;
use ringbuf::{rb, traits::{Consumer, Producer, Split}};

use crate::audio::{
    decoder::{DecodedStream, decode_stream},
};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32};

pub enum AudioCommand {
    HlsReceived(Vec<u8>),
    Pause,
    Resume,
    Shutdown,
}

pub enum AudioMessage {
    StoreMediaPlaylist((Vec<u8>, MediaPlaylist)),
}

pub struct AudioHandler {
    pub audio_messeneger: Sender<AudioCommand>,
    pub media_playlist: Option<(Vec<u8>, MediaPlaylist)>,
}

impl AudioHandler {
    pub fn mount() -> Self {
        let (audio_tx, audio_rx) = std::sync::mpsc::channel::<AudioCommand>();
        std::thread::spawn(move || {
            let _volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
            let _paused = Arc::new(AtomicBool::new(false));
            let mut _audio_stream = None;
            let mut rb_producer: Option<ringbuf::wrap::caching::Caching<Arc<rb::SharedRb<ringbuf::storage::Heap<f32>>>, true, false>> = None;
            
            loop {
                if let Ok(event) = audio_rx.recv() {
                    match event {
                        AudioCommand::Shutdown => {
                            break;
                        }
                        AudioCommand::Pause => {}
                        AudioCommand::Resume => {}
                        AudioCommand::HlsReceived(bytes) => {
                            let decoded_stream = AudioHandler::decode_segment(&bytes, 0).expect("should decode segment");
                            if let Some(prod) = rb_producer.as_mut() {
                                for sample in &decoded_stream.stream_samples {
                                    let _ = prod.try_push(*sample);
                                }
                            } else {
                                let device = default_host().default_output_device().expect("no output device");
                                let stream_config = StreamConfig {
                                    channels: decoded_stream.channels,
                                    sample_rate: decoded_stream.sample_rate,
                                    buffer_size: cpal::BufferSize::Default
                                };
                                let ring_buffer = ringbuf::HeapRb::<f32>::new(44100*2*240);
                                let (mut producer, mut consumer) = ring_buffer.split();

                                
                                for sample in &decoded_stream.stream_samples {
                                    let _ = producer.try_push(*sample);
                                }
                                rb_producer = Some(producer);

                                let stream = device.build_output_stream(&stream_config, move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                    for sample in data.iter_mut() {
                                        *sample = consumer.try_pop().unwrap_or(0.0);
                                    }
                                }, |err| {
                                    eprintln!("error: {err}")
                                }, None);

                                match stream {
                                    Ok(strm) => {
                                        let _ = strm.play();
                                        _audio_stream = Some(strm);
                                    }
                                    Err(_) => {}
                                }
                                
                            }
                        }
                    }
                }
            }
        });

        Self {
            audio_messeneger: audio_tx,
            media_playlist: None,
        }
    }

    pub fn update(&mut self, msg: AudioMessage) {
        match msg {
            AudioMessage::StoreMediaPlaylist(playlist) => {
                self.media_playlist = Some(playlist);
            }
        }
    }
    
    pub fn decode_segment(segment: &Vec<u8>, mut count: usize) -> Option<DecodedStream> {
        let mut tmp: Option<DecodedStream> = None;
        if let Ok(stream) = decode_stream(segment.to_vec()) {
            tmp = Some(stream);
        } else if count < 5 {
            count += 1;
            AudioHandler::decode_segment(segment, count);
        }
        tmp
    }
}
