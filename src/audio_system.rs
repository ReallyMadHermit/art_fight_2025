use bevy::prelude::*;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::io::Cursor;

#[derive(Resource)]
pub struct MyAudio {
    pub sound_sink: Sink,
    pub music_sink: Sink
} impl MyAudio {
    
    pub fn new(handle: OutputStreamHandle) -> Self {
        let sound_sink = Sink::try_new(&handle).unwrap();
        let music_sink = Sink::try_new(&handle).unwrap();
        Self {
            sound_sink,
            music_sink
        }
    }
    
    pub fn play_sound(&self, sound_data: &'static[u8]) {
        let cursor = Cursor::new(sound_data);
        let source = Decoder::new(cursor).unwrap();
        self.sound_sink.append(source);
    }
    
    pub fn play_music(&self, sound_data: &'static[u8]) {
        self.music_sink.stop();
        let cursor = Cursor::new(sound_data);
        let source = Decoder::new(cursor).unwrap().repeat_infinite();
        self.sound_sink.append(source);
    }
}
