use std::{error::Error, sync::mpsc::Receiver};

use tidal::media::{AudioDecoder, Stream};

use super::audio;

pub enum PlayerCtl {
    Play(Stream),
}

// very unsafe and probably bad, but faster than Arc<Mutex<T>> :)
// data does not need to be correct and can race
// I just hope that allocation and deallocation does not get fucked
pub struct UnsafeDecoder {
    decoder: *mut Option<AudioDecoder>,
}
impl UnsafeDecoder {
    pub fn new() -> Self {
        let none = Box::new(None);
        let none = Box::into_raw(none);
        Self { decoder: none }
    }
    pub fn get(&self) -> &mut Option<AudioDecoder> {
        unsafe { &mut *self.decoder }
    }
    pub fn set(&mut self, decoder: Option<AudioDecoder>) {
        unsafe { *self.decoder = decoder }
    }
    pub fn share(&self) -> Self {
        Self {
            decoder: self.decoder,
        }
    }
}
unsafe impl Send for UnsafeDecoder {}

pub fn init(rx: Receiver<PlayerCtl>) -> Result<(), Box<dyn Error>> {
    let mut decoder = UnsafeDecoder::new();
    let _stream = audio::init(decoder.share());

    while let Ok(ctl) = rx.recv() {
        match ctl {
            PlayerCtl::Play(stream) => {
                let d = AudioDecoder::from_stream(stream)?;
                decoder.set(Some(d));
            }
        }
    }

    Ok(())
}
