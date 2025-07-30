use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI, TAU};
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::color::palettes::css;
use crate::segmented_displays::{spawn_segmented_string, SegmentedDisplayAssets, SegmentedDisplayString};
use crate::spider_well::level_layout::{LEVEL_WIDTH, SlidingBlocks2, DAMSEL_Y};
use crate::spider_well::mechanics::{POVCamera, PlayerEntity, PlayerPos, IsIdle, SpeedRunTimer, ImWinningDad};

const FOV: f32 = PI / 8.0;

const LIGHT_Z: f32 = (LEVEL_DEPTH / 2.0) + 0.5;
const LIGHT_COLOR: Color = Color::linear_rgb(1.0, 0.8, 0.4);
const BULB_Z: f32 = -(LEVEL_DEPTH / 2.0) + 0.25;
pub const LEVEL_DEPTH: f32 = 2.0;
pub const ORB_RADIUS: f32 = 0.05;
pub const ORB_ORBIT_RADIUS: f32 = 0.35;
pub const ORB_A: f32 = TAU / 3.0;
pub const ORB_SPIN_SPEED: f32 = FRAC_PI_2;

const BULB_RADIUS: f32 = 0.25;
const LIGHT_POST_DISTANCE: f32 = 2.0;
const LIGHT_POST_LENGTH: f32 = 6.0;
const LIGHT_POST_Z: f32 = -1.0;
const LIGHT_POST_RADIUS: f32 = BULB_RADIUS;

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
    // let camera = commands.spawn(
    //     (
    //         Camera3d::default(),
    //         Camera {
    //             hdr: true,
    //             ..default()
    //         },
    //         Projection::Perspective(
    //             PerspectiveProjection{
    //                 fov: FOV,
    //                 ..default()
    //             }
    //         ),
    //         Transform::from_xyz(0.0, -3.0, LEVEL_WIDTH)
    //             .looking_at(Vec3::new(0.0, -3.0, 0.0), Vec3::Y),
    //         Bloom::OLD_SCHOOL,
    //         Tonemapping::AcesFitted,
    //         Msaa::Sample4,
    //         POVCamera
    //     )
    // ).id();
    let timer = spawn_timer(&mut commands, &mut meshes, &mut materials);
    commands.entity(timer).insert(ChildOf(camera));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(LEVEL_WIDTH, LEVEL_WIDTH, 0.0125))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.5, 0.5, 0.5),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -LEVEL_WIDTH - (LEVEL_DEPTH / 2.0)),
        ChildOf(camera),
        NotShadowCaster
    ));

}

pub fn spawn_first_lamp(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // commands.insert_resource(AmbientLight{
    //     color: Color::WHITE,
    //     brightness: 100.0,
    //     ..default()
    // });
    commands.insert_resource(AmbientLight{
        color: Color::BLACK,
        brightness: 0.0,
        ..default()
    });
    commands.insert_resource(ClearColor(Color::BLACK));
    // for vec in &gaps.vec {
    //     commands.spawn((
    //         PointLight {
    //             color: LIGHT_COLOR,
    //             intensity: 20000.0,
    //             range: 12.0,
    //             radius: 0.5,
    //             shadows_enabled: true,
    //             shadow_map_near_z: 0.25,
    //             ..default()
    //         },
    //         Transform::from_translation(vec.extend(LIGHT_Z))
    //     ));
    // };
    let light_mat = materials.add(StandardMaterial {
        base_color: LIGHT_COLOR,
        unlit: true,
        ..default()
    });
    let post_mat = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.3, 0.3, 0.3),
        metallic: 0.7,
        perceptual_roughness: 0.3,
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
        Transform::from_xyz(0.0, 3.0, 2.0)
    ));
    //lamp bulb
    commands.spawn((
        Mesh3d(bulb_mesh),
        MeshMaterial3d(light_mat.clone()),
        Transform::from_xyz(0.0, 3.0, -0.5),
        NotShadowReceiver,
        NotShadowCaster
    ));
    // post
    commands.spawn((
        Mesh3d(post_mesh),
        MeshMaterial3d(post_mat.clone()),
        Transform::from_xyz(-1.2, 1.0, -0.5).with_rotation(Quat::from_rotation_z(-FRAC_PI_6)),
        NotShadowCaster
    ));
    commands.insert_resource(LampMaterials {bulb_material: light_mat, post_material: post_mat});
}

#[derive(Resource)]
pub struct LampMaterials {
    bulb_material: Handle<StandardMaterial>,
    post_material: Handle<StandardMaterial>
}

struct LampSpecs {
    root_angle: f32,
    bulb_location: Vec2,
} impl LampSpecs {
    pub fn new(
        root_angle: f32, bulb_location: Vec2
    ) -> Self {
        Self {root_angle, bulb_location}
    }
}

