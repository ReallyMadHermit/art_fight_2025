use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::smaa::Smaa;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy::render::view::NoFrustumCulling;
use crate::dino_run_characters::PITCH_CONSTANT;
use fastrand::Rng;
use crate::common::MaterialWizard;
use crate::dino_run_mechanics::LevelSpeed;

const CAVE_RADIUS: f32 = 3.0;
const CAVE_LENGTH: f32 = 50.0;
const CAVE_CENTER: f32 = CAVE_RADIUS * PITCH_CONSTANT;

pub fn spawn_cave_tunnel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cyl_mat = materials.add(
        StandardMaterial {
            base_color: Color::linear_rgb(0.2, 0.2, 0.2),
            perceptual_roughness: 1.0,
            reflectance: 0.1,
            metallic: 0.2,
            double_sided: true,
            cull_mode: Some(Face::Front),
            ..default()
        }
    );
    let slab_mat = materials.add(
        StandardMaterial {
            base_color: Color::linear_rgb(0.2, 0.2, 0.2),
            perceptual_roughness: 1.0,
            reflectance: 0.1,
            metallic: 0.2,
            ..default()
        }
    );
    let cyl_mesh = meshes.add(
        Cylinder::new(CAVE_RADIUS, CAVE_LENGTH)
    );
    let slab_mesh = meshes.add(
        Cuboid::new(CAVE_LENGTH, CAVE_RADIUS * 2.0, 0.5)
    );
    let slab_entity = commands.spawn(
        (
            Transform::from_xyz(2.5, 0.0, -0.25),
            Mesh3d(slab_mesh),
            MeshMaterial3d(slab_mat),
            NotShadowCaster
        )
    ).id();
    commands.spawn(
        (
            Transform::from_xyz(0.0, 0.0, CAVE_CENTER)
                .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
            Mesh3d(cyl_mesh),
            MeshMaterial3d(cyl_mat),
            NotShadowCaster,
            ChildOf(slab_entity)
        )
    );
    commands.spawn(
        (
            Camera3d::default(),
            Camera {
                hdr: true,
                ..default()
            },
            Projection::Perspective(
                PerspectiveProjection {
                    fov: FRAC_PI_4 * 0.9,
                    ..default()
                }
            ),
            Transform::from_translation(Vec3::new(0.0, -8.0, 3.5))
                .looking_at(Vec3::new(1.2, 0.0, 1.75), Vec3::Z),
            Bloom::OLD_SCHOOL,
            Tonemapping::AcesFitted,
            Smaa::default(),
            Msaa::Off
        )
    );
}

pub fn insert_crystal_stuff(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hues: Vec<f32> = {
        let normals = MaterialWizard::generate_normal_hue_vec(32);
        let mut hues: Vec<f32> = Vec::with_capacity(32);
        for i in 0..32 {
            hues.push(normals[i] * 360.0);
        };
        hues
    };
    
    let saturation = 1.0;
    let lightness = 0.5;
    let alpha = 0.5;
    let mat = StandardMaterial {
        base_color: Color::WHITE.with_alpha(alpha),
        perceptual_roughness: 0.2,
        reflectance: 0.6,
        diffuse_transmission: 0.3,
        ..default()
    };
    let wizard = MaterialWizard::new(
        &mut materials, mat, saturation,
        lightness, alpha, 32, 0.5, false,
    );
    
    commands.insert_resource(CrystalAssets{wizard, hues});
    commands.insert_resource(CrystalTimer::new());
    commands.insert_resource(AmbientLight {
        color: Color::WHITE, 
        brightness: 80.0, 
        ..default()});
}

#[derive(Resource)]
pub struct CrystalTimer{
    rng: Rng,
    last_x: f32
} impl CrystalTimer {
    fn new() -> Self {
        Self {rng: Rng::new(), last_x: 0.0}
    }
}

#[derive(Resource)]
pub struct CrystalAssets {
    wizard: MaterialWizard,
    hues: Vec<f32>
}
const CRYSTAL_A_RANGE: f32 = PI + FRAC_PI_2;

pub fn spawn_crystals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut crystal_timer: ResMut<CrystalTimer>,
    crystal_assets: Res<CrystalAssets>,
    time: Res<Time>,
    speed: Res<LevelSpeed>
) {
    crystal_timer.last_x -= time.delta_secs() * speed.f32;
    while crystal_timer.last_x <= 15.0 {
        let randoms = [crystal_timer.rng.f32(); 5];
        crystal_timer.last_x += randoms[0] * 0.5 + 1.5;
        let crystal_length = 0.5 + (randoms[2] - 0.5) * 0.2;
        let crystal_radius = 0.2 + (randoms[3] - 0.5) * 0.1;
        let a = -FRAC_PI_4 + CRYSTAL_A_RANGE * randoms[1];
        let x = crystal_timer.last_x;
        let y = a.cos() * (CAVE_RADIUS - crystal_length);
        let z = a.sin() * (CAVE_RADIUS - crystal_length) + CAVE_CENTER - 0.25;
        let mesh = meshes.add(Extrusion::new(
            RegularPolygon::new(crystal_radius, 6), crystal_length * 2.0));
        let i = (randoms[4] * 31.0).round() as usize;
        let material = crystal_assets.wizard.get_index(i);
        let hue = crystal_assets.hues[i];
        let light = PointLight {
            color: Color::hsl(hue, 1.0, 0.6),
            intensity: 24000.0,
            range: 8.0,
            shadows_enabled: true,
            radius: crystal_length,
            shadow_map_near_z: crystal_length * 2.0,
            ..default()
        };
        commands.spawn(
            (
                CrystalLight,
                Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_x(a - FRAC_PI_2)),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                light,
                NotShadowReceiver,
                NotShadowCaster,
                NoFrustumCulling
                )
        );
    };
}

#[derive(Component)]
pub struct CrystalLight;

pub fn update_lights(
    mut query: Query<(&mut Transform, Entity), With<CrystalLight>>,
    time: Res<Time>,
    mut commands: Commands,
    speed: Res<LevelSpeed>
) {
    let dt = time.delta_secs();
    let step = speed.f32 * dt;
    for (mut t, e) in &mut query {
        t.translation.x -= step;
        if t.translation.x < -7.0 {
            commands.entity(e).despawn()
        };
    };
}
