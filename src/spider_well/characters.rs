use std::f32::consts::{PI, TAU, FRAC_1_SQRT_2, FRAC_PI_6, FRAC_PI_2};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use crate::spider_well::mechanics::{PlayerInputs, PLAYER_CLIMB, PlayerEntity};

const SPIDER_GREEN: Color = Color::hsl(118.0, 0.7, 0.7);
const SPIDER_PURPLE: Color = Color::hsl(313.0, 0.5, 0.6);
const SPIDER_GREY: Color = Color::hsl(37.0, 0.1, 0.7);
const SPIDER_BLACK: Color = Color::hsl(0.0, 0.0, 0.15);
const SPIDER_EYES: Color = Color::hsl(287.0, 0.6, 0.5);

const CHARACTER_SCALE: f32 = 0.75;

const JOINT_RADIUS: f32 = 0.045 * CHARACTER_SCALE;
const SEGMENT_RADIUS: f32 = 0.04 * CHARACTER_SCALE;
const CLAW_RADIUS: f32 = 0.035 * CHARACTER_SCALE;
const SEGMENT_LENGTH: f32 = 0.25 * CHARACTER_SCALE;
const LIMB_SPACING: f32 = -0.05 * CHARACTER_SCALE;
const LIMB_START: f32 = 0.1;

const BONE_RADIUS: f32 = SEGMENT_RADIUS * 0.5;
const BONE_HALF_HEIGHT: f32 = 0.7;
const BONE_Z: f32 = SEGMENT_RADIUS * 0.8;

const BODY_TRANSLATION: Vec3 = Vec3{x: 0.0, y: 0.0, z: 0.0};
const BODY_RADIUS: f32 = 0.125 * CHARACTER_SCALE;
const BODY_HH: f32 = 0.25 * CHARACTER_SCALE;
const TUMMY_TRANSLATION: Vec3 = Vec3{x: 0.0, y: 0.025, z: 0.025};
const TUMMY_RADIUS: f32 = BODY_RADIUS * 0.9;
const TUMMY_HH: f32 = BODY_HH;

const BODY_BONE_DELTA: f32 = 0.01;
const BODY_BONE_INNER: f32 = BODY_RADIUS - BODY_BONE_DELTA;
const BODY_BONE_OUTER: f32 = BODY_RADIUS + BODY_BONE_DELTA;

const BULB_RADIUS: f32 = 0.25 * CHARACTER_SCALE;
const BULB_SCALE: Vec3 = Vec3{x: 1.0, y: 1.0, z: 0.5};
const BULB_TRANSLATION: Vec3 = Vec3{x: 0.0, y: 0.2, z: -0.1};
const BULB_ROTATION: f32 = -FRAC_PI_6;
const RING_INNER: f32 = 0.12 * CHARACTER_SCALE;
const RING_OUTER: f32 = 0.18 * CHARACTER_SCALE;
const RING_TRANSLATION: Vec3 = Vec3{x: 0.0, y: 0.0, z: -0.15};

const HEAD_RADIUS: f32 = 0.15 * CHARACTER_SCALE;
const HEAD_Y: f32 = -0.2;
const HEAD_Z: f32 = 0.05;

const LARGE_EYE_RADIUS: f32 = 0.06 * CHARACTER_SCALE;
const LARGE_EYE_X: f32 = LARGE_EYE_RADIUS;
const LARGE_EYE_Y: f32 = -LARGE_EYE_RADIUS * 0.25;
const PUPIL_SCALE_FACTOR: f32 = 0.6;

const TOOTH_RADIUS: f32 = 0.03 * CHARACTER_SCALE;
const TOOTH_LENGTH: f32 = 0.12 * CHARACTER_SCALE;
const TOOTH_Y: f32 = 0.08;

const CLIMB_DISPLACEMENT: f32 = 0.25;
const CLIMB_CYCLE_TIME: f32 = (CLIMB_DISPLACEMENT * 2.0) / PLAYER_CLIMB;

#[derive(Component, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SpiderLimbPart {
    part_type: LimbPartType,
    segment_id: u8,
    side: i8
}