pub fn spawn_lamps(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    lamp_materials: Res<LampMaterials>
) {
    let intensity = 20000.0;
    let range = 10.0;
    let radius = 0.15;
    let bulb_mesh = meshes.add(Sphere::new(radius));
    let post_mesh = meshes.add(Cone::new(LIGHT_POST_RADIUS, LIGHT_POST_LENGTH));
    let lamp_locations = [
        Vec3::new(2.8, -3.2, LIGHT_Z),
        Vec3::new(-2.9, -5.2, LIGHT_Z),
        Vec3::new(1.2, -10.5, LIGHT_Z),
        Vec3::new(-2.3, -16.0, LIGHT_Z),
        Vec3::new(2.5, -21.6, LIGHT_Z),
        Vec3::new(-3.6, -28.8, LIGHT_Z),
        Vec3::new(4.5, -30.2, LIGHT_Z),
        Vec3::new(0.0, -33.7, LIGHT_Z),
        Vec3::new(-1.8, -37.5, LIGHT_Z),
        Vec3::new(3.5, -39.4, LIGHT_Z)
    ];
    let root_angles = [
        -FRAC_PI_2 + 0.6,
        -FRAC_PI_2 - 0.8,
        FRAC_PI_2 - 0.1,
        PI - 0.4,
        FRAC_PI_2 - 0.1,
        PI - FRAC_PI_4 + 0.1,
        -FRAC_PI_2 + FRAC_PI_4 + 0.1,
        -0.0,
        PI - 0.2,
        0.3
    ];
    for i in 0..10 {
        let bulb_vec = lamp_locations[i].xy().extend(BULB_Z);
        let light_post_vec = {
            let a = root_angles[i];
            let x = a.cos() * LIGHT_POST_DISTANCE + bulb_vec.x;
            let y = a.sin() * LIGHT_POST_DISTANCE + bulb_vec.y;
            Vec3::new(x, y, LIGHT_POST_Z)
        };
        let mut lamp_transform = Transform::from_translation(light_post_vec)
            .looking_at(bulb_vec, Vec3::Z);
        lamp_transform.rotation *= Quat::from_rotation_x(-FRAC_PI_2);
        if i != 7 {
            commands.spawn((
                Mesh3d(post_mesh.clone()),
                MeshMaterial3d(lamp_materials.post_material.clone()),
                lamp_transform,
                NotShadowCaster,
                NotShadowReceiver
            ));
        };
        commands.spawn((
            Mesh3d(bulb_mesh.clone()),
            MeshMaterial3d(lamp_materials.bulb_material.clone()),
            Transform::from_translation(bulb_vec),
            NotShadowCaster,
            NotShadowReceiver
        ));
        commands.spawn((
            PointLight {
                color: LIGHT_COLOR,
                intensity,
                range,
                radius,
                shadows_enabled: true,
                shadow_map_near_z: radius,
                ..default()
            },
            Transform::from_translation(lamp_locations[i])
        ));
    }
}

pub fn spawn_sliding_lights(
    sliding_blocks: Res<SlidingBlocks2>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    lamp_materials: Res<LampMaterials>
) {
    let intensity = 10000.0;
    let range = 10.0;
    let radius = 0.10;
    let mesh = meshes.add(Sphere::new(radius));
    for &entity in &sliding_blocks.vec {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(lamp_materials.bulb_material.clone()),
            Transform::from_xyz(0.0, 0.0, 0.5),
            ChildOf(entity),
            NotShadowCaster,
            NotShadowReceiver
        ));
        commands.spawn((
            PointLight {
                color: LIGHT_COLOR,
                intensity,
                range,
                radius,
                shadows_enabled: true,
                shadow_map_near_z: radius,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
            ChildOf(entity)
        ));
    };
}

#[derive(Component)]
pub struct BigOrb;

#[derive(Component)]
pub struct TheSpirits {
    id: u8,
    orb_owned: bool
}

