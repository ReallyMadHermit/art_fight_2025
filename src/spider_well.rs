use bevy::prelude::*;
pub mod mechanics;

use mechanics::debug_scene_setup;

pub struct SpiderWellPlugin;
impl Plugin for SpiderWellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, debug_scene_setup);
    }
}