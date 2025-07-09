use bevy::{core_pipeline::{bloom::Bloom, tonemapping::Tonemapping}, prelude::*, render::camera::ScalingMode};

pub fn debug_scene_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let mesh = meshes.add(Sphere::new(0.5));
    let material = materials.add(StandardMaterial::from_color(Color::WHITE));
    for n in [-1.0, 0.0, 2.0] {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(n, n, 0.0)
        ));
    }
    commands.spawn(
        (
            Camera3d::default(),
            Camera {
                hdr: true,
                ..default()
            },
            Projection::Orthographic(
                OrthographicProjection {
                    scaling_mode: ScalingMode::FixedHorizontal {viewport_width: 10.0},
                    ..OrthographicProjection::default_3d()
                }
            ),
            Transform::from_xyz(0.0, 0.0, -10.0).looking_at(Vec3::ZERO, Vec3::Z),
            Bloom::OLD_SCHOOL,
            Tonemapping::AcesFitted,
            Msaa::Sample4
        )
    );
}