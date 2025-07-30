use std::f32::consts::TAU;
use bevy::prelude::*;
use crate::common::RectChecks;
use crate::spider_well::environment::SpiritsAcquired;
use crate::spider_well::level_layout::LEVEL_WIDTH;

const THREAD_LENGTH_START: f32 = 2.0;
const THREAD_RADIUS: f32 = 0.0125;
const STICKING_POINT_RADIUS: f32 = 0.2;
pub const PLAYER_RADIUS: f32 = 0.45;
const GRAVITY: f32 = 0.625;
const LEAP_GRAVITY: f32 = 20.0;
const PLAYER_LEAN: f32 = 0.05;
pub const PLAYER_CLIMB: f32 = 2.0;
const PLAYER_DRAG: f32 = 0.10;
const STATIC_DRAG: f32 = 1.0;
const CAMERA_RATE: f32 = 3.0;
const HANG_POINT: Vec2 = Vec2{ x: 0.0, y: THREAD_LENGTH_START + 1.0};
const HURT_RETURN_TIME: f32 = 1.6;

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

#[derive(Resource)]
pub struct PlayerEntity{
    pub entity: Entity
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    webbing_assets: Res<WebbingAssets>
) {
    let entity = commands.spawn((
        // Mesh3d(mesh),
        // MeshMaterial3d(mat),
        Transform::from_xyz(0.0, -2.0, 0.0),
        PlayerMarker
    )).id();
    commands.spawn((
        Mesh3d(webbing_assets.thread_mesh.clone()),
        MeshMaterial3d(webbing_assets.material.clone()),
        Transform::default(),
        ThreadMarker
    ));
    commands.insert_resource(PlayerEntity{entity});
}

