use bevy::prelude::*;
use std::f32::consts::{SQRT_2, PI, FRAC_PI_2};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use crate::dino_run_mechanics::{LevelSpeed, Player, JUMP_V};

const PITCH_CONSTANT: f32 = SQRT_2 / 2.0;
const GREY: f32 = 0.4;

#[derive(Resource)]
pub struct AnimationState {
    pub jumping: bool
}

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub enum LegPart {
    Hip,
    LeftThigh,
    RightThigh,
    LeftShin,
    RightShin
} impl LegPart {
    const ALL: [LegPart; 5] = [
        Self::Hip,
        Self::LeftThigh,
        Self::RightThigh,
        Self::LeftShin,
        Self::RightShin,
    ];
}

pub fn spawn_legs(
    player: Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let joint_mesh = meshes.add(Sphere::new(0.125));
    let bone_mesh = meshes.add(Cuboid::new(0.15, 0.2, 0.6));
    let black = materials.add(StandardMaterial::from_color(
        Color::linear_rgb(0.05, 0.05, 0.05)));
    let grey = materials.add(StandardMaterial::from(
        Color::linear_rgb(GREY, GREY, GREY)));
    let mut hip_entity = Entity::PLACEHOLDER;
    for leg_part in LegPart::ALL{
        let mesh = match leg_part {
            LegPart::Hip => {
                joint_mesh.clone()
            }
            _ => {
                bone_mesh.clone()
            }
        };
        let mat = match leg_part {
            LegPart::Hip | LegPart::LeftThigh | LegPart::RightThigh => {
                black.clone()
            },
            _ => {
                grey.clone()
            }
        };
        let scale = {
            match leg_part {
                LegPart::LeftThigh | LegPart::RightThigh => { Vec3::ONE * 1.1 },
                _ => {Vec3::ONE}
            }
        };
        let joint_entity = commands.spawn(
            (
                Transform::from_scale(scale),
                leg_part,
                Visibility::Inherited,
                ChildOf(player),
                Mesh3d(mesh),
                MeshMaterial3d(mat)
            )
        ).id(); 
        if leg_part == LegPart::Hip {
            hip_entity = joint_entity;
        };
    };
    commands.insert_resource(AnimationState{jumping: false});
    hip_entity
}

pub fn animate_legs(
    mut query: Query<(&mut Transform, &LegPart)>,
    player_query: Query<&Player>,
    animation_state: Res<AnimationState>,
    time: Res<Time>,
    speed: Res<LevelSpeed>
) {
    let leg_length = 1.2;
    let step_height = 0.5;
    let hip_splay = 0.25;
    let bone_length = leg_length / 2.0;
    let (
        left_foot, right_foot, left_knee, right_knee, hip, left_hip, right_hip
    ) = if animation_state.jumping {
        let v = player_query.single().unwrap().velocity;
        let hip_height = leg_length;
        let jump_normal = (v / JUMP_V).abs();
        let foot_height = step_height - (jump_normal * step_height);
        let hip = Vec3::new(0.2, 0.0, hip_height);
        let mut left_hip = hip;
        let mut right_hip = hip;
        left_hip.y -= hip_splay;
        right_hip.y += hip_splay;
        let left_foot = Vec3::new(0.2, -hip_splay, foot_height);
        let right_foot = Vec3::new(0.2, hip_splay, foot_height);
        let left_knee = calculate_knee_pos(left_foot, left_hip, bone_length, leg_length);
        let right_knee = calculate_knee_pos(right_foot, right_hip, bone_length, leg_length);
        (left_foot, right_foot, left_knee, right_knee, hip, left_hip, right_hip)
        // (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO)
    } else {
        let step_distance = PITCH_CONSTANT;
        let t = time.elapsed_secs() * (speed.f32 / (step_distance * 2.0));
        let t1 = t % 2.0;
        let t2 = (t + 1.0) % 2.0;
        let step_height = 0.5;
        let left_foot = calculate_foot_pos(t1, step_distance, step_height, -hip_splay);
        let right_foot = calculate_foot_pos(t2, step_distance, step_height, hip_splay);
        let mut hip = calculate_hip_pos(t, leg_length, step_distance);
        hip.x += 0.2;
        let mut left_hip = hip;
        let mut right_hip = hip;
        left_hip.y -= hip_splay;
        right_hip.y += hip_splay;
        let left_knee = calculate_knee_pos(left_foot, left_hip, leg_length / 2.0, leg_length);
        let right_knee = calculate_knee_pos(right_foot, right_hip, leg_length / 2.0, leg_length);
        (left_foot, right_foot, left_knee, right_knee, hip, left_hip, right_hip)
    };
    for (mut transform, &joint) in &mut query {
        match joint {
            LegPart::Hip => {transform.translation = hip;},
            LegPart::LeftThigh => {
                transform.translation = (left_hip + left_knee) / 2.0;
                transform.look_at(left_knee, Vec3::Z);
            },
            LegPart::RightThigh => {
                transform.translation = (right_hip + right_knee) / 2.0;
                transform.look_at(right_knee, Vec3::Z)
            },
            LegPart::LeftShin => {
                transform.translation = (left_knee + left_foot) / 2.0;
                transform.look_at(left_foot, Vec3::Z);
            },
            LegPart::RightShin => {
                transform.translation = (right_knee + right_foot) / 2.0;
                transform.look_at(right_foot, Vec3::Z);
            }
        };
    };
}

