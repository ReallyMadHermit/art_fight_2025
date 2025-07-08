use bevy::prelude::*;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::render::view::NoFrustumCulling;
use crate::common::MaterialWizard;
use crate::event_exists;
use crate::dino_run_characters::{
    spawn_legs, animate_legs, AnimationState, spawn_body, animate_tail, spawn_neck_and_head
};
use crate::dino_run_environment::{spawn_cave_tunnel, insert_crystal_stuff, spawn_crystals, update_lights};
use crate::dino_run_audio::setup_audio;
use fastrand::Rng;

pub struct DinoRunPlugin;
impl Plugin for DinoRunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        app.add_systems(PreUpdate, player_jump_system);
        app.add_systems(Startup, insert_obstacle_resources);
        app.add_event::<SpawnObstacle>();
        app.add_systems(FixedPreUpdate, obstacle_spawn_timing);
        app.add_systems(FixedPreUpdate, obstacle_spawner.after(obstacle_spawn_timing).run_if(event_exists!(SpawnObstacle)));
        app.add_systems(Update, update_obstacles);
        app.add_event::<PlayerHit>();
        app.add_event::<PlayerScores>();
        app.insert_resource(LevelSpeed {f32: 5.0});
        app.add_systems(Update, animate_legs);
        app.add_systems(Update, animate_tail);
        app.add_systems(Startup, spawn_cave_tunnel);
        app.add_systems(Startup, insert_crystal_stuff);
        app.add_systems(Update, (spawn_crystals, update_lights).chain());
        app.insert_resource(HurtCounters{total_remaining: 0, flick: 0, should_show: true});
        app.add_systems(FixedPreUpdate, hurt_manager);
        app.add_systems(Startup, setup_audio);
    }
}

#[derive(Component)]
pub struct Player{
    pub velocity: f32
}

#[derive(Resource)]
pub struct PlayerEntity {
    entity: Entity
}

fn spawn_player(
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
    let hip = spawn_legs(
        player,
        &mut commands,
        &mut meshes,
        &mut materials
    );
    let body = spawn_body(
        hip,
        &mut commands,
        &mut meshes,
        &mut materials
    );
    spawn_neck_and_head(
        body,
        &mut commands,
        &mut meshes,
        &mut materials
    );
    commands.insert_resource(PlayerEntity{entity: player});
}

pub const JUMP_V: f32 = 10.0;

fn player_jump_system(
    mut query: Query<(&mut Transform, &mut Player)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut animation_state: ResMut<AnimationState>
) {
    // statics
    let dt = time.delta_secs();
    let g = 40.0 * dt;
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
            p.velocity = JUMP_V;
            animation_state.jumping = true;
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
            animation_state.jumping = false;
        };
    };
}

#[derive(Resource)]
struct ObstacleAssets {
    hex_mesh: Handle<Mesh>,
    wizard: MaterialWizard,
    hues: Vec<f32>
}

#[derive(Resource)]
struct ObstacleRng {
    rng: Rng
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
            lightness, alpha, color_count, 0.5, false,
        )
    };
    let hues: Vec<f32> = {
        let normals = MaterialWizard::generate_normal_hue_vec(COLOR_COUNT);
        let mut hues: Vec<f32> = Vec::with_capacity(COLOR_COUNT);
        for i in 0..COLOR_COUNT {
            hues.push(normals[i] * 360.0);
        };
        hues
    };
    let hex = meshes.add(
        Extrusion::new(
            RegularPolygon::new(0.75, 6), 1.0
        )
    );
    let obs_assets = ObstacleAssets {
        hex_mesh: hex,
        wizard,
        hues
    };
    commands.insert_resource(obs_assets);
    commands.insert_resource(ObstacleTimer{next: time.elapsed_secs() + 1.0, count: 0});
    commands.insert_resource(ObstacleRng{rng: Rng::new()});
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

