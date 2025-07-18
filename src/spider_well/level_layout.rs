use bevy::prelude::*;
use crate::spider_well::mechanics::{
    CollisionRect, spawn_checkpoint
};

pub const LEVEL_WIDTH: f32 = 20.0;
pub const LEVEL_DEPTH: f32 = 1.0;

struct Stage1;
impl Stage1 {
    const START: f32 = -2.0;
    const VERTICAL_SPACING: f32 = 5.0;
    const HOLE_RADIUS: f32 = 2.0;
    const INITIAL_WIDTH: f32 = 1.5;
    const HOLE_WIDENING: f32 = 0.5;
    const LEVEL_LENGTH: usize = 6;
    const PLATFORM_THICKNESS: f32 = 0.5;
    const END: f32 = Self::START - (Self::VERTICAL_SPACING * (Self::LEVEL_LENGTH - 1) as f32);
    const CHECKPOINT: f32 = Self::END - 2.0;
}

#[derive(Resource)]
pub struct DebugLevelAssets {
    cube: Handle<Mesh>,
    red_mat: Handle<StandardMaterial>,
    green_mat: Handle<StandardMaterial>,
    blue_mat: Handle<StandardMaterial>
}

pub fn insert_debug_level_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let cube = meshes.add(Cuboid::from_length(1.0));
    let red_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.0, 0.0),
        unlit: true,
        ..default()
    });
    let green_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 1.0, 0.0),
        unlit: true,
        ..default()
    });
    let blue_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 0.0, 1.0),
        unlit: true,
        ..default()
    });
    commands.insert_resource(DebugLevelAssets{
        cube, red_mat, green_mat, blue_mat
    });
}

pub fn spawn_stage_1(
    mut commands: Commands,
    assets: Res<DebugLevelAssets>
) {
    let x_max = (LEVEL_WIDTH / 2.0);
    let x_min = -x_max;
    for i in 0..Stage1::LEVEL_LENGTH {
        let sign = if i % 2 == 0 {
            1.0f32
        } else {
            -1.0f32
        };
        let mat = match i % 3 {
            0 => assets.red_mat.clone(),
            1 => assets.green_mat.clone(),
            _ => assets.blue_mat.clone()
        };
        let hole_x = sign * (Stage1::INITIAL_WIDTH + Stage1::HOLE_WIDENING * i as f32);
        let hole_y = Stage1::START - i as f32 * Stage1::VERTICAL_SPACING;
        let left_edge = hole_x - (Stage1::HOLE_RADIUS);
        let right_edge = hole_x + (Stage1::HOLE_RADIUS);
        let left_center = (left_edge + x_min) / 2.0;
        let right_center = (right_edge + x_max) / 2.0;
        let left_length = (left_edge - x_min).abs();
        let right_length = (right_edge - x_max).abs();
        commands.spawn((
            MeshMaterial3d(mat.clone()),
            Mesh3d(assets.cube.clone()),
            Transform::from_xyz(left_center, hole_y, 0.0).with_scale(
                Vec3::new(left_length, Stage1::PLATFORM_THICKNESS, LEVEL_DEPTH)),
            CollisionRect::new(left_length, Stage1::PLATFORM_THICKNESS)
        ));
        commands.spawn((
            MeshMaterial3d(mat),
            Mesh3d(assets.cube.clone()),
            Transform::from_xyz(right_center, hole_y, 0.0).with_scale(
                Vec3::new(right_length, Stage1::PLATFORM_THICKNESS, LEVEL_DEPTH)),
            CollisionRect::new(right_length, Stage1::PLATFORM_THICKNESS)
        ));
    };
    spawn_checkpoint(Stage1::CHECKPOINT, &mut commands);
}