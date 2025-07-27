use bevy::prelude::*;
use crate::spider_well::mechanics::{CollisionRect, spawn_checkpoint, ObstaclePathing};

pub const LEVEL_WIDTH: f32 = 16.0;
pub const LEVEL_DEPTH: f32 = 1.0;
const X_MAX: f32 = LEVEL_WIDTH / 2.0;
const X_MIN: f32 = -X_MAX;
pub const DAMSEL_Y: f32 = Stage2::BLOCKS2END - 5.0;

pub struct Stage1;
impl Stage1 {
    const START: f32 = -2.0;
    const VERTICAL_SPACING: f32 = 5.0;
    const HOLE_RADIUS: f32 = 2.0;
    pub const INITIAL_WIDTH: f32 = 1.5;
    const HOLE_WIDENING: f32 = 0.5;
    pub const LEVEL_LENGTH: usize = 6;
    const PLATFORM_THICKNESS: f32 = 0.5;
    const END: f32 = Self::START - (Self::VERTICAL_SPACING * (Self::LEVEL_LENGTH - 1) as f32);
    const CHECKPOINT: f32 = Self::END - 2.0;
}

#[derive(Resource)]
pub struct DebugLevelAssets {
    cube: Handle<Mesh>,
    red_mat: Handle<StandardMaterial>,
    green_mat: Handle<StandardMaterial>,
    blue_mat: Handle<StandardMaterial>,
    black_mat: Handle<StandardMaterial>
}

pub fn insert_debug_level_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let cube = meshes.add(Cuboid::from_length(1.0));
    let red_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.0, 0.0),
        ..default()
    });
    let green_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 1.0, 0.0),
        ..default()
    });
    let blue_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 0.0, 1.0),
        ..default()
    });
    let black_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.3, 0.3, 0.3),
        metallic: 0.7,
        perceptual_roughness: 0.3,
        ..default()
    });
    commands.insert_resource(DebugLevelAssets{
        cube, red_mat, green_mat, blue_mat, black_mat
    });
}

#[derive(Resource)]
pub struct Stage1Gaps {
    pub vec: Vec<Vec2>
}

pub fn spawn_stage_1(
    mut commands: Commands,
    assets: Res<DebugLevelAssets>
) {
    let mut gaps = Stage1Gaps{vec: Vec::with_capacity(Stage1::LEVEL_LENGTH)};
    for i in 0..Stage1::LEVEL_LENGTH {
        let sign = if i % 2 == 0 {
            1.0f32
        } else {
            -1.0f32
        };
        // let mat = match i % 3 {
        //     0 => assets.red_mat.clone(),
        //     1 => assets.green_mat.clone(),
        //     _ => assets.blue_mat.clone()
        // };
        let mat = assets.black_mat.clone();
        let hole_x = sign * (Stage1::INITIAL_WIDTH + Stage1::HOLE_WIDENING * i as f32);
        let hole_y = Stage1::START - i as f32 * Stage1::VERTICAL_SPACING;
        let left_edge = hole_x - (Stage1::HOLE_RADIUS);
        let right_edge = hole_x + (Stage1::HOLE_RADIUS);
        let left_center = (left_edge + X_MIN) / 2.0;
        let right_center = (right_edge + X_MAX) / 2.0;
        let left_length = (left_edge - X_MIN).abs();
        let right_length = (right_edge - X_MAX).abs();
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
        gaps.vec.push(Vec2::new(hole_x, hole_y));
    };
    spawn_checkpoint(Stage1::CHECKPOINT, &mut commands);
    commands.insert_resource(gaps);
}

struct Stage2;
impl Stage2 {
    const START: f32 = Stage1::CHECKPOINT - 2.0;
    const VERTICAL_SPACING: f32 = 5.0;
    const BLOCKS1HEIGHT: f32 = 2.0;
    const BLOCKS1DURATION: f32 = 5.0;
    const BLOCKS1WIDTH: f32 = 6.0;
    const BLOCKS1END: f32 = Self::START - Self::BLOCKS1HEIGHT * 3.0;

