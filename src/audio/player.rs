use anyhow::Error;
use cpal::{BufferSize, SampleRate, StreamConfig, default_host};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use ringbuf::traits::{*,Producer, Split};



pub fn player(decoded_samples: Vec<f32>, channels: u16, sample_rate: SampleRate) -> anyhow::Result<cpal::Stream, Error> {
    let device = default_host().default_output_device().expect("no output device");
    let stream_config = StreamConfig {channels,sample_rate,buffer_size: BufferSize::Default};

    let ring_buffer = ringbuf::HeapRb::<f32>::new(decoded_samples.len());
    let (mut producer, mut consumer) = ring_buffer.split();

    for sample in decoded_samples {
        let _ = producer.try_push(sample);
    }
    let stream = device.build_output_stream(&stream_config,
    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for sample in data.iter_mut() {
            *sample = consumer.try_pop().unwrap_or(0.0); // silence on underrun
        }
    },
    |err| eprintln!("error: {err}"),
        None,
    )?;
    match stream.play() {
        Ok(_) => {
            Ok(stream)
        }
        Err(e) => {
            Err(e.into())
        }
    }
    
}