pub fn insert_simple_resources(
    mut commands: Commands,
) {
    commands.insert_resource(PlayerPos{vec: Vec2::ZERO});
    commands.insert_resource(PlayerSwing{thread_length: THREAD_LENGTH_START, angle: 0.0, angular_v: 0.0});
    commands.insert_resource(PlayerInputs{
        x: 0, y: 0, leaping: false,
    });
    commands.insert_resource(PlayerVelocity {vec: Vec2::ZERO});
    commands.insert_resource(LastCheckPoint {pos: Vec2::ZERO, hurt_pos: Vec2::ZERO});
    commands.insert_resource(HurtReturn {f32: 0.0});
    commands.insert_resource(IsIdle{bool: true});
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ControlScheme {
    Wasd,
    Arrows,
    Numpad,
    None
}

#[derive(Resource)]
pub struct IsIdle {
    pub bool: bool
}

#[derive(Resource)]
pub struct PlayerInputs {
    pub x: i8,
    pub y: i8,
    pub leaping: bool
}

pub fn player_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_inputs: ResMut<PlayerInputs>
) {
    let mut x = 0;
    let mut y = 0;
    let mut leaping = false;
    let pressed = keys.get_pressed();
    for key in pressed {
        match key {
            KeyCode::KeyW => {y+=1;},
            KeyCode::KeyA => {x+=1;},
            KeyCode::KeyS => {y-=1;},
            KeyCode::KeyD => {x-=1;},
            KeyCode::ArrowUp => {y+=1;},
            KeyCode::ArrowLeft => {x+=1;},
            KeyCode::ArrowDown => {y-=1;},
            KeyCode::ArrowRight => {x-=1;},
            KeyCode::Numpad8 => {y+=1;},
            KeyCode::Numpad4 => {x+=1;},
            KeyCode::Numpad5 => {y-=1;},
            KeyCode::Numpad6 => {x-=1;},
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
}

#[derive(Resource)]
pub struct PlayerSwing {
    pub thread_length: f32,
    pub angle: f32,
    pub angular_v: f32
}

#[derive(Resource)]
pub struct PlayerVelocity {
    vec: Vec2
}

#[derive(Resource)]
pub struct PlayerPos {
    pub vec: Vec2
}

pub fn move_player(
    player_inputs: Res<PlayerInputs>,
    time: Res<Time>,
    mut player_swing: ResMut<PlayerSwing>,
    mut player_velocity: ResMut<PlayerVelocity>,
    mut player_pos: ResMut<PlayerPos>,
    mut query: Query<&mut Transform, With<PlayerMarker>>,
    checkpoint: Res<LastCheckPoint>,
    mut hurt_return: ResMut<HurtReturn>,
    mut is_idle: ResMut<IsIdle>
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
        is_idle.bool = false;
        // {
        //     let x = player_pos.vec.x;
        //     let y = player_pos.vec.y;
        //     println!("{}, {}", x, y);
        // };
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
        player_transform.rotation = Quat::from_rotation_z(player_swing.angle);
        return;
    };

    // rope-crawl input handling
    if player_inputs.y > 0 {
        let l = (player_swing.thread_length - PLAYER_CLIMB * dt).max(1.0);
        player_swing.angular_v *= player_swing.thread_length / l;
        player_swing.thread_length = l;
        is_idle.bool = false;
    } else if player_inputs.y < 0 {
        let l = player_swing.thread_length + PLAYER_CLIMB * dt;
        player_swing.angular_v *= player_swing.thread_length / l;
        player_swing.thread_length = l;
        is_idle.bool = false;
    };

    // swing input handling
    let d = (PLAYER_LEAN * dt * player_swing.angle.cos()) / player_swing.thread_length;
    if player_inputs.x > 0 {
        player_swing.angular_v -= d;
        player_swing.angular_v -= player_swing.angular_v * dt * PLAYER_DRAG;
        is_idle.bool = false;
    } else if player_inputs.x < 0 {
        player_swing.angular_v += d;
        player_swing.angular_v -= player_swing.angular_v * dt * PLAYER_DRAG;
        is_idle.bool = false;
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
    
    //update transform
    player_transform.translation = player_pos.vec.extend(0.0);
    if is_idle.bool {
        player_transform.rotation = Quat::from_rotation_y(time.elapsed_secs());
    } else {
        player_transform.rotation = Quat::from_rotation_z(player_swing.angle);
    };
    
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
}

pub fn insert_webbing_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        // perceptual_roughness: 0.0,
        // reflectance: 1.0,
        // clearcoat: 0.0,
        // clearcoat_perceptual_roughness: 0.0,
        // anisotropy_rotation: 0.0,
        // anisotropy_strength: 1.0,
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

#[derive(Event)]
pub struct CollisionEvent;

#[derive(Component)]
pub struct CollisionRect{
    pub collision_radi: Vec2
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

#[derive(Event)]
pub struct CheckPointEvent;

pub fn checkpoint_checker(
    player_pos: Res<PlayerPos>,
    mut last_check_point: ResMut<LastCheckPoint>,
    mut query: Query<&mut CheckPoint>,
    is_ridle: Res<IsIdle>,
    mut checkpoint_writer: EventWriter<CheckPointEvent>
) {
    if is_ridle.bool {
        return;
    };
    let py = player_pos.vec.y;
    for mut checkpoint in &mut query {
        if checkpoint.checked {
            continue;
        } else if py < checkpoint.y {
            checkpoint_writer.write(CheckPointEvent);
            last_check_point.pos.y = checkpoint.y;
            checkpoint.checked = true;
        };
    };
}

pub fn insert_state_resources(
    mut commands: Commands,
) {
    
    let mut s = String::with_capacity(6);
    s = "000".to_string();
    commands.insert_resource(SpeedRunTimer{start: 0.0, running: false, string: s});
    commands.insert_resource(ImWinningDad{bool: false});
    commands.insert_resource(ResetTimer{held: 0.0});
}

#[derive(Resource)]
pub struct ImWinningDad{
    pub bool: bool
}

pub fn are_ya_winning_son(
    spirits_acquired: Res<SpiritsAcquired>,
    player_pos: Res<PlayerPos>,
    mut im_winning: ResMut<ImWinningDad>,
    mut checkpoint_writer: EventWriter<CheckPointEvent>
) {
    if !im_winning.bool && spirits_acquired.bool && player_pos.vec.y > 0.0 {
        im_winning.bool = true;
        checkpoint_writer.write(CheckPointEvent);
    };
}

#[derive(Resource)]
pub struct SpeedRunTimer {
    start: f32,
    running: bool,
    pub string: String
}

pub fn speed_run_timer(
    time: Res<Time>,
    is_idle: Res<IsIdle>,
    im_winning_dad: Res<ImWinningDad>,
    mut speed_run_timer: ResMut<SpeedRunTimer>
) {
    if !speed_run_timer.running && !im_winning_dad.bool && !is_idle.bool {  // if not idle and not running, start the speedrun timer
        speed_run_timer.running = true;
        speed_run_timer.start = time.elapsed_secs();
    } else if im_winning_dad.bool && speed_run_timer.running {  // if winning, stop the timer
        let t = time.elapsed_secs() - speed_run_timer.start;
        speed_run_timer.string = format!("{}", t as i32);
        speed_run_timer.running = false;
    } else if speed_run_timer.running {  // if running, update the timer
        let t = time.elapsed_secs() - speed_run_timer.start;
        speed_run_timer.string = format!("{}", t as i32);
    };
    // println!("{}", speed_run_timer.string);
}

#[derive(Resource)]
pub struct ResetTimer {
    held: f32
}

#[derive(Event)]
pub struct ResetEvent;

pub fn are_we_resetting(
    mut reset_timer: ResMut<ResetTimer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut reset_writer: EventWriter<ResetEvent>,
    time: Res<Time>
) {
    if keys.pressed(KeyCode::KeyR) {
        reset_timer.held += time.delta_secs()
    } else {
        reset_timer.held = 0.0
    };
    if reset_timer.held > 2.0 {
        reset_timer.held = 0.0;
        reset_writer.write(ResetEvent);
    };
}

pub fn we_are_indeed_resetting(
    mut event_reader: EventReader<ResetEvent>,
    mut hurt_return: ResMut<HurtReturn>,
    mut last_check_point: ResMut<LastCheckPoint>,
    mut query: Query<&mut CheckPoint>,
    mut spirits_acquired: ResMut<SpiritsAcquired>,
    mut im_winning_dad: ResMut<ImWinningDad>,
    mut speed_run_timer: ResMut<SpeedRunTimer>,
    mut is_idle: ResMut<IsIdle>,
    player_pos: Res<PlayerPos>
) {
    for _ in event_reader.read() {
        // last checkpoint & hurt return
        last_check_point.pos = Vec2::new(0.0, 1.0);
        last_check_point.hurt_pos = player_pos.vec;
        hurt_return.f32 = HURT_RETURN_TIME;
        // reset checkpoints
        for mut checkpoint in &mut query {
            checkpoint.checked = false;
        }
        // bools
        spirits_acquired.bool = false;
        im_winning_dad.bool = false;
        is_idle.bool = true;
        
        // reset timer
        speed_run_timer.running = false;
        speed_run_timer.string = "000".to_string();
        speed_run_timer.start = 0.0;
    }
}
