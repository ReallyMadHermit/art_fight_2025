use std::f32::consts::{TAU, FRAC_PI_2};
use bevy::{core_pipeline::{bloom::Bloom, tonemapping::Tonemapping}, prelude::*, render::camera::ScalingMode};
use crate::common::RectChecks;
use crate::spider_well::level_layout::{LEVEL_WIDTH, DAMSEL_Y};

const THREAD_LENGTH_START: f32 = 3.0;
const THREAD_RADIUS: f32 = 0.06125;
const STICKING_POINT_RADIUS: f32 = 0.2;
const PLAYER_RADIUS: f32 = 0.5;
const GRAVITY: f32 = 0.625;
const LEAP_GRAVITY: f32 = 20.0;
const PLAYER_LEAN: f32 = 0.05;
const PLAYER_CLIMB: f32 = 2.0;
const PLAYER_DRAG: f32 = 0.10;
const STATIC_DRAG: f32 = 1.0;
const PLATFORM_THICKNESS: f32 = 0.125;
const CAMERA_RATE: f32 = 3.0;
const HANG_POINT: Vec2 = Vec2{ x: 0.0, y: THREAD_LENGTH_START};
const HURT_RETURN_TIME: f32 = 0.5;

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
            Transform::from_xyz(0.0, -3.0, LEVEL_WIDTH)
                .looking_at(Vec3::new(0.0, -3.0, 0.0), Vec3::Y),
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
pub struct POVCamera;

pub fn camera_mover(
    player_swing: Res<PlayerSwing>,
    mut camera_query: Query<&mut Transform, With<POVCamera>>,
    time: Res<Time>
) {
    let y = HANG_POINT.y - player_swing.thread_length;
    let mut camera_transform = camera_query.single_mut().unwrap();
    let delta = y - camera_transform.translation.y;
    camera_transform.translation.y += delta * time.delta_secs() * CAMERA_RATE;
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
    commands.spawn((
        Mesh3d(webbing_assets.thread_mesh.clone()),
        MeshMaterial3d(webbing_assets.material.clone()),
        Transform::default(),
        ThreadMarker
    ));
}

