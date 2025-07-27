use std::f32::consts::FRAC_PI_6;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use crate::spider_well::level_layout::{Stage1, LEVEL_WIDTH, Stage1Gaps};
use crate::spider_well::mechanics::{POVCamera, PlayerEntity};

const LIGHT_Z: f32 = 1.5;
const LIGHT_COLOR: Color = Color::linear_rgb(1.0, 0.8, 0.4);

pub fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let camera = commands.spawn(
        (
            Camera3d::default(),
            Camera {
                hdr: true,
                ..default()
            },
            Projection::Orthographic(
                OrthographicProjection {
                    scaling_mode: ScalingMode::FixedHorizontal {viewport_width: LEVEL_WIDTH},
                    ..OrthographicProjection::default_3d()
                }
            ),
            Transform::from_xyz(0.0, -3.0, LEVEL_WIDTH)
                .looking_at(Vec3::new(0.0, -3.0, 0.0), Vec3::Y),
            Bloom::OLD_SCHOOL,
            Tonemapping::AcesFitted,
            Msaa::Sample4,
            POVCamera
        )
    ).id();
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(LEVEL_WIDTH, LEVEL_WIDTH, 0.0125))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.2, 0.2, 0.2),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -LEVEL_WIDTH -0.5),
        ChildOf(camera),
        NotShadowCaster
    ));
}

pub fn spawn_lights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gaps: Res<Stage1Gaps>
) {
    commands.insert_resource(AmbientLight{
        color: Color::BLACK,
        brightness: 0.0,
        ..default()
    });
    commands.insert_resource(ClearColor(Color::BLACK));
    for vec in &gaps.vec {
        commands.spawn((
            PointLight {
                color: LIGHT_COLOR,
                intensity: 20000.0,
                range: 10.0,
                radius: 0.5,
                shadows_enabled: true,
                shadow_map_near_z: 0.25,
                ..default()
            },
            Transform::from_translation(vec.extend(LIGHT_Z))
        ));
    }
    let light_mat = materials.add(StandardMaterial {
        base_color: LIGHT_COLOR,
        unlit: true,
        ..default()
    });
    let post_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.6, 0.5, 0.5),
        perceptual_roughness: 0.5,
        metallic: 0.2,
        ..default()
    });
    let bulb_mesh = meshes.add(Sphere::new(0.25));
    let post_mesh = meshes.add(Cone::new(0.25, 7.0));
    // lamp light
    commands.spawn((
        PointLight {
            color: LIGHT_COLOR,
            intensity: 50000.0,
            range: 10.0,
            radius: 0.25,
            shadows_enabled: true,
            shadow_map_near_z: 0.3,
            ..default()
        },
        Transform::from_xyz(0.0, 3.0, 1.5)
    ));
    //lamp bulb
    commands.spawn((
        Mesh3d(bulb_mesh),
        MeshMaterial3d(light_mat),
        Transform::from_xyz(0.0, 3.0, -0.5),
        NotShadowReceiver,
        NotShadowCaster
    ));
    // post
    commands.spawn((
        Mesh3d(post_mesh),
        MeshMaterial3d(post_mat),
        Transform::from_xyz(-1.2, 1.0, -0.5).with_rotation(Quat::from_rotation_z(-FRAC_PI_6)),
        NotShadowCaster
    ));
}