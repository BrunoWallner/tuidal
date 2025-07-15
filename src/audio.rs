use cpal::{
    Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::player::UnsafeDecoder;

pub fn init(decoder: UnsafeDecoder) -> Option<Stream> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = device.default_output_config().ok()?;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut d = decoder.get();
                if let Some(d) = &mut d {
                    let read = d.read(data);
                    if read < data.len() {
                        data[read..].fill(0.0);
                    }
                } else {
                    data.fill(0.0);
                }
            },
            err_fn,
            None,
        )
        .ok()?;
    stream.play().ok()?;

    Some(stream)
}