#[derive(Resource)]
pub struct LimbPositions {
    hash_map: HashMap<SpiderLimbPart, Vec2>
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
enum LimbPartType {
    LegSegment,
    LegJoint,
    UpperArmSegment,
    UpperArmJoint,
    MiddleArmSegment,
    MiddleArmJoint,
    LowerArmSegment,
    LowerArmJoint
} impl LimbPartType {
    fn is_segment(&self) -> bool {
        match self {
            Self::LegSegment => true,
            Self::UpperArmSegment => true,
            Self::MiddleArmSegment => true,
            Self::LowerArmSegment => true,
            _ => false
        }
    }
    fn get_twin(&self) -> Self {
        match self {
            Self::LegJoint => Self::LegSegment,
            Self::UpperArmJoint => Self::UpperArmSegment,
            Self::MiddleArmJoint => Self::MiddleArmSegment,
            Self::LowerArmJoint => Self::LowerArmSegment,
            Self::LegSegment => Self::LegJoint,
            Self::UpperArmSegment => Self::UpperArmJoint,
            Self::MiddleArmSegment => Self::MiddleArmJoint,
            Self::LowerArmSegment => Self::LowerArmJoint
        }
    }
    fn get_root_offset(&self) -> f32 {
        match self {
            Self::LegJoint => LIMB_START,
            Self::LowerArmJoint => LIMB_START + LIMB_SPACING,
            Self::MiddleArmJoint => LIMB_START + LIMB_SPACING * 2.0,
            Self::UpperArmJoint =>  LIMB_START + LIMB_SPACING * 3.0,
            _ => 0.0
        }
    }
    const LIMB_IDS: [u8; 3] = [0, 1, 2];
    const LIMB_SIDES: [i8; 2] = [-1, 1];
    const LIMB_PARTS: [Self; 8] = [
        Self::LegJoint,
        Self::UpperArmJoint,
        Self::MiddleArmJoint,
        Self::LowerArmJoint,
        Self::LegSegment,
        Self::UpperArmSegment,
        Self::MiddleArmSegment,
        Self::LowerArmSegment,
    ];
    const JOINTS: [Self; 4] = [
        Self::LegJoint,
        Self::UpperArmJoint,
        Self::MiddleArmJoint,
        Self::LowerArmJoint
    ];
}

#[derive(Resource)]
pub struct SpiderMaterials {
    green: Handle<StandardMaterial>,
    purple: Handle<StandardMaterial>,
    grey: Handle<StandardMaterial>,
    black: Handle<StandardMaterial>,
    eyes: Handle<StandardMaterial>,
    teeth: Handle<StandardMaterial>
} impl SpiderMaterials {
    fn get_green(&self) -> Handle<StandardMaterial> {
        self.green.clone()
    }
    fn get_purple(&self) -> Handle<StandardMaterial> {
        self.purple.clone()
    }
    fn get_grey(&self) -> Handle<StandardMaterial> {
        self.grey.clone()
    }
    fn get_black(&self) -> Handle<StandardMaterial> {
        self.black.clone()
    }
    fn get_eyes(&self) -> Handle<StandardMaterial> {
        self.eyes.clone()
    }
}

pub fn insert_spider_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let green = materials.add(StandardMaterial {
        base_color: SPIDER_GREEN,
        ..default()
    });
    let purple = materials.add(StandardMaterial {
        base_color: SPIDER_PURPLE,
        ..default()
    });
    let grey = materials.add(StandardMaterial {
        base_color: SPIDER_GREY,
        ..default()
    });
    let black = materials.add(StandardMaterial {
        base_color: SPIDER_BLACK,
        ..default()
    });
    let eyes = materials.add(StandardMaterial {
        base_color: SPIDER_EYES,
        ..default()
    });
    let teeth = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    commands.insert_resource(
        SpiderMaterials {green, purple, grey, black, eyes, teeth}
    );
}

