use std::io::Cursor;

use symphonia::{core::{audio::{SampleBuffer}, codecs::{CODEC_TYPE_NULL, DecoderOptions}, formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint}, default::{get_codecs, get_probe}};


pub struct DecodedStream {
    pub stream_samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}


pub fn decode_stream(stream: Vec<u8>) -> Result<DecodedStream, anyhow::Error> {
    let cursor = Cursor::new(stream);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let mut hint = Hint::new();
    hint.mime_type("audio/mpeg");

    let probed = get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;

    let mut format = probed.format;

    let track = format.tracks().iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .expect("no audio track");
    let track_id = track.id;

    let channels_count = track.codec_params.channels.expect("should have channels").count() as u16;
    let sample_rate = track.codec_params.sample_rate.expect("should have");
    

    let mut decoder = get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;

    let mut sample_buf = None;
    let mut new_vec = Vec::<f32>::new();
    loop {
        match format.next_packet() {
            Ok(packet) if packet.track_id() == track_id => {
                let decoded = decoder.decode(&packet)?;
                let buf = sample_buf.get_or_insert_with(|| {
                    SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec())
                });
                buf.copy_interleaved_ref(decoded);
                for &s in buf.samples() {
                    let _ = new_vec.push(s);
                }
            }
            Err(symphonia::core::errors::Error::IoError(ref e))
            if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
            _ => continue,
        }
    }
    Ok(DecodedStream { 
        stream_samples: new_vec,
        channels: channels_count,
        sample_rate: sample_rate,
    })
}