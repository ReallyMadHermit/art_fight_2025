use bevy::prelude::*;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::io::Cursor;

#[macro_export]
macro_rules! event_exists {
    ($event:ty) => {
        |reader: bevy::ecs::event::EventReader<$event>| !reader.is_empty()
    };
}

pub struct MaterialWizard {
    handles: Vec<Handle<StandardMaterial>>
} impl MaterialWizard {

    pub fn new(
        materials: &mut ResMut<Assets<StandardMaterial>>, reference_material: StandardMaterial,
        saturation: f32, lightness: f32, alpha: f32, color_count: usize, emissive: f32, unlit: bool
    ) -> Self {
        let hues = Self::generate_normal_hue_vec(color_count);
        let mut standard_materials: Vec<StandardMaterial> = Vec::with_capacity(color_count);
        for hue in hues {
            let mut mat = reference_material.clone();
            mat.base_color = Color::hsla(hue * 360.0, saturation, lightness, alpha);
            standard_materials.push(mat);
        };
        if emissive > 0.0 {
            for mat in &mut standard_materials {
                mat.emissive = mat.base_color.to_linear();
                mat.emissive_exposure_weight = emissive;
            };
        };
        if alpha < 1.0 {
            for mat in &mut standard_materials {
                mat.alpha_mode = AlphaMode::Blend;
            };
        };
        if unlit {
            for mat in &mut standard_materials {
                mat.unlit = true;
            };
        };
        let mut handles: Vec<Handle<StandardMaterial>> = Vec::with_capacity(color_count);
        for mat in standard_materials {
            handles.push(materials.add(mat))
        };
        Self {
            handles
        }
    }

    pub fn basic(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        saturation: f32, lightness: f32, alpha: f32, color_count: usize, unlit: bool
    ) -> Self {
        let basic_material = StandardMaterial{
            base_color: Color::BLACK,
            ..default()
        };
        Self::new(
            materials, basic_material,
            saturation, lightness, alpha, color_count, 0.0, unlit
        )
    }

    pub fn get_index(&self, index: usize) -> Handle<StandardMaterial> {
        self.handles[index].clone()
    }

    pub fn generate_normal_hue_vec(color_count: usize) -> Vec<f32> {
        let mut hue_vec: Vec<f32> = Vec::with_capacity(color_count);
        hue_vec.push(0.0);
        let mut step_size = 1.0f32;
        let mut i = 1usize;
        while i < color_count {
            step_size *= 0.5;
            for hue_index in 0..hue_vec.len() {
                hue_vec.push(hue_vec[hue_index] + step_size);
                i += 1;
            };
        };
        hue_vec
    }

}

#[derive(Resource)]
pub struct AudioSystem {
    pub audio_handle: OutputStreamHandle,
    pub music_sink: Sink
} impl AudioSystem {

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

pub struct RectChecks;
impl RectChecks {

    pub fn get_radi(corner_a: Vec2, corner_b: Vec2, radius_mod: f32) -> Vec2 {
        let mut radius = (corner_a - corner_b).abs() / 2.0;
        radius += radius_mod;
        radius.max(Vec2::ZERO)
    }

    pub fn get_rect_center(corner_a: Vec2, corner_b: Vec2) -> Vec2 {
        let center = (corner_a + corner_b) / 2.0;
        center
    }

    pub fn is_inside_y_first(xy_radius: Vec2, center: Vec2, point: Vec2) -> bool {
        let d = (point - center).abs();
        d.y <= xy_radius.y && d.x <= xy_radius.x
    }

    pub fn is_inside_x_first(xy_radius: Vec2, center: Vec2, point: Vec2) -> bool {
        let d = (point - center).abs();
        d.x <= xy_radius.x && d.y <= xy_radius.y
    }

}