fn calculate_foot_pos(
    t: f32, step_displacement: f32, step_height: f32, y: f32
) -> Vec3 {
    if t < 1.0 {
        let foot_x = (t * PI).cos() * -step_displacement;
        // let foot_y = ((t * PI).sin() * y + y) * 0.5;
        let foot_z = (t * PI).sin() * step_height;
        Vec3::new(foot_x, y, foot_z)
    } else {
        let foot_x = step_displacement - ((t - 1.0) * step_displacement * 2.0);
        Vec3::new(foot_x, y, 0.0)
    }
}

fn calculate_hip_pos(
    t: f32, limb_length: f32, step_displacement: f32
) -> Vec3 {
    let hip_bob_amount = 0.2 * step_displacement;
    let hip_height = limb_length * 0.9 - hip_bob_amount * 1.2;
    let z = hip_height + (((t % 1.0) * PI).sin().abs()) * hip_bob_amount;
    Vec3::new(0.0, 0.0, z)
}

fn calculate_knee_pos(
    foot_pos: Vec3, hip_pos: Vec3, bone_length: f32, limb_length: f32
) -> Vec3 {
    let mut projection_pos = (foot_pos + hip_pos) / 2.0;
    let projection_angle = hip_pos.z.atan2(foot_pos.x) + FRAC_PI_2;
    let displacement = (foot_pos - hip_pos).length();
    if displacement < limb_length {
        let d2 = displacement / 2.0;
        // let projection_length = bone_length * bone_length - Vec2{x: d2, y: bone_length}.length();
        let projection_length = ((bone_length * bone_length) - (d2 * d2)).sqrt();
        projection_pos.x += projection_angle.cos() * projection_length;
        projection_pos.z -= projection_angle.sin() * projection_length;
        projection_pos.y = foot_pos.y;
    };
    return projection_pos;
}

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub struct BodyPart;

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub struct TailSegment {
    i: u8
}

const TAIL_STEP: f32 = 0.25;
const TAIL_LENGTH: usize = 8;
const STRIPE_DEPTH: f32 = 1.0 / 64.0;
const TAIL_Z: f32 = 0.15;

