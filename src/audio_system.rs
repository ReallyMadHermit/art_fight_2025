use bevy::prelude::*;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::io::Cursor;

#[derive(Resource)]
pub struct MyAudio {
    pub audio_handle: OutputStreamHandle,
    pub music_sink: Sink
} impl MyAudio {
    
    pub fn new(audio_handle: OutputStreamHandle) -> Self {
        let music_sink = Sink::try_new(&audio_handle).unwrap();
        Self {
            audio_handle,
            music_sink
        }
    }
    
    pub fn play_sound(&self, sound_data: &'static[u8]) {
        let sink = Sink::try_new(&self.audio_handle).unwrap();
        let cursor = Cursor::new(sound_data);
        let source = Decoder::new(cursor).unwrap();
        sink.append(source);
        sink.detach()
    }
    
    pub fn play_music(&self, sound_data: &'static[u8]) {
        self.music_sink.stop();
        let cursor = Cursor::new(sound_data);
        let source = Decoder::new(cursor).unwrap().repeat_infinite();
        self.music_sink.append(source);
    }
}
