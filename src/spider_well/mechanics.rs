use std::f32::consts::{FRAC_PI_2, TAU};
use bevy::{core_pipeline::{bloom::Bloom, tonemapping::Tonemapping}, prelude::*, render::camera::ScalingMode};

const THREAD_LENGTH_START: f32 = 3.0;
const THREAD_RADIUS: f32 = 0.125;
const STICKING_POINT_RADIUS: f32 = 0.2;
const PLAYER_RADIUS: f32 = 0.5;
const LEVEL_WIDTH: f32 = 20.0;
const GRAVITY: f32 = 0.75;
const LEAP_GRAVITY: f32 = 20.0;
const PLAYER_LEAN: f32 = 0.05;
const PLAYER_CLIMB: f32 = 2.0;
const PLAYER_DRAG: f32 = 0.10;
const STATIC_DRAG: f32 = 1.0;
const ASSUMED_STICKING_POINTS: usize = 10;

#[derive(Component)]
pub struct POVCamera;

pub fn debug_scene_setup(
    mut commands: Commands,
) {
    commands.spawn(
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
            Transform::from_xyz(0.0, 0.0, -LEVEL_WIDTH).looking_at(Vec3::ZERO, Vec3::Y),
            Bloom::OLD_SCHOOL,
            Tonemapping::AcesFitted,
            Msaa::Sample4,
            POVCamera
        )
    );
    commands.insert_resource(AmbientLight{
        color: Color::WHITE,
        brightness: 1000.0,
        ..default()
    });
}

#[derive(Component)]
pub struct PlayerMarker;

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    webbing_assets: Res<WebbingAssets>
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.8, 0.0, 0.0, 1.0),
        unlit: true,
        ..default()
    });
    let mesh = meshes.add(Sphere::new(PLAYER_RADIUS));
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::default(),
        PlayerMarker
    ));
    let thread = commands.spawn((
        Mesh3d(webbing_assets.thread_mesh.clone()),
        MeshMaterial3d(webbing_assets.material.clone()),
        Transform::default(),
        ThreadMarker
    )).id();
    commands.insert_resource(PlayerThread{entity: thread});
}

pub fn insert_simple_resources(
    mut commands: Commands,
) {
    commands.insert_resource(PlayerPos{vec: Vec2::ZERO});
    commands.insert_resource(PlayerSwing{thread_length: THREAD_LENGTH_START, angle: 0.0, angular_v: 0.0});
    commands.insert_resource(StickingPoints{vec: {
        let mut v = Vec::with_capacity(ASSUMED_STICKING_POINTS);
        v.push(Vec2::new(0.0, THREAD_LENGTH_START));
        v
    }});
    commands.insert_resource(StickingSpheres{vec: Vec::with_capacity(ASSUMED_STICKING_POINTS)});
    commands.insert_resource(StickingThreads{vec: Vec::with_capacity(ASSUMED_STICKING_POINTS-1)});
    commands.insert_resource(PlayerInputs{
        x: 0, y: 0, leaping: false, scheme: ControlScheme::Wasd
    });
    commands.insert_resource(PlayerVelocity {vec: Vec2::ZERO});
    commands.insert_resource(UpdateStaticThreads{bool: true});
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ControlScheme {
    Wasd,
    Arrows,
    Numpad,
    None
}

#[derive(Resource)]
pub struct PlayerInputs {
    x: i8,
    y: i8,
    leaping: bool,
    scheme: ControlScheme
}

pub fn player_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_inputs: ResMut<PlayerInputs>
) {
    let mut x = 0;
    let mut y = 0;
    let mut leaping = false;
    let mut scheme = ControlScheme::None;
    let pressed = keys.get_pressed();
    for key in pressed {
        match key {
            KeyCode::KeyW => {y+=1; scheme=ControlScheme::Wasd;},
            KeyCode::KeyA => {x-=1; scheme=ControlScheme::Wasd;},
            KeyCode::KeyS => {y-=1; scheme=ControlScheme::Wasd;},
            KeyCode::KeyD => {x+=1; scheme=ControlScheme::Wasd;},
            KeyCode::ArrowUp => {y+=1; scheme=ControlScheme::Arrows;},
            KeyCode::ArrowLeft => {x-=1; scheme=ControlScheme::Arrows;},
            KeyCode::ArrowDown => {y-=1; scheme=ControlScheme::Arrows;},
            KeyCode::ArrowRight => {x+=1; scheme=ControlScheme::Arrows;},
            KeyCode::Numpad8 => {y+=1; scheme=ControlScheme::Numpad;},
            KeyCode::Numpad4 => {x-=1; scheme=ControlScheme::Numpad;},
            KeyCode::Numpad5 => {y-=1; scheme=ControlScheme::Numpad;},
            KeyCode::Numpad6 => {x+=1; scheme=ControlScheme::Numpad;},
            KeyCode::Space => {leaping=true; break;},
            KeyCode::Numpad0 => {leaping=true; break;},
            KeyCode::Insert => {leaping=true; break;},
            _ => {}
        }
    };
    player_inputs.leaping = leaping;
    if leaping {
        return;
    };
    player_inputs.x = x;
    player_inputs.y = y;
    if scheme != ControlScheme::None {
        player_inputs.scheme = scheme;
    };
}