pub fn spawn_spider_parts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    player_entity: Res<PlayerEntity>,
    spider_materials: Res<SpiderMaterials>
) {
    // spawn limbs, keep segment IDs for bone embellishment
    let segment_entities = {
        // assets
        let joint_mesh = meshes.add(Sphere::new(JOINT_RADIUS));
        let segment_mesh = meshes.add(Cylinder::new(SEGMENT_RADIUS, 1.0));
        let claw_mesh = meshes.add(Sphere::new(CLAW_RADIUS));
        let joint_material = spider_materials.get_purple();
        let segment_material = spider_materials.get_grey();
        let claw_material = spider_materials.get_black();
        
        // spawn limbs
        let mut entities: Vec<(Entity, i8)> = Vec::with_capacity(12);
        for part in LimbPartType::LIMB_PARTS {  // attach limbs to the spider
            let is_segment = part.is_segment();
            let mut mesh = if is_segment {
                &segment_mesh
            } else {
                &joint_mesh
            };
            let mut material = if is_segment {
                &segment_material
            } else {
                &joint_material
            };
            for id in LimbPartType::LIMB_IDS {
                if !is_segment && id == LimbPartType::LIMB_IDS.last().unwrap().clone() {
                    mesh = &claw_mesh;
                    material = &claw_material;
                };
                for side in LimbPartType::LIMB_SIDES {
                    let l = if is_segment {
                        SEGMENT_LENGTH
                    } else {
                        1.0
                    };
                    let entity = commands.spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_scale(Vec3::new(1.0, l, 1.0)),
                        SpiderLimbPart {
                            part_type: part,
                            segment_id: id,
                            side
                        },
                        ChildOf(player_entity.entity.clone())
                    )).id();
                    if is_segment {
                        entities.push((entity, side));
                    } else {
                        commands.entity(entity).insert(NotShadowCaster);
                    };
                };
            };
        };
        // returns segment entities to add bone shapes to
        entities
    };
    let bone_mesh = meshes.add(Capsule3d::new(BONE_RADIUS, BONE_HALF_HEIGHT));
    for (entity, side) in segment_entities {
        commands.spawn((
            Mesh3d(bone_mesh.clone()),
            MeshMaterial3d(spider_materials.get_green()),
            Transform::from_xyz(0.0, 0.0, BONE_Z * side as f32),
            ChildOf(entity),
            NotShadowCaster,
            NotShadowReceiver
        ));
    }
}

pub fn insert_limb_positions(
    mut commands: Commands
) {
    let mut limb_positions = LimbPositions{hash_map: HashMap::with_capacity(8 * 3)};
    for joint in LimbPartType::JOINTS {
        let o = joint.get_root_offset();
        let a = match joint {
            LimbPartType::LegJoint => 1.5 * FRAC_PI_6,
            LimbPartType::LowerArmJoint => 0.5 * FRAC_PI_6,
            LimbPartType::MiddleArmJoint => -0.5 * FRAC_PI_6,
            LimbPartType::UpperArmJoint => -1.5 * FRAC_PI_6,
            _ => 0.0
        };
        let cos = a.cos();
        let sin = a.sin();
        for side_sign in LimbPartType::LIMB_SIDES {
            let s = side_sign as f32;
            for id in LimbPartType::LIMB_IDS {
                let l = (id + 1) as f32 * SEGMENT_LENGTH;
                let limb_part = SpiderLimbPart {
                    part_type: joint,
                    segment_id: id,
                    side: side_sign
                };
                limb_positions.hash_map.insert(limb_part, Vec2::new(cos * l * s, (sin * l) + o));
            };
        };
    };
    commands.insert_resource(limb_positions);
}

pub fn apply_limb_positions(
    limb_positions: Res<LimbPositions>,
    transform_query: Query<(&mut Transform, &SpiderLimbPart)>
) {
    // okay let's iterate the stuff
    for (mut transform, spider_part) in transform_query {
        // is it a segment or a joint?
        let is_segment = spider_part.part_type.is_segment();
        if is_segment {  // if it's a segment...
            // we gotta look up its joint, since all segments point to their joints
            let mut joint_key = spider_part.clone();
            joint_key.part_type = joint_key.part_type.get_twin();
            let point_to = limb_positions.hash_map.get(&joint_key).unwrap().clone();
            // if this is the first joint, we attach at 0.0 + limb offset
            let p = if spider_part.segment_id < 1 {
                let o = joint_key.part_type.get_root_offset();
                let v = Vec2::new(0.0, o);
                (point_to + v) / 2.0
            } else { // else we lookup the previous joint
                let mut base_key = joint_key.clone();
                base_key.segment_id -= 1;
                let base_pos = limb_positions.hash_map.get(&base_key).unwrap().clone();
                (base_pos + point_to) / 2.0
            };
            transform.translation = p.extend(0.0);
            transform.look_at(point_to.extend(0.0), Vec3::Y);
            transform.rotate_z(FRAC_PI_2);  // twist it cause rotate don't do it right
        } else {
            // if it's a joint we just... apply it, lmao, spheres, amiright?
            let joint_pos = limb_positions.hash_map.get(spider_part).unwrap().extend(0.0);
            transform.translation = joint_pos;
        };
    };
}