pub fn spawn_body (
    hip_entity: Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>
) -> Entity {
    let black = materials.add(
        StandardMaterial::from_color(Color::linear_rgb(0.05, 0.05, 0.05)));
    let yellow = materials.add(
        StandardMaterial::from_color(Color::hsl(58.0, 1.0, 0.5)));
    let body0_mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.6));
    let body1_mesh = meshes.add(Cuboid::new(0.5, 0.4, 0.5));
    let body2_mesh = meshes.add(Cuboid::new(0.4, 0.4, 0.4));
    let body_stripe_mesh_0 = meshes.add(
        Cuboid::new(0.5 + STRIPE_DEPTH, 0.5 + STRIPE_DEPTH, 0.15));
    let body_stripe_mesh_1 = meshes.add(
        Cuboid::new(0.5 + STRIPE_DEPTH, 0.4 + STRIPE_DEPTH, 0.1));
    let body_stripe_mesh_2 = meshes.add(
        Cuboid::new(0.4 + STRIPE_DEPTH, 0.4 + STRIPE_DEPTH, 0.1));
    let body_0 = commands.spawn(
        (
            Transform::from_xyz(0.0, 0.0, 0.1),
            BodyPart,
            Visibility::Inherited,
            ChildOf(hip_entity),
            Mesh3d(body0_mesh),
            MeshMaterial3d(black.clone())
        )
    ).id();
    let body_1 = commands.spawn(
        (
            Transform::from_xyz(0.5, 0.0, 0.15),
            BodyPart,
            Visibility::Inherited,
            ChildOf(hip_entity),
            Mesh3d(body1_mesh),
            MeshMaterial3d(black.clone())
        )
    ).id();
    let body_2 = commands.spawn(
        (
            Transform::from_xyz(-0.45, 0.0, 0.2),
            BodyPart,
            Visibility::Inherited,
            ChildOf(hip_entity),
            Mesh3d(body2_mesh),
            MeshMaterial3d(black.clone())
        )
    ).id();
    
    commands.spawn(
        (
            Transform::from_xyz(0.0, 0.0, 0.1),
            Visibility::Inherited,
            ChildOf(body_0),
            Mesh3d(body_stripe_mesh_0),
            MeshMaterial3d(yellow.clone()),
            NotShadowCaster
        )
    );
    commands.spawn(
        (
            Transform::from_xyz(0.0, 0.0, 0.1),
            Visibility::Inherited,
            ChildOf(body_1),
            Mesh3d(body_stripe_mesh_1),
            MeshMaterial3d(yellow.clone()),
            NotShadowCaster
        )
    );
    commands.spawn(
        (
            Transform::default(),
            Visibility::Inherited,
            ChildOf(body_2),
            Mesh3d(body_stripe_mesh_2),
            MeshMaterial3d(yellow.clone()),
            NotShadowCaster
        )
    );
    let tail_stripe_mesh = meshes.add(Cuboid::new(
        TAIL_STEP + STRIPE_DEPTH, 0.3  + STRIPE_DEPTH, 0.05));
    let tail_mesh = meshes.add(Cuboid::new(TAIL_STEP, 0.3, 0.3));
    for i in 0..TAIL_LENGTH {
        let s = 1.0 - (i as f32 * 0.05);
        let seg = commands.spawn(
            (
                Transform::from_xyz(-TAIL_STEP * i as f32 - 0.5, 0.0, TAIL_Z)
                    .with_scale(Vec3::new(1.0, s, s)),
                TailSegment{i: i as u8},
                Visibility::Inherited,
                ChildOf(body_0),
                Mesh3d(tail_mesh.clone()),
                MeshMaterial3d(black.clone())
            )
        ).id();
        commands.spawn(
            (
                Transform::default(),
                Visibility::Inherited,
                ChildOf(seg),
                Mesh3d(tail_stripe_mesh.clone()),
                MeshMaterial3d(yellow.clone())
            )
        );
    };

    body_1
}