    fn spawn_blocks_1(
        commands: &mut Commands, assets: &Res<DebugLevelAssets>
    ) {
        let right_x = Self::BLOCKS1WIDTH / 2.0;
        let left_x = -right_x;
        let top_y = Self::START - Self::BLOCKS1HEIGHT / 2.0;
        let bottom_y = top_y - Self::BLOCKS1HEIGHT * 2.0;
        let tl = Vec2::new(left_x, top_y);
        let tr = Vec2::new(right_x, top_y);
        let bl = Vec2::new(left_x, bottom_y);
        let br = Vec2::new(right_x, bottom_y);
        let ways_a = vec![tl, tr, br, bl];
        let ways_b = vec![br, bl, tl, tr];
        for ways in [ways_a, ways_b] {
            commands.spawn((
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(assets.blue_mat.clone()),
                Transform::from_xyz(0.0, -Self::START, 0.0)
                    .with_scale(Vec3::new(Self::BLOCKS1WIDTH, Self::BLOCKS1HEIGHT, LEVEL_DEPTH)),
                ObstaclePathing::uniform_timing(ways, Self::BLOCKS1DURATION),
                CollisionRect::new(Self::BLOCKS1WIDTH, Self::BLOCKS1HEIGHT)
            ));
        };
        let filler_y = (top_y + bottom_y) / 2.0;
        let filler_width = X_MAX - Self::BLOCKS1WIDTH;
        let filler_x = X_MAX - filler_width / 2.0;
        for x in [filler_x, -filler_x] {
            commands.spawn((
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(assets.green_mat.clone()),
                Transform::from_xyz(x, filler_y, 0.0)
                    .with_scale(Vec3::new(filler_width, Self::BLOCKS1HEIGHT * 3.0, LEVEL_DEPTH)),
                CollisionRect::new(filler_width, Self::BLOCKS1HEIGHT * 3.0)
            ));
        };
    }

    const BLOCKS2START: f32 = Self::BLOCKS1END - Self::VERTICAL_SPACING;
    const BLOCKS2COUNT: usize = 10;
    const BLOCKS2SIZE: f32 = 2.5;
    const BLOCKS2SWING_RANGE: f32 = 4.0;
    const BLOCKS2DURATION: f32 = 4.0;
    const BLOCKS2END: f32 = Self::BLOCKS2START - (Self::BLOCKS2SIZE * Self::BLOCKS2COUNT as f32);

    fn spawn_blocks_2(commands: &mut Commands, assets: &Res<DebugLevelAssets>) {
        let scale_vec = Vec3::ONE * Self::BLOCKS2SIZE;
        let mats = [&assets.red_mat, &assets.green_mat, &assets.blue_mat];
        let mesh = &assets.cube;
        let mut lowest_y = 0.0;
        for i in 0..Self::BLOCKS2COUNT {
            let y = Self::BLOCKS2START - Self::BLOCKS2SIZE * i as f32 - Self::BLOCKS2SIZE / 2.0;
            let x = if i % 2 == 0 {
                Self::BLOCKS2SWING_RANGE
            } else {
                -Self::BLOCKS2SWING_RANGE
            };
            let ways = vec![
                Vec2::new(x, y),
                Vec2::new(-x, y)
            ];
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mats[i % 3].clone()),
                Transform::from_xyz(x, y, 0.0)
                    .with_scale(scale_vec),
                ObstaclePathing::uniform_timing(ways, Self::BLOCKS2DURATION),
                CollisionRect::new(Self::BLOCKS2SIZE, Self::BLOCKS2SIZE)
            ));
            lowest_y = y;
        };
        let filler_width = X_MAX - Self::BLOCKS2SWING_RANGE - Self::BLOCKS2SIZE / 2.0;
        let filler_height = (Self::BLOCKS2START - Self::BLOCKS2END).abs();
        let filler_x = X_MAX - filler_width / 2.0;
        let filler_y = (Self::BLOCKS2START + Self::BLOCKS2END) / 2.0;
        for x in [-filler_x, filler_x] {
            commands.spawn((
                Mesh3d(assets.cube.clone()),
                MeshMaterial3d(assets.blue_mat.clone()),
                Transform::from_xyz(x, filler_y, 0.0)
                    .with_scale(Vec3::new(filler_width, filler_height, LEVEL_DEPTH)),
                CollisionRect::new(filler_width, filler_height)
            ));
        };
        spawn_checkpoint(Self::BLOCKS2END - 2.0, commands);
    }
}

pub fn spawn_stage_2(
    mut commands: Commands,
    assets: Res<DebugLevelAssets>
) {
    Stage2::spawn_blocks_1(&mut commands, &assets);
    Stage2::spawn_blocks_2(&mut commands, &assets);
}