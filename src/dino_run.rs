use bevy::{core_pipeline::{bloom::Bloom, smaa::Smaa, tonemapping::Tonemapping}, prelude::*};
use std::f32::consts::FRAC_PI_8;
use crate::common::MaterialWizard;

pub struct DinoRunPlugin;
impl Plugin for DinoRunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Startup, spawn_debug_cube);
        app.add_systems(Startup, spawn_light);
        app.add_systems(Startup, spawn_player_cube);
        app.add_systems(PreUpdate, player_jump_system);
    }
}

fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn(
        (
            Camera3d::default(),
            Camera {
                hdr: true,
                ..default()
            },
            Projection::Perspective(
                PerspectiveProjection {
                    fov: FRAC_PI_8,
                    ..default()
                }
            ),
            Transform::from_translation(Vec3::new(0.0, -20.0, 10.0))
                .looking_at(Vec3::new(0.0, 0.0, 3.0), Vec3::Z),
            Bloom::OLD_SCHOOL,
            Tonemapping::AcesFitted,
            Smaa::default(),
            Msaa::Off
        )
    );
}

fn spawn_light(
    mut commands: Commands
) {
    commands.spawn(
        (
            DirectionalLight {
                illuminance: 5000.0,
                color: Color::WHITE,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)).looking_at(Vec3::ZERO, Vec3::Z)
        )
    );
}

fn spawn_debug_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mat = materials.add(
        StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        }
    );
    let mesh = meshes.add(
        Cuboid::new(10.0, 3.0, 0.2)
    );
    commands.spawn(
        (
            Transform::default(),
            Mesh3d(mesh),
            MeshMaterial3d(mat)
            )
    );
}

#[derive(Component)]
pub struct Player{
    velocity: f32
}

fn spawn_player_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player = commands.spawn(
        (
            Player{velocity: 0.0},
            Transform::default(),
            Visibility::Visible
        )
    ).id();
    commands.spawn(
        (
            Mesh3d(
                meshes.add(Cuboid::new(1.0, 1.0, 2.0))
            ),
            MeshMaterial3d(
                materials.add(
                    StandardMaterial {
                        base_color: Color::WHITE,
                        ..default()
                    }
                )
            ),
            Transform::from_xyz(0.0, 0.0, 1.0),
            ChildOf(player)
        )
    );
}

fn player_jump_system(
    mut query: Query<(&mut Transform, &mut Player)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>
) {
    //vars
    let jump_v = 8.0;
    // statics
    let dt = time.delta_secs();
    let g = 20.0 * dt;
    let f = g / 2.0;
    // inputs
    let jumped = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::ArrowUp)
        || keys.just_pressed(KeyCode::KeyW);
    let held = keys.pressed(KeyCode::Space)
        || keys.pressed(KeyCode::ArrowUp)
        || keys.pressed(KeyCode::KeyW);
    // query
    if let Ok((mut t, mut p)) = query.single_mut() {
        // jump key
        if t.translation.z < 0.25 && p.velocity <= 0.0 && jumped {
            p.velocity = jump_v;
        };
        // gravity, hold to jump higher
        let float = held && p.velocity > 0.0;
        if t.translation.z > 0.0 {
            if float {
                p.velocity -= f;
            } else {
                p.velocity -= g;
            };
        };
        // apply velocity
        if p.velocity != 0.0 {
            t.translation.z += p.velocity * dt;
        };
        // catch falling player
        if p.velocity < 0.0 && t.translation.z <= 0.0 {
            t.translation.z = 0.0;
            p.velocity = 0.0;
        };
    };
}

// #[derive(Component)]
// struct Obstacle {
//     radius: f32,
//     height: f32,
//     scored: bool
// }
// 
// #[derive(Resource)]
// struct ObstacleTimer {
//     next: f32
// }
// 
// #[derive(Resource)]
// struct ObstacleAssets {
//     hex_prism_mesh: Handle<Mesh>,
//     cube_mesh: Handle<Mesh>
// }
// 
// fn insert_obstacle_resources(
//     commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>
// ) {
//     
// }