const OBSTACLE_DELAY: f32 = 2.0;

fn obstacle_spawn_timing(
    time: Res<Time>,
    mut obstacle_timer: ResMut<ObstacleTimer>,
    mut event_writer: EventWriter<SpawnObstacle>,
    mut obstacle_rng: ResMut<ObstacleRng>
) {
    let t = time.elapsed_secs();
    if t >= obstacle_timer.next {
        event_writer.write(SpawnObstacle{count: obstacle_timer.count});
        let r = obstacle_rng.rng.f32() - 0.5;
        obstacle_timer.next = t + OBSTACLE_DELAY + OBSTACLE_DELAY * r;
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
    mut commands: Commands,
    mut obstacle_rng: ResMut<ObstacleRng>,
    obstacle_query: Query<(&Transform, Entity), With<Obstacle>>
) {
    for event in event_reader.read() {
        let i = event.count as usize % COLOR_COUNT;
        commands.spawn(
            (
                Transform::from_xyz(15.0, 0.0, 0.49).with_rotation(Quat::from_rotation_z(obstacle_rng.rng.f32())),
                Mesh3d(assets.hex_mesh.clone()),
                MeshMaterial3d(assets.wizard.get_index(i)),
                Obstacle {
                    radius: 0.75,
                    height: 1.25,
                    scored: false
                },
                NotShadowCaster,
                NotShadowReceiver,
                NoFrustumCulling,
                PointLight {
                    color: Color::hsl(assets.hues[i], 1.0, 0.6),
                    intensity: 32000.0,
                    range: 10.0,
                    shadows_enabled: true,
                    radius: 0.5,
                    shadow_map_near_z: 1.0,
                    ..default()
                }
            )
        );
    };
    for (t, en) in obstacle_query {
        if t.translation.x < -7.0 {
            commands.entity(en).despawn()
        };
    };
}

#[derive(Event)]
struct PlayerHit;

#[derive(Event)]
struct PlayerScores;

#[derive(Resource)]
pub struct LevelSpeed {
    pub f32: f32
}

fn update_obstacles (
    mut transform_query: Query<&mut Transform>,
    mut obstacle_query: Query<(&mut Obstacle, Entity)>,
    p_entity: Res<PlayerEntity>,
    time: Res<Time>,
    speed: Res<LevelSpeed>,
    mut hit_writer: EventWriter<PlayerHit>,
    mut score_writer: EventWriter<PlayerScores>
) {
    let dt = time.delta_secs();
    let motion = dt * speed.f32;
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
        } else if transform.translation.x < -0.0 {
            obstacle.scored = true;
            println!("Score!!");
            score_writer.write(PlayerScores);
        };
    };
}

#[derive(Resource)]
struct HurtCounters {
    total_remaining: u8,
    flick: u8,
    should_show: bool
}

const FLASH_DUR: u8 = 40;
const FLICK_DUR: u8 = 4;

fn hurt_manager(
    mut vis_query: Query<&mut Visibility, With<Player>>,
    mut hurt_counters: ResMut<HurtCounters>,
    mut event_reader: EventReader<PlayerHit>
) {
    let mut v = vis_query.single_mut().unwrap();
    let is_visible = match v.clone() {
        Visibility::Visible => true,
        _ => false
    };
    for _ in event_reader.read() {
        hurt_counters.total_remaining = FLASH_DUR;
    };
    if hurt_counters.total_remaining > 0 {
        if hurt_counters.flick == 0 {
            hurt_counters.flick = FLICK_DUR;
            hurt_counters.should_show = !hurt_counters.should_show;
        } else if hurt_counters.flick > 0 {
            hurt_counters.flick -= 1;
            hurt_counters.total_remaining -= 1;
        };
    } else {
        hurt_counters.should_show = true;
        hurt_counters.flick = 0;
    };
    if is_visible != hurt_counters.should_show {
        v.toggle_visible_hidden();
    };
}