pub fn spawn_body(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    player_entity: Res<PlayerEntity>,
    spider_materials: Res<SpiderMaterials>
) {
    let body_mesh = meshes.add(Capsule3d::new(BODY_RADIUS, BODY_HH));
    let tummy_mesh = meshes.add(Capsule3d::new(TUMMY_RADIUS, TUMMY_HH));
    let body = commands.spawn((
        Mesh3d(body_mesh),
        MeshMaterial3d(spider_materials.get_grey()),
        Transform::from_translation(BODY_TRANSLATION),
        ChildOf(player_entity.entity)
    )).id();
    commands.spawn((
        Mesh3d(tummy_mesh),
        MeshMaterial3d(spider_materials.get_purple()),
        Transform::from_translation(TUMMY_TRANSLATION),
        ChildOf(body),
        NotShadowCaster
    ));
    let bone_mesh = meshes.add(Torus::new(BODY_BONE_INNER, BODY_BONE_OUTER));
    let ys = [-0.5 * LIMB_SPACING, LIMB_SPACING * 1.5];
    for y in ys{
        commands.spawn((
            Mesh3d(bone_mesh.clone()),
            MeshMaterial3d(spider_materials.get_green()),
            Transform::from_xyz(0.0, y, 0.0),
            ChildOf(body),
            NotShadowCaster,
            NotShadowReceiver
        ));
    }
    let spine_length = (ys[1] - ys[0]).abs();
    let spine_y = (ys[0] + ys[1]) / 2.0;
    let spine_mesh = meshes.add(Cylinder::new(BODY_BONE_DELTA * 2.0, spine_length));
    commands.spawn((
        Mesh3d(spine_mesh),
        MeshMaterial3d(spider_materials.get_green()),
        Transform::from_xyz(0.0, spine_y, -BODY_RADIUS),
        ChildOf(body),
        NotShadowCaster,
        NotShadowReceiver
    ));
}

pub fn spawn_bum(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    player_entity: Res<PlayerEntity>,
    spider_materials: Res<SpiderMaterials>
) {
    let bulb_mesh = meshes.add(Sphere::new(BULB_RADIUS));
    let ring_mesh = meshes.add(Torus::new(RING_INNER, RING_OUTER));
    let bulb = commands.spawn((
        Mesh3d(bulb_mesh),
        MeshMaterial3d(spider_materials.get_grey()),
        Transform::from_translation(BULB_TRANSLATION)
            .with_scale(BULB_SCALE)
            .with_rotation(Quat::from_rotation_x(BULB_ROTATION)),
        ChildOf(player_entity.entity)
    )).id();
    commands.spawn((
        Mesh3d(ring_mesh),
        MeshMaterial3d(spider_materials.get_purple()),
        Transform::from_translation(RING_TRANSLATION)
            .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        ChildOf(bulb),
        NotShadowCaster
    ));
}

pub fn spawn_head(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    player_entity: Res<PlayerEntity>,
    spider_materials: Res<SpiderMaterials>
) {
    let head_mesh = meshes.add(Sphere::new(HEAD_RADIUS));
    let head = commands.spawn((
        Mesh3d(head_mesh),
        MeshMaterial3d(spider_materials.get_grey()),
        Transform::from_xyz(0.0, HEAD_Y, HEAD_Z),
        ChildOf(player_entity.entity)
    )).id();
    let large_sclara_mesh = meshes.add(Sphere::new(LARGE_EYE_RADIUS));
    let tooth_mesh = meshes.add(Cone::new(TOOTH_RADIUS, TOOTH_LENGTH));
    for x in [LARGE_EYE_X, -LARGE_EYE_X] {
        let sclara = commands.spawn((
            Mesh3d(large_sclara_mesh.clone()),
            MeshMaterial3d(spider_materials.get_black()),
            Transform::from_xyz(x, LARGE_EYE_Y, HEAD_RADIUS - LARGE_EYE_RADIUS),
            ChildOf(head),
            NotShadowCaster,
            NotShadowReceiver
        )).id();
        commands.spawn((
            Mesh3d(large_sclara_mesh.clone()),
            MeshMaterial3d(spider_materials.get_eyes()),
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::new(PUPIL_SCALE_FACTOR, 0.8, 1.1)),
            ChildOf(sclara),
            NotShadowCaster,
            NotShadowReceiver
        ));
        commands.spawn((
            Mesh3d(tooth_mesh.clone()),
            MeshMaterial3d(spider_materials.teeth.clone()),
            ChildOf(head),
            Transform::from_xyz(x * 0.8, TOOTH_Y, HEAD_RADIUS * 0.6),
            NotShadowCaster,
            NotShadowReceiver
        ));
    };
}