#[derive(Resource)]
pub struct StickingPoints {
    vec: Vec<Vec2>
}

#[derive(Resource)]
pub struct PlayerSwing {
    thread_length: f32,
    angle: f32,
    angular_v: f32
}

#[derive(Resource)]
pub struct PlayerVelocity {
    vec: Vec2
}

#[derive(Resource)]
pub struct PlayerPos {
    vec: Vec2
}

pub fn move_player(
    player_inputs: Res<PlayerInputs>,
    time: Res<Time>,
    sticking_points: Res<StickingPoints>,
    mut player_swing: ResMut<PlayerSwing>,
    mut player_velocity: ResMut<PlayerVelocity>,
    mut player_pos: ResMut<PlayerPos>,
    mut query: Query<&mut Transform, With<PlayerMarker>>
) {
    // resource variables
    let dt = time.delta_secs();
    let player_transform = &mut query.single_mut().unwrap();
    let &stuck = sticking_points.vec.last().unwrap();

    // leaping logic
    if player_inputs.leaping {
        player_velocity.vec.y -= LEAP_GRAVITY * dt;
        let d = player_velocity.vec * dt * PLAYER_DRAG;
        player_velocity.vec -= d;
        player_pos.vec += player_velocity.vec * dt;
        player_swing.thread_length = player_pos.vec.distance(stuck);
        let delta_vec = stuck - player_pos.vec;
        player_swing.angle = -delta_vec.x.atan2(delta_vec.y);
        player_transform.translation = player_pos.vec.extend(0.0);
        return;  // TODO: improve the way leaping velocity is added to swinging velocity
    };

    // rope-crawl
    if player_inputs.y > 0 {
        let l = player_swing.thread_length - PLAYER_CLIMB * dt;
        player_swing.angular_v *= player_swing.thread_length / l;
        player_swing.thread_length = l;
    } else if player_inputs.y < 0 {
        let l = player_swing.thread_length + PLAYER_CLIMB * dt;
        player_swing.angular_v *= player_swing.thread_length / l;
        player_swing.thread_length = l;
    };

    let circumference = TAU * player_swing.thread_length;

    // input handling
    let d = (PLAYER_LEAN * dt * player_swing.angle.cos()) / player_swing.thread_length;
    if player_inputs.x > 0 {
        player_swing.angular_v -= d;
        player_swing.angular_v -= player_swing.angular_v * dt * PLAYER_DRAG;
    } else if player_inputs.x < 0 {
        player_swing.angular_v += d;
        player_swing.angular_v -= player_swing.angular_v * dt * PLAYER_DRAG;
    } else {
        player_swing.angular_v -= player_swing.angular_v * dt * STATIC_DRAG;
    };
    
    // apply gravity to velocity, then apply angular velocity
    let a_accel = -(GRAVITY / player_swing.thread_length) * player_swing.angle.sin() * dt;
    player_swing.angular_v += a_accel;
    player_swing.angle += player_swing.angular_v;

    // update position
    let offset = Vec2 {
        x: player_swing.angle.sin() * player_swing.thread_length,
        y: -player_swing.angle.cos() * player_swing.thread_length
    };
    player_pos.vec = stuck + offset;

    // update velocity for flinging
    player_velocity.vec = (player_pos.vec - player_transform.translation.xy()) / dt;
    
    //update translation
    player_transform.translation = player_pos.vec.extend(0.0);
}

