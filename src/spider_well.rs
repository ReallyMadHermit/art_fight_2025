use bevy::prelude::*;
pub mod mechanics;
use mechanics::{
    debug_scene_setup, spawn_player, insert_simple_resources, player_controls, move_player,
    insert_webbing_assets, web_updater, camera_mover, CollisionEvent, collision_rect_checker,
    move_obstacles
};

pub mod level_layout;
use level_layout::{
    insert_debug_level_assets, spawn_stage_1
};

pub struct SpiderWellPlugin;
impl Plugin for SpiderWellPlugin {
    fn build(&self, app: &mut App) {
        // mechanics
        app.add_systems(Startup, debug_scene_setup);
        app.add_systems(Startup, insert_simple_resources);
        app.add_systems(Startup, insert_webbing_assets);
        app.add_systems(Startup, spawn_player.after(insert_webbing_assets));
        app.add_systems(PreUpdate, player_controls);
        app.add_systems(Update, move_player);
        app.add_systems(Update, web_updater.after(move_player));
        app.add_systems(PostUpdate, camera_mover);
        app.add_event::<CollisionEvent>();
        app.add_systems(Update, collision_rect_checker.after(move_player));
        app.add_systems(Update, move_obstacles.before(collision_rect_checker));
        
        // layout
        app.add_systems(Startup, insert_debug_level_assets);
        app.add_systems(Startup, spawn_stage_1.after(insert_debug_level_assets));
    }
}