pub fn insert_simple_resources(
    mut commands: Commands,
) {
    commands.insert_resource(PlayerPos{vec: Vec2::ZERO});
    commands.insert_resource(PlayerSwing{thread_length: THREAD_LENGTH_START, angle: 0.0, angular_v: 0.0});
    commands.insert_resource(PlayerInputs{
        x: 0, y: 0, leaping: false, scheme: ControlScheme::Wasd
    });
    commands.insert_resource(PlayerVelocity {vec: Vec2::ZERO});
    commands.insert_resource(LastCheckPoint {pos: Vec2::ZERO, hurt_pos: Vec2::ZERO});
    commands.insert_resource(HurtReturn {f32: 0.0});
    commands.insert_resource(DamselAcquired{bool: false});
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
            KeyCode::KeyA => {x+=1; scheme=ControlScheme::Wasd;},
            KeyCode::KeyS => {y-=1; scheme=ControlScheme::Wasd;},
            KeyCode::KeyD => {x-=1; scheme=ControlScheme::Wasd;},
            KeyCode::ArrowUp => {y+=1; scheme=ControlScheme::Arrows;},
            KeyCode::ArrowLeft => {x+=1; scheme=ControlScheme::Arrows;},
            KeyCode::ArrowDown => {y-=1; scheme=ControlScheme::Arrows;},
            KeyCode::ArrowRight => {x-=1; scheme=ControlScheme::Arrows;},
            KeyCode::Numpad8 => {y+=1; scheme=ControlScheme::Numpad;},
            KeyCode::Numpad4 => {x+=1; scheme=ControlScheme::Numpad;},
            KeyCode::Numpad5 => {y-=1; scheme=ControlScheme::Numpad;},
            KeyCode::Numpad6 => {x-=1; scheme=ControlScheme::Numpad;},
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

// TODO: make the swings slower and less dramatic so they don't skip collisions
pub fn move_player(
    player_inputs: Res<PlayerInputs>,
    time: Res<Time>,
    mut player_swing: ResMut<PlayerSwing>,
    mut player_velocity: ResMut<PlayerVelocity>,
    mut player_pos: ResMut<PlayerPos>,
    mut query: Query<&mut Transform, With<PlayerMarker>>,
    checkpoint: Res<LastCheckPoint>,
    mut hurt_return: ResMut<HurtReturn>
) {
    // resource variables
    let dt = time.delta_secs();
    let player_transform = &mut query.single_mut().unwrap();
    
    // hurt return logic
    if hurt_return.f32 > 0.0 {
        hurt_return.f32 -= dt;
        if hurt_return.f32 <= 0.0 {
            hurt_return.f32 = 0.0;
            player_transform.translation = checkpoint.pos.extend(0.0);
            player_pos.vec = checkpoint.pos;
            player_swing.angle = 0.0;
        } else {
            let n = hurt_return.f32 / HURT_RETURN_TIME;
            let n1 = 1.0 - n;
            let x = checkpoint.hurt_pos.x * n + checkpoint.pos.x * n1;
            let y = checkpoint.hurt_pos.y * n + checkpoint.pos.y * n1;
            player_transform.translation.x = x;
            player_transform.translation.y = y;
            player_pos.vec = Vec2::new(x, y);
        };
        player_swing.thread_length = player_pos.vec.distance(HANG_POINT);
        player_swing.angular_v = 0.0;
        return;
    };

    // leaping logic
    if player_inputs.leaping {
        // apply velocity
        player_velocity.vec.y -= LEAP_GRAVITY * dt;
        let d = player_velocity.vec * dt * PLAYER_DRAG;
        player_velocity.vec -= d;
        player_pos.vec += player_velocity.vec * dt;
        player_transform.translation = player_pos.vec.extend(0.0);
        // update player swing
        player_swing.thread_length = player_pos.vec.distance(HANG_POINT);
        let delta_vec = HANG_POINT - player_pos.vec;
        player_swing.angle = -delta_vec.x.atan2(delta_vec.y);
        // update angular velocity
        let acos = player_swing.angle.cos();
        let asin = player_swing.angle.sin();
        let xv_a = (player_velocity.vec.x * acos) / (player_swing.thread_length.powi(2) * TAU);
        let yv_a = (player_velocity.vec.y * asin) / (player_swing.thread_length.powi(2) * TAU);
        player_swing.angular_v = xv_a + yv_a;
        return;
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
    player_pos.vec = HANG_POINT + offset;

    // update velocity for flinging
    player_velocity.vec = (player_pos.vec - player_transform.translation.xy()) / dt;
    
    //update translation
    player_transform.translation = player_pos.vec.extend(0.0);
}

#[derive(Resource)]
pub struct WebbingAssets {
    material: Handle<StandardMaterial>,
    thread_mesh: Handle<Mesh>,
    sphere_mesh: Handle<Mesh>
} impl WebbingAssets {
    pub fn new(
        material: Handle<StandardMaterial>, thread_mesh: Handle<Mesh>, sphere_mesh: Handle<Mesh>
    ) -> Self {
        Self {material, thread_mesh, sphere_mesh }
    }
    pub fn get_material(&self) -> Handle<StandardMaterial> {
        self.material.clone()
    }
    pub fn get_thread(&self) -> Handle<Mesh> {
        self.thread_mesh.clone()
    }
    pub fn get_sphere(&self) -> Handle<Mesh> {
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

#[derive(Component)]
pub struct ThreadMarker;

pub fn web_updater(
    player_pos: Res<PlayerPos>,
    player_swing: Res<PlayerSwing>,
    mut query: Query<&mut Transform, With<ThreadMarker>>
) {
    let mut player_thread_transform = query.single_mut().unwrap();
    let average_pos = (player_pos.vec + HANG_POINT) / 2.0;
    player_thread_transform.translation = average_pos.extend(0.0);
    player_thread_transform.scale.y = player_swing.thread_length;
    player_thread_transform.rotation = Quat::from_rotation_z(player_swing.angle);
}

pub fn spawn_some_holes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let left_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 0.0, 1.0),
        ..default()
    });
    let right_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.0, 0.0),
        ..default()
    });
    let holes = [
        (0.0, -1.0, 4.0),
        (2.0, -5.0, 4.0),
        (-2.0, -9.0, 4.0)
    ];
    let x_max = LEVEL_WIDTH / 2.0;
    let x_min = -x_max;
    for (x, y, w) in holes {
        let left_edge = x - (w/2.0);
        let right_edge = x + (w/2.0);
        let left_center = (left_edge + x_min) / 2.0;
        let right_center = (right_edge + x_max) / 2.0;
        let left_length = (left_edge - x_min).abs();
        let right_length = (right_edge - x_max).abs();
        commands.spawn((
            MeshMaterial3d(left_mat.clone()),
            Mesh3d(meshes.add(Cuboid::new(left_length, PLATFORM_THICKNESS, 1.0))),
            Transform::from_xyz(left_center, y, 0.0),
            CollisionRect::new(left_length, PLATFORM_THICKNESS)
        ));
        commands.spawn((
            MeshMaterial3d(right_mat.clone()),
            Mesh3d(meshes.add(Cuboid::new(right_length, PLATFORM_THICKNESS, 1.0))),
            Transform::from_xyz(right_center, y, 0.0),
            CollisionRect::new(right_length, PLATFORM_THICKNESS)
        ));
    };
}

#[derive(Event)]
pub struct CollisionEvent;

#[derive(Component)]
pub struct CollisionRect{
    collision_radi: Vec2
} impl CollisionRect {
    pub fn new(width: f32, height: f32) -> Self {
        Self {collision_radi: Vec2::new(width/2.0, height/2.0) + PLAYER_RADIUS}
    }
}

#[derive(Resource)]
pub struct LastCheckPoint {
    pos: Vec2,
    hurt_pos: Vec2
}

