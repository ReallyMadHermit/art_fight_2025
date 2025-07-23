use std::f32::consts::{PI, TAU, FRAC_1_SQRT_2, FRAC_PI_6, FRAC_PI_2};
use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use crate::spider_well::mechanics::{PlayerInputs, PLAYER_CLIMB, PlayerEntity};

const SPIDER_GREEN: Color = Color::hsl(118.0, 0.6, 0.9);
const SPIDER_PURPLE: Color = Color::hsl(313.0, 0.5, 0.6);
const SPIDER_GREY: Color = Color::hsl(37.0, 0.1, 0.7);
const SPIDER_BLACK: Color = Color::hsl(0.0, 0.0, 0.15);
const SPIDER_EYES: Color = Color::hsl(287.0, 0.6, 0.5);

const JOINT_RADIUS: f32 = 0.05;
const SEGMENT_RADIUS: f32 = 0.04;
const CLAW_RADIUS: f32 = 0.05;
const SEGMENT_LENGTH: f32 = 0.25;

const LIMB_SPACING: f32 = 0.125;
const CLIMB_DISPLACEMENT: f32 = 0.25;
const CLIMB_CYCLE_TIME: f32 = (CLIMB_DISPLACEMENT * 2.0) / PLAYER_CLIMB;
const JOINT_ANGLE_SPACING: f32 = PI / 4.0;

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
    const SEGMENTS: [Self; 4] = [
        Self::LegSegment,
        Self::UpperArmSegment,
        Self::MiddleArmSegment,
        Self::LowerArmSegment,
    ];
}

pub fn spawn_spider_parts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_entity: Res<PlayerEntity>
) {
    // spawn limbs, keep segment IDs for bone embellishment
    let segment_entities = {
        // assets
        let joint_mesh = meshes.add(Sphere::new(JOINT_RADIUS));
        let segment_mesh = meshes.add(Cylinder::new(SEGMENT_RADIUS, 1.0));
        let claw_mesh = meshes.add(Sphere::new(CLAW_RADIUS));
        let joint_material = materials.add(StandardMaterial {
            base_color: SPIDER_PURPLE,
            ..default()
        });
        let segment_material = materials.add(StandardMaterial {
            base_color: SPIDER_GREY,
            ..default()
        });
        let claw_material = materials.add(StandardMaterial {
            base_color: SPIDER_BLACK,
            ..default()
        });
        
        // spawn limbs
        let mut entities: Vec<Entity> = Vec::with_capacity(12);
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
                        entities.push(entity);
                    };
                };
            };
        };
        // returns segment entities to add bone shapes to
        entities
    };
}

pub fn update_joints(
    time: Res<Time>,
    player_inputs: Res<PlayerInputs>,
    mut limb_query: Query<(&mut Transform, &SpiderLimbPart)>,
) {
    let t = (time.elapsed_secs() % CLIMB_CYCLE_TIME) / CLIMB_CYCLE_TIME;
    let t1 = (t + 0.5) % 1.0;
    let cycle = (t * TAU).sin();
    let cycle_1 = (t1 * TAU).sin();
}

pub fn insert_star_pos_hashmap(
    mut commands: Commands
) {
    let mut limb_positions = LimbPositions{hash_map: HashMap::with_capacity(8 * 3)};
    for joint in LimbPartType::JOINTS {
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
                limb_positions.hash_map.insert(limb_part, Vec2::new(cos * l * s, sin * l));
            };
        };
    };
    commands.insert_resource(limb_positions);
}

pub fn update_spider_parts(
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
            // if this is the first joint, we attach at 0.0 (TODO: add the limb spacing here)
            let p = if spider_part.segment_id < 1 {
                point_to / 2.0
            } else { // else we lookup the previous joint
                let mut base_key = joint_key.clone();
                base_key.segment_id -= 1;
                let base_pos = limb_positions.hash_map.get(&base_key).unwrap().clone();
                (base_pos + point_to) / 2.0
            };
            transform.translation = p.extend(0.0);
            transform.look_at(point_to.extend(0.0), Vec3::Y);
            transform.rotate_z(FRAC_PI_2);
        } else {
            // if it's a joint we just... apply it, lmao, spheres, amiright?
            let joint_pos = limb_positions.hash_map.get(spider_part).unwrap().clone();
            transform.translation = joint_pos.extend(0.0);
        };
    };
}

fn animate_legs(
    cycle: f32, cycle_1: f32, player_inputs: &Res<PlayerInputs>
) -> [[Vec2; 3]; 2] {
    let m = SEGMENT_LENGTH * FRAC_1_SQRT_2;
    let v = Vec2::new(-m, m);
    let v1 = Vec2::new(m, m);
    return [
        [v, v * 2.0, v * 3.0],
        [v1, v1 * 2.0, v1 * 3.0]
    ];
}
