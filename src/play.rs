use rodio::{Decoder, OutputStreamBuilder, Sink};

pub fn stream(stream: tidal::media::Stream) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream_handle = OutputStreamBuilder::open_default_stream()?;
    stream_handle.log_on_drop(false);
    let sink = Sink::connect_new(&stream_handle.mixer());
    let source = Decoder::builder()
        .with_data(stream)
        .with_seekable(true)
        .build()?;
    sink.append(source);

    // let _ = sink.try_seek(Duration::from_secs(100));

    sink.sleep_until_end();

    Ok(())
}