#[derive(Resource)]
pub struct HurtReturn {
    f32: f32
}

pub fn collision_rect_checker(
    player_pos: Res<PlayerPos>,
    collision_rects: Query<(&CollisionRect, &Transform)>,
    mut event_writer: EventWriter<CollisionEvent>,
    mut hurt_return: ResMut<HurtReturn>,
    mut checkpoint: ResMut<LastCheckPoint>
) {
    if hurt_return.f32 > 0.0 {
        return;
    };
    let mut hurting = false;
    if player_pos.vec.x.abs() - PLAYER_RADIUS > LEVEL_WIDTH / 2.0 {
        hurting = true;
    }
    for (rect, transform) in collision_rects {
        if hurting {
            break;
        };
        if RectChecks::is_inside_y_first(rect.collision_radi, transform.translation.xy(), player_pos.vec) {
            hurting = true;
        };
    };
    if hurting {
        event_writer.write(CollisionEvent);
        hurt_return.f32 = HURT_RETURN_TIME;
        checkpoint.hurt_pos = player_pos.vec;
    };
}

#[derive(Component)]
pub struct ObstaclePathing {
    waypoints: Vec<(Vec2, f32)>,
    progress: f32,
    vec_size: i8,
    vec_step: i8
} impl ObstaclePathing {

    pub fn new(waypoints: Vec<(Vec2, f32)>) -> Self {
        let n = waypoints.len();
        Self {
            waypoints,
            progress: 0.0,
            vec_step: 0,
            vec_size: n as i8
        }
    }

    pub fn uniform_timing(travel_points: Vec<Vec2>, step_duration: f32) -> Self {
        let mut waypoints: Vec<(Vec2, f32)> = Vec::with_capacity(travel_points.len());
        for point in travel_points {
            waypoints.push((point, step_duration));
        };
        Self::new(waypoints)
    }

    pub fn update(&mut self, dt: f32) {
        self.progress += dt;
        let duration = self.get_duration();
        if self.progress >= duration {
            self.progress -= duration;
            self.vec_step = self.get_step(1);
        };
    }

    pub fn get_vec(&self) -> Vec2 {
        let (goal, duration) = self.waypoints[self.vec_step as usize];
        let last_stop = self.waypoints[self.get_step(-1) as usize].0;
        let n = self.progress / duration;
        let nm = 1.0 - n;
        last_stop * nm + goal * n
    }

    fn get_duration(&self) -> f32 {
        self.waypoints[self.vec_step as usize].1
    }

    fn get_step(&self, step_mod: i8) -> i8 {
        let n = self.vec_step + step_mod;
        if n >= self.vec_size {
            0
        } else if n < 0 {
            self.vec_size - 1
        } else {
            n
        }
    }

}

pub fn move_obstacles(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ObstaclePathing)>
) {
    let dt = time.delta_secs();
    for (mut transform, mut obstacle) in &mut query {
        obstacle.update(dt);
        transform.translation = obstacle.get_vec().extend(0.0)
    };
}

pub fn spawn_some_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let mesh = meshes.add(Cuboid::from_length(1.0));
    let material = materials.add(StandardMaterial{
        base_color: Color::linear_rgb(0.0, 1.0, 0.0),
        ..default()
    });
    let y_base = -9.5f32;
    let x_range = 5.0f32;
    for i in 0..5 {
        let mut ways: Vec<Vec2> = Vec::with_capacity(2);
        ways.push(Vec2::new(-x_range, y_base - i as f32));
        ways.push(Vec2::new(x_range, y_base - i as f32));
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            CollisionRect::new(1.0, 1.0),
            ObstaclePathing::uniform_timing(ways, i as f32 + 1.0)
        ));
    }
}

#[derive(Component)]
pub struct CheckPoint {
    y: f32,
    checked: bool
} impl CheckPoint {
    pub fn new(y: f32) -> Self {
        Self {y, checked: false}
    }
}

pub fn spawn_checkpoint(
    y: f32,
    commands: &mut Commands
) {
    commands.spawn(CheckPoint::new(y));
}

pub fn checkpoint_checker(
    player_pos: Res<PlayerPos>,
    mut last_check_point: ResMut<LastCheckPoint>,
    mut query: Query<&mut CheckPoint>
) {
    let py = player_pos.vec.y;
    for mut checkpoint in &mut query {
        if checkpoint.checked {
            continue;
        } else if py < checkpoint.y {
            println!("bing bing, check point!!");
            last_check_point.pos.y = checkpoint.y;
            checkpoint.checked = true;
        };
    };
}

#[derive(Resource)]
pub struct DamselAcquired {
    bool: bool
}

pub fn damsel_checker(
    player_pos: Res<PlayerPos>,
    mut damsel_acquired: ResMut<DamselAcquired>,
) {
    if !damsel_acquired.bool && player_pos.vec.y < DAMSEL_Y {
        damsel_acquired.bool = true;
        println!("Damsel acquired!");
    };
}