pub fn animate_tail(
    mut query: Query<(&mut Transform, &TailSegment)>,
    player_query: Query<&Player>,
    animation_state: Res<AnimationState>,
    time: Res<Time>,
    speed: Res<LevelSpeed>
) {
    if animation_state.jumping {
        let v = player_query.single().unwrap().velocity;
        let a = (v / JUMP_V) * FRAC_PI_2;
        let a_step = a / TAIL_LENGTH as f32;
        let wag = 0.5;
        for (mut transform, segment) in &mut query {
            transform.translation.z = TAIL_Z - (((segment.i + 1) as f32 * a_step).sin() * wag);
            transform.translation.y = 0.0;
        };
    } else {
        let step_distance = PITCH_CONSTANT;
        let t = time.elapsed_secs() * (speed.f32 / (step_distance * 2.0));
        let a = t * PI;
        let a_step = (PI * 2.0) / TAIL_LENGTH as f32;
        for (mut transform, segment) in &mut query {
            let y = (a + a_step).sin() * (segment.i as f32 * 0.05);
            transform.translation.y = y * (1.0 + segment.i as f32 * 0.05);
            transform.translation.z = TAIL_Z;
        };
    };
}

#[derive(Component, Copy, Clone, PartialEq, Eq)]
pub struct Head{
    i: u8
}

const NECK_X: f32 = 0.35;
const SKULL_X: f32 = 0.65;

pub fn spawn_neck_and_head(
    body_1_entity: Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>
) {
    let black = materials.add(
        StandardMaterial::from_color(Color::linear_rgb(0.05, 0.05, 0.05)));
    let yellow = materials.add(
        StandardMaterial::from_color(Color::hsl(58.0, 1.5, 0.5)));
    let eye_color = materials.add(
        StandardMaterial{
            base_color: Color::hsl(38.0, 1.0, 0.5),
            unlit: true,
            ..default()
        });
    let eye_mesh = meshes.add(
        Cuboid::new(0.075, 0.425, 0.1));
    let grey = materials.add(
        StandardMaterial::from(Color::linear_rgb(GREY, GREY, GREY)));
    let neck0_mesh = meshes.add(
        Cuboid::new(0.4, 0.4, 0.3));
    let skull_mesh = meshes.add(
        Cuboid::from_length(0.4));
    let beak_mesh = meshes.add(
        Cuboid::new(0.5, 0.4 + STRIPE_DEPTH * 2.0, 0.2));
    let neck_stripe = meshes.add(
        Cuboid::new(0.4 + STRIPE_DEPTH, 0.4 + STRIPE_DEPTH, 0.075));
    let skull_stripe = meshes.add(
        Cuboid::new(0.4 + STRIPE_DEPTH, 0.4 + STRIPE_DEPTH, 0.05));
    let neck = commands.spawn(
        (
            Transform::from_xyz(NECK_X, 0.0, 0.1),
            Visibility::Inherited,
            ChildOf(body_1_entity),
            Mesh3d(neck0_mesh),
            MeshMaterial3d(black.clone()),
            Head{i: 0}
        )
    ).id();
    let skull = commands.spawn(
        (
            Transform::from_xyz(SKULL_X, 0.0, 0.15),
            Visibility::Inherited,
            ChildOf(body_1_entity),
            Mesh3d(skull_mesh),
            MeshMaterial3d(black.clone()),
            Head{i: 1}
        )
    ).id();
    
    // beak
    commands.spawn(
        (
            Transform::from_xyz(0.3, 0.0, -0.1),
            Visibility::Inherited,
            ChildOf(skull),
            Mesh3d(beak_mesh),
            MeshMaterial3d(grey.clone())
        )
    );
    
    // stripes
    commands.spawn(
        (
            Transform::default(),
            Visibility::Inherited,
            ChildOf(neck),
            Mesh3d(neck_stripe),
            MeshMaterial3d(yellow.clone()),
            NotShadowCaster
        )
    );
    commands.spawn(
        (
            Transform::from_xyz(0.0, 0.0, -0.1),
            Visibility::Inherited,
            ChildOf(skull),
            Mesh3d(skull_stripe),
            MeshMaterial3d(yellow.clone()),
            NotShadowCaster
        )
    );
    
    // eyes
    commands.spawn(
        (
            Transform::from_xyz(0.125, 0.0, 0.1),
            Visibility::Inherited,
            ChildOf(skull),
            Mesh3d(eye_mesh.clone()),
            MeshMaterial3d(eye_color.clone()),
            NotShadowCaster,
            NotShadowReceiver
        )
    );
}