pub fn spawn_the_spirits(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_entity: Res<PlayerEntity>
) {
    let central_mesh = meshes.add(Sphere::new(0.15));
    let material = materials.add(StandardMaterial{
        base_color: Color::linear_rgba(1.0, 1.0, 1.0, 1.0),
        emissive: LinearRgba::new(1.0, 1.0, 1.0, 1.0),
        ..default()
    });
    let big_orb = commands.spawn((
        Mesh3d(central_mesh),
        MeshMaterial3d(material),
        TheSpirits{id: 0, orb_owned: true},
        Transform::from_xyz(0.0, DAMSEL_Y - 0.5, 0.0),
        PointLight {
            color: LIGHT_COLOR,
            intensity: 10000.0,
            range: 8.0,
            radius: 0.15,
            shadows_enabled: true,
            shadow_map_near_z: 0.15,
            ..default()
        },
        NotShadowReceiver,
        NotShadowCaster,
        Visibility::Visible,
        BigOrb
    )).id();
    let mut i = 0u8;
    let orb_mesh = meshes.add(Sphere::new(ORB_RADIUS));
    let sphere_colors = [
        Color::linear_rgba(1.0, 0.0, 0.0, 1.0),
        Color::linear_rgba(0.0, 1.0, 0.0, 1.0),
        Color::linear_rgba(0.0, 0.0, 1.0, 1.0)
    ];
    for color in sphere_colors {
        let material = materials.add(StandardMaterial{
            base_color: color,
            emissive: color.to_linear(),
            ..default()
        });
        for entity in [big_orb, player_entity.entity] {
            let o = entity == big_orb;
            commands.spawn((
                Mesh3d(orb_mesh.clone()),
                MeshMaterial3d(material.clone()),
                TheSpirits{id: i, orb_owned: o},
                PointLight {
                    color,
                    intensity: 1000.0,
                    range: 1.0,
                    radius: 0.1,
                    shadows_enabled: true,
                    shadow_map_near_z: 0.15,
                    ..default()
                },
                Transform::default(),
                ChildOf(entity),
                NotShadowReceiver,
                NotShadowCaster,
                Visibility::Visible,
            ));
        };
        i += 1;
    };
    commands.insert_resource(SpiritsAcquired{bool: false});
}


#[derive(Resource)]
pub struct SpiritsAcquired {
    pub bool: bool
}

pub fn acquire_the_orbs(
    player_pos: Res<PlayerPos>,
    mut spirits_acquired: ResMut<SpiritsAcquired>,
    is_idle: Res<IsIdle>
) {
    if !spirits_acquired.bool && !is_idle.bool && player_pos.vec.y < DAMSEL_Y {
        spirits_acquired.bool = true;
    };
}

pub fn manage_the_the_spirits(
    time: Res<Time>,
    orb_query: Query<(&mut Transform, &mut Visibility, &TheSpirits), Without<BigOrb>>
) {
    for (mut transform, mut vis, spirits) in orb_query {
        let is_visible = vis.clone() == Visibility::Visible;
        if !is_visible {
            continue
        };
        let a = spirits.id as f32 * ORB_A + time.elapsed_secs() * ORB_SPIN_SPEED;
        let cos = a.cos() * ORB_ORBIT_RADIUS;
        let sin = a.sin() * ORB_ORBIT_RADIUS;
        transform.translation.x = cos;
        transform.translation.z = sin;
    };
}

pub fn orb_vis_system(
    orb_query: Query<(&mut Visibility, &TheSpirits)>,
    spirits_acquired: Res<SpiritsAcquired>
) {
    for (mut vis, spirits) in orb_query {
        let is_visible = vis.clone() == Visibility::Visible;
        let should_be_visible = spirits_acquired.bool != spirits.orb_owned;
        if is_visible != should_be_visible {
            vis.toggle_visible_hidden();
        };
    };
}

// #[derive(Component)]
// pub struct SpinnyTitle;

pub fn spawn_title(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let font_size = 1.0f32;
    let lit_material = materials.add(
        StandardMaterial{
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        }
    );
    let unlit_material = materials.add(
        StandardMaterial{
            base_color: Color::BLACK,
            unlit: true,
            ..default()
        }
    );
    let assets = SegmentedDisplayAssets::new(
        font_size, lit_material, unlit_material, &mut meshes
    );
    let s = "SPIDER WELL";
    let segmented_string = SegmentedDisplayString::new(
        s, font_size, 'O', s.len() as u8, false
    );
    let s = spawn_segmented_string(
        Transform::from_xyz(0.0, 4.0, 1.0), segmented_string, assets, &mut commands
    );
    // commands.entity(s).insert(SpinnyTitle);
}

// pub fn make_it_spin(
//     query: Query<&mut Transform, With<SpinnyTitle>>,
//     time: Res<Time>
// ) {
//     for mut t in query {
//         t.rotate_y(time.delta_secs());
//     };
// }

#[derive(Component)]
pub struct TimerText {
    flash: f32
}

pub fn spawn_timer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>
) -> Entity {
    let font_size = 0.532;
    let lit_material = materials.add(
        StandardMaterial{
            base_color: Color::from(css::GOLD),
            unlit: true,
            ..default()
        }
    );
    let unlit_material = materials.add(
        StandardMaterial{
            base_color: Color::BLACK,
            unlit: true,
            ..default()
        }
    );
    let assets = SegmentedDisplayAssets::new(
        font_size, lit_material, unlit_material, meshes
    );
    let s = "000";
    let segmented_string = SegmentedDisplayString::new(
        s, font_size, '0', 3, false
    );
    let entity = spawn_segmented_string(
        Transform::from_xyz((-LEVEL_WIDTH / 2.0) + 0.5, 3.5, -1.0), segmented_string, assets, commands
    );
    commands.entity(entity).insert(TimerText{flash: 0.0});
    entity
}

