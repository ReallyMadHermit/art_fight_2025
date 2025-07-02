use bevy::{core_pipeline::{bloom::Bloom, smaa::Smaa, tonemapping::Tonemapping}, prelude::*};
use std::f32::consts::FRAC_PI_8;
use crate::common::MaterialWizard;
use crate::event_exists;

pub struct DinoRunPlugin;
impl Plugin for DinoRunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Startup, spawn_debug_cube);
        app.add_systems(Startup, spawn_light);
        app.add_systems(Startup, spawn_player_cube);
        app.add_systems(PreUpdate, player_jump_system);
        app.add_systems(Startup, insert_obstacle_resources);
        app.add_event::<SpawnObstacle>();
        app.add_systems(FixedPreUpdate, obstacle_spawn_timing);
        app.add_systems(FixedPreUpdate, obstacle_spawner.after(obstacle_spawn_timing).run_if(event_exists!(SpawnObstacle)));
        app.add_systems(Update, update_obstacles);
        app.add_event::<PlayerHit>();
        app.add_event::<PlayerScores>();
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
            Transform::from_translation(Vec3::new(2.5, -20.0, 10.0))
                .looking_at(Vec3::new(2.5, 0.0, 3.0), Vec3::Z),
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
        Cuboid::new(10.0, 10.0, 0.5)
    );
    commands.spawn(
        (
            Transform::from_xyz(2.5, 0.0, -0.25),
            Mesh3d(mesh),
            MeshMaterial3d(mat)
            )
    );
}

#[derive(Component)]
pub struct Player{
    velocity: f32
}

#[derive(Resource)]
pub struct PlayerEntity {
    entity: Entity
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
    commands.insert_resource(PlayerEntity{entity: player});
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

#[derive(Resource)]
struct ObstacleAssets {
    hexagon_prism_mesh: Handle<Mesh>,
    pentagon_prism_mesh: Handle<Mesh>,
    cube_mesh: Handle<Mesh>,
    wizard: MaterialWizard
}

const COLOR_COUNT: usize = 16;

fn insert_obstacle_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>
) {
    let wizard = {
        let saturation = 1.0;
        let lightness = 0.5;
        let alpha = 0.5;
        let color_count = COLOR_COUNT;
        let crystal_mat = StandardMaterial {
            base_color: Color::WHITE.with_alpha(alpha),
            perceptual_roughness: 0.2,
            reflectance: 0.6,
            diffuse_transmission: 0.3,
            ..default()
        };
        MaterialWizard::new(
            &mut materials, crystal_mat, saturation,
            lightness, alpha, color_count, 0.0, false,
        )
    };
    let hex = meshes.add(
        Extrusion::new(
            RegularPolygon::new(0.5, 6),
            1.0
        )
    );
    let pen = meshes.add(
        Extrusion::new(
            RegularPolygon::new(0.5, 5),
            1.0
        )
    );
    let cube = meshes.add(
        Cuboid::from_length(1.0)
    );
    let obs_assets = ObstacleAssets {
        hexagon_prism_mesh: hex,
        pentagon_prism_mesh: pen,
        cube_mesh: cube,
        wizard
    };
    commands.insert_resource(obs_assets);
    commands.insert_resource(ObstacleTimer{next: time.elapsed_secs() + 1.0, count: 0});
}

#[derive(Event)]
struct SpawnObstacle{
    count: u32
}

#[derive(Resource)]
struct ObstacleTimer {
    next: f32,
    count: u32
}

fn obstacle_spawn_timing(
    time: Res<Time>,
    mut obstacle_timer: ResMut<ObstacleTimer>,
    mut event_writer: EventWriter<SpawnObstacle>
) {
    let t = time.elapsed_secs();
    if t >= obstacle_timer.next {
        event_writer.write(SpawnObstacle{count: obstacle_timer.count});
        obstacle_timer.next = t + 5.0;
        obstacle_timer.count +=1;
    };
}

#[derive(Component)]
struct Obstacle {
    radius: f32,
    height: f32,
    scored: bool
}

fn obstacle_spawner(
    mut event_reader: EventReader<SpawnObstacle>,
    assets: Res<ObstacleAssets>,
    mut commands: Commands
) {
    for event in event_reader.read() {
        commands.spawn(
            (
                Transform::from_xyz(10.0, 0.0, 0.5),
                Mesh3d(assets.cube_mesh.clone()),
                MeshMaterial3d(assets.wizard.get_index((event.count as usize) % COLOR_COUNT)),
                Obstacle {
                    radius: 0.5,
                    height: 1.0,
                    scored: false
                }
            )
        );
    };
}

#[derive(Event)]
struct PlayerHit;

#[derive(Event)]
struct PlayerScores;

fn update_obstacles (
    mut transform_query: Query<&mut Transform>,
    mut obstacle_query: Query<(&mut Obstacle, Entity)>,
    p_entity: Res<PlayerEntity>,
    time: Res<Time>,
    mut hit_writer: EventWriter<PlayerHit>,
    mut score_writer: EventWriter<PlayerScores>
) {
    let dt = time.delta_secs();
    let motion = dt * 5.0;
    let player_z = transform_query.get(p_entity.entity).unwrap().translation.z;
    for (mut obstacle, entity) in &mut obstacle_query {
        let mut transform = transform_query.get_mut(entity).unwrap();
        transform.translation.x -= motion;
        if obstacle.scored {
            continue;
        } else if transform.translation.x.abs() < obstacle.radius && player_z < obstacle.height {
            println!("Hit!!");
            obstacle.scored = true;
            hit_writer.write(PlayerHit);
        } else if transform.translation.x < -obstacle.radius {
            obstacle.scored = true;
            println!("Score!!");
            score_writer.write(PlayerScores);
        };
    };
}