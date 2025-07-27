use bevy::prelude::*;
pub mod mechanics;
use mechanics::{
    spawn_player, insert_simple_resources, player_controls, move_player,
    insert_webbing_assets, web_updater, camera_mover, CollisionEvent, collision_rect_checker,
    move_obstacles, checkpoint_checker, damsel_checker
};

pub mod level_layout;
use level_layout::{
    insert_debug_level_assets, spawn_stage_1, spawn_stage_2
};

pub mod characters;
use characters::{
    spawn_spider_parts, insert_limb_positions, apply_limb_positions, insert_spider_materials, 
    spawn_body, spawn_bum, spawn_head, calculate_leg_joints, calculate_arm_joints
};

pub mod environment;
use environment::{
    spawn_lights, spawn_camera
};

pub struct SpiderWellPlugin;
impl Plugin for SpiderWellPlugin {
    fn build(&self, app: &mut App) {
        // mechanics
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
        app.add_systems(Update, checkpoint_checker);
        app.add_systems(Update, damsel_checker);

        // layout
        app.add_systems(Startup, insert_debug_level_assets);
        app.add_systems(Startup, (
            spawn_stage_1, spawn_stage_2
        ).after(insert_debug_level_assets));
        
        // characters
        app.add_systems(Startup, insert_spider_materials.after(spawn_player));
        app.add_systems(Startup, spawn_spider_parts.after(insert_spider_materials));
        app.add_systems(Startup, insert_limb_positions);
        app.add_systems(Startup, spawn_body.after(insert_spider_materials));
        app.add_systems(Startup, spawn_bum.after(insert_spider_materials));
        app.add_systems(Startup, spawn_head.after(insert_spider_materials));
        app.add_systems(Update, apply_limb_positions);
        app.add_systems(Update, calculate_leg_joints);
        app.add_systems(Update, calculate_arm_joints.before(calculate_leg_joints));
        
        // environment
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Startup, spawn_lights.after(spawn_stage_1));
    }
}