pub fn update_timer_text(
    timer: Res<SpeedRunTimer>,
    time: Res<Time>,
    im_winning: Res<ImWinningDad>,
    mut query: Query<(&mut SegmentedDisplayString, &mut Visibility, &mut TimerText)>
) {
    for (mut segmented_display, mut visibility, mut timer_text) in query {
        if segmented_display.string != timer.string {
            segmented_display.string = timer.string.clone();
        };
        let is_visible = match visibility.clone() {
            Visibility::Visible => true,
            _ => false
        };
        if !im_winning.bool && !is_visible {
            visibility.toggle_visible_hidden()
        } else if im_winning.bool{
            timer_text.flash += time.delta_secs();
            if is_visible && timer_text.flash > 0.5 {
                visibility.toggle_visible_hidden()
            } else if timer_text.flash >= 1.0 {
                visibility.toggle_visible_hidden();
                timer_text.flash = 0.0
            };
        };
    };
}

pub fn spawn_helpful_text(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {

    // define sizes
    let helpful_font_size = 0.5f32;
    let timer_font_size = 0.5f32;
    let win_font_size = 2.0;

    //define materials
    let white_material = materials.add(
        StandardMaterial{
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        }
    );
    let unlit_material = materials.add(
        StandardMaterial{
            base_color: Color::BLACK,
            unlit: true,
            ..default()
        }
    );
    let clear_material = materials.add(
        StandardMaterial{
            base_color: Color::linear_rgba(0.0, 0.0, 0.0, 0.0),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            ..default()
        }
    );
    let green_material = materials.add(
        StandardMaterial {
            base_color: Color::from(css::LAWN_GREEN),
            unlit: true,
            ..default()
        }
    );

    // assets
    let helpful_assets = SegmentedDisplayAssets::new(
        helpful_font_size, white_material.clone(), clear_material.clone(), &mut meshes
    );
    let win_assets = SegmentedDisplayAssets::new(
        win_font_size, green_material.clone(), clear_material.clone(), &mut meshes
    );

    let win_string = "YOU WIN";
    let ascend_string = "ASCEND";
    let descend_string = "DESCEND";

    let descend_segments = SegmentedDisplayString::new(
        descend_string, helpful_font_size, ' ', descend_string.len() as u8, false
    );
    let win_segments = SegmentedDisplayString::new(
        win_string, win_font_size, ' ', win_string.len() as u8, false
    );
    let ascend_segment = SegmentedDisplayString::new(
        ascend_string, helpful_font_size, ' ', ascend_string.len() as u8, false
    );

    let win_entity = spawn_segmented_string(
        Transform::from_xyz(-0.5, 0.0, 2.0), win_segments, win_assets, &mut commands
    );
    commands.entity(win_entity).insert(ShowConditionally{condition: DisplayCondition::ShowAfterWin});

    let descend_entity = spawn_segmented_string(
        Transform::from_xyz(1.4, -1.5, 2.0), descend_segments, helpful_assets.clone(), &mut commands
    );
    commands.entity(descend_entity).insert(ShowConditionally{condition: DisplayCondition::ShowBeforeOrbs});

    let ascend_entity = spawn_segmented_string(
        Transform::from_xyz(0.0, DAMSEL_Y + 1.0, 2.0), ascend_segment, helpful_assets.clone(), &mut commands
    );
    commands.entity(ascend_entity).insert(ShowConditionally{condition: DisplayCondition::ShowAfterOrbs});
}

pub enum DisplayCondition{
    ShowAfterOrbs,
    ShowAfterWin,
    ShowBeforeOrbs
}

#[derive(Component)]
pub struct ShowConditionally {
    condition: DisplayCondition
}

pub fn update_conditional_displays(
    im_winning: Res<ImWinningDad>,
    spirits_acquired: Res<SpiritsAcquired>,
    mut query: Query<(&mut Visibility, &ShowConditionally)>
) {
    for (mut vis, cond) in &mut query {
        let is_visible = vis.clone() == Visibility::Visible;
        let should_be_visible = match cond.condition {
            DisplayCondition::ShowAfterWin => im_winning.bool,
            DisplayCondition::ShowAfterOrbs => spirits_acquired.bool,
            DisplayCondition::ShowBeforeOrbs => !spirits_acquired.bool
        };
        if is_visible != should_be_visible {
            vis.toggle_visible_hidden()
        };
    }
}