#[derive(Resource)]
pub struct WebbingAssets {
    material: Handle<StandardMaterial>,
    thread_mesh: Handle<Mesh>,
    sticky_mesh: Handle<Mesh>
} impl WebbingAssets {
    pub fn new(
        material: Handle<StandardMaterial>, thread_mesh: Handle<Mesh>, sticky_mesh: Handle<Mesh>
    ) -> Self {
        Self {material, thread_mesh, sticky_mesh}
    }
    pub fn get_material(&self) -> Handle<StandardMaterial> {
        self.material.clone()
    }
    pub fn get_thread(&self) -> Handle<Mesh> {
        self.thread_mesh.clone()
    }
    pub fn get_sticky(&self) -> Handle<Mesh> {
        self.thread_mesh.clone()
    }
}

pub fn insert_webbing_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.0,
        reflectance: 1.0,
        clearcoat: 0.0,
        clearcoat_perceptual_roughness: 0.0,
        anisotropy_rotation: 0.0,
        anisotropy_strength: 1.0,
        ..default()
    });
    let thread_mesh = meshes.add(Cylinder::new(THREAD_RADIUS, 1.0));
    let sticky_mesh = meshes.add(Sphere::new(STICKING_POINT_RADIUS));
    commands.insert_resource(WebbingAssets::new(material, thread_mesh, sticky_mesh));
}

#[derive(Resource)]
pub struct StickingSpheres {
    vec: Vec<Entity>
}

#[derive(Resource)]
pub struct StickingThreads {
    vec: Vec<Entity>
}

#[derive(Component)]
pub struct ThreadMarker;

#[derive(Component)]
pub struct StickingPointMarker;

pub fn web_spawner(
    sticking_points: Res<StickingPoints>,
    webbing_assets: Res<WebbingAssets>,
    mut commands: Commands,
    mut sticking_spheres: ResMut<StickingSpheres>,
    mut sticking_threads: ResMut<StickingThreads>,
    mut update_threads: ResMut<UpdateStaticThreads>
) {
    while sticking_points.vec.len() > sticking_spheres.vec.len() {
        let i = sticking_spheres.vec.len();
        let point = sticking_points.vec[i];
        let entity = commands.spawn((
            Mesh3d(webbing_assets.sticky_mesh.clone()),
            MeshMaterial3d(webbing_assets.material.clone()),
            Transform::from_translation(point.extend(0.0)),
            StickingPointMarker
        )).id();
        sticking_spheres.vec.push(entity);
        if sticking_spheres.vec.len() > 1 {
            let entity = commands.spawn((
                Mesh3d(webbing_assets.thread_mesh.clone()),
                MeshMaterial3d(webbing_assets.material.clone()),
                Transform::default(),
                ThreadMarker
            )).id();
            sticking_threads.vec.push(entity);
            update_threads.bool = true;
        };
    };
    while sticking_points.vec.len() < sticking_spheres.vec.len() {
        let entity = sticking_spheres.vec.pop().unwrap();
        commands.entity(entity).despawn();
        let entity = sticking_threads.vec.pop().unwrap();
        commands.entity(entity).despawn();
        update_threads.bool = true;
    };
}

#[derive(Resource)]
pub struct UpdateStaticThreads {
    bool: bool
}

#[derive(Resource)]
pub struct PlayerThread {
    entity: Entity
}

pub fn web_updater(
    sticking_points: Res<StickingPoints>,
    threads: Res<StickingThreads>,
    player_thread: Res<PlayerThread>,
    player_pos: Res<PlayerPos>,
    player_swing: Res<PlayerSwing>,
    update_threads: Res<UpdateStaticThreads>,
    mut query: Query<&mut Transform, With<ThreadMarker>>
) {
    let mut player_thread_transform = query.get_mut(player_thread.entity).unwrap();
    let average_pos = (player_pos.vec + sticking_points.vec.last().unwrap()) / 2.0;
    player_thread_transform.translation = average_pos.extend(0.0);
    player_thread_transform.scale.y = player_swing.thread_length;
    player_thread_transform.rotation = Quat::from_rotation_z(player_swing.angle);
    if update_threads.bool {
        
    };
}