use bevy::prelude::*;

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
        saturation: f32, lightness: f32, alpha: f32, color_count: usize, use_emissive: bool, unlit: bool
    ) -> Self {
        let hues = Self::generate_normal_hue_vec(color_count);
        let mut standard_materials: Vec<StandardMaterial> = Vec::with_capacity(color_count);
        for hue in hues {
            let mut mat = reference_material.clone();
            mat.base_color = Color::hsla(hue * 360.0, saturation, lightness, alpha);
            standard_materials.push(mat);
        };
        if use_emissive {
            for mat in &mut standard_materials {
                mat.emissive = mat.base_color.to_linear();
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
            saturation, lightness, alpha, color_count, false, unlit
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