use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};
use std::process::id;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};

const SEGMENT_LENGTH: f32 = 0.42;
const SEGMENT_HALF_LENGTH: f32 = 0.18;
const SEGMENT_DIAMETER: f32 = 0.03;
const SPACING_MAX: f32 = 0.48;
const SPACING_MID: f32 = 0.24;
const SPACING_MIN: f32 = 0.12;
const CHAR_SPACING: f32 = SPACING_MIN;
const ANGLED_ANGLE: f32 = FRAC_PI_3;

#[derive(Component)]
pub struct SegmentedDisplayAssets {
    long_mesh: Handle<Mesh>,
    short_mesh: Handle<Mesh>,
    lit_material: Handle<StandardMaterial>,
    unlit_material: Handle<StandardMaterial>
} impl SegmentedDisplayAssets {
    pub fn new(
        char_height: f32,
        lit_material: Handle<StandardMaterial>,
        unlit_material: Handle<StandardMaterial>,
        meshes: &mut ResMut<Assets<Mesh>>
    ) -> Self {
        let ll = char_height * SEGMENT_LENGTH;
        let sl = char_height * SEGMENT_HALF_LENGTH;
        let dr = char_height * SEGMENT_DIAMETER;
        let long_mesh = meshes.add(Cuboid::new(ll, dr, dr));
        let short_mesh = meshes.add(Cuboid::new(sl, dr, dr));
        Self {
            long_mesh,
            short_mesh,
            lit_material,
            unlit_material
        }
    }
    pub fn get_segment_mesh(&self) -> Handle<Mesh> {
        self.long_mesh.clone()
    }
    pub fn get_half_segment_mesh(&self) -> Handle<Mesh> {
        self.short_mesh.clone()
    }
    pub fn get_lit_segment_material(&self) -> Handle<StandardMaterial> {
        self.lit_material.clone()
    }
    pub fn get_unlit_segment_material(&self) -> Handle<StandardMaterial> {
        self.unlit_material.clone()
    }
}

#[derive(Copy, Clone)]
pub enum SegmentedAnchor {
    Top,
    TopLeft,
    Left,
    Right,
}

#[derive(Component)]
pub struct SegmentedString {
    pub string: String,
    char_height: f32,
    char_count: u8
} impl SegmentedString {
    pub fn new(
        string: &str, char_height: f32, char_count: u8
    ) -> Self {
        Self {string: string.to_string(), char_height, char_count }
    }
    pub fn get_rendered_length(char_count: u8, char_height: f32) -> f32 {
        let l = char_count as f32 * char_height * 0.5;
        if char_count > 1 {
            let s = (char_count - 1) as f32 * char_height * CHAR_SPACING;
            s + l
        } else {
            l
        }
    }
}

#[derive(Component)]
pub struct SegmentedChar {
    char: char,
    index: u8
} impl SegmentedChar {
    // the segments 0 through 5 are the outermost segments, clockwise, from the top
    // 6 and 7 are the left and right middle horizontal segments
    // 8, 9, and 10 are the 3 segments above the horizontal line, left to right
    // 11, 12, and 13 are below the horizontal line, left to right
    pub fn get_sequence(&self) -> [bool; 14] {
        let mut off = [false; 14];
        let segs = match self.char {
            '1' => "1,2,10",
            '2' => "0,1,7,6,4,3",
            '3' => "0,1,2,3,7",
            '4' => "1,2,5,6,7",
            '5' => "0,5,6,7,2,3",
            '6' => "0,5,4,3,2,7,6",
            '7' => "0,10,12",
            '8' => "0,1,2,3,4,5,6,7",
            '9' => "2,1,0,5,6,7",
            '0' => "0,1,2,3,4,5",
            'A' => "4,5,0,1,2,6,7",
            'B' => "0,1,2,3,7,9,12",
            'C' => "0,5,4,3",
            'D' => "0,1,2,3,7,9,12",
            'E' => "0,5,4,3,6,7",
            'F' => "0,5,4,6,7",
            'G' => "0,5,4,3,2,7",
            'H' => "1,2,4,5,6,7",
            'I' => "0,3,9,12",
            'J' => "1,2,3,4",
            'K' => "5,4,6,10,13",
            'L' => "5,4,3",
            'M' => "1,2,4,5,8,10",
            'N' => "1,2,4,5,8,13",
            'O' => "0,1,2,3,4,5",
            'P' => "4,5,0,1,6,7",
            'Q' => "0,1,2,3,4,5,13",
            'R' => "4,5,0,1,6,7,13",
            'S' => "0,8,7,2,3",
            'T' => "0,9,12",
            'U' => "1,2,3,4,5",
            'V' => "4,5,10,11",
            'W' => "1,2,4,5,11,13",
            'X' => "8,10,11,13",
            'Y' => "8,10,12",
            'Z' => "0,3,10,11",
            _ => return off
        };
        let split = segs.split(',');
        for s in split {
            let n: usize = s.parse().unwrap();
            off[n] = true;
        };
        off
    }
}

#[derive(Component)]
pub struct Segment {
    id: u8,
    lit: bool
} impl Segment {
    fn get_transform(id: u8, char_height: f32) -> Transform {
        let x = match id {
            4 | 5 => -SPACING_MID,
            6 | 8 | 11 => -SPACING_MIN,
            0 | 3 | 9| 12 => 0.0,
            7 | 10 | 13 => SPACING_MIN,
            1 | 2 => SPACING_MID,
            _ => 0.0
        };
        let y = match id {
            0 => SPACING_MAX,
            1 | 5 | 8 | 9 | 10 => SPACING_MID,
            6 | 7 => 0.0,
            2 | 4 | 11 | 12 | 13 => -SPACING_MID,
            3 => -SPACING_MAX,
            _ => 0.0
        };
        let a = match id {
            0 | 3 | 6 | 7 => 0.0,
            1 | 2 | 4 | 5 | 9 | 12 => FRAC_PI_2,
            10 | 11 => ANGLED_ANGLE,
            8 | 13 => ANGLED_ANGLE * 2.0,
            _ => 0.0
        };
        Transform::from_xyz(x * char_height, y * char_height, 0.0)
            .with_rotation(Quat::from_rotation_z(a))
    }
    fn is_short(id: u8) -> bool {
        match id {
            6 | 7 => true,
            _ => false
        }
    }
}

pub fn spawn_segmented_string(
    transfrom: Transform,
    segmented_string: SegmentedString,
    display_assets: SegmentedDisplayAssets,
    commands: &mut Commands
) -> Entity {
    let chars = spawn_chars(
        commands, segmented_string.char_count, segmented_string.char_height
    );
    for &char_entity in &chars {
        spawn_segments(commands, &display_assets, segmented_string.char_height, char_entity)
    };
    let segmented_entity = commands.spawn((
        segmented_string,
        display_assets,
        transfrom
    )).id();
    for char in chars {
        commands.entity(char).insert(ChildOf(segmented_entity));
    };
    segmented_entity
}

fn spawn_chars(
    commands: &mut Commands,
    char_count: u8,
    char_height: f32
) -> Vec<Entity> {
    let mut chars: Vec<Entity> = Vec::with_capacity(char_count as usize);
    let (start_x, dx) = if char_count > 1 {
        let length = SegmentedString::get_rendered_length(char_count, char_height);
        let step = length / char_count as f32;
        let start = -(length / 2.0) + (char_height / 2.0);
        (start, step)
    } else {
        (0.0f32, 0.0f32)
    };
    for i in 0..char_count {
        let entity = commands.spawn((
            SegmentedChar {
                char: '0',
                index: i
            },
            Transform::from_xyz(start_x + dx * i as f32, 0.0, 0.0)
        )).id();
        chars.push(entity)
    };
    chars
}

fn spawn_segments(
    commands: &mut Commands,
    assets: &SegmentedDisplayAssets,
    char_height: f32,
    parent: Entity
) {
    for i in 0..14u8 {
        let segment = Segment{id: i, lit: false};
        let transform = Segment::get_transform(i, char_height);
        let mesh = if Segment::is_short(i) {
            assets.short_mesh.clone()
        } else {
            assets.long_mesh.clone()
        };
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(assets.unlit_material.clone()),
            segment,
            transform,
            ChildOf(parent),
            NotShadowReceiver,
            NotShadowCaster
        ));
    };
}

pub fn update_segmented_displays(
    mut commands: Commands,
    mut segmented_string_query: Query<(&SegmentedString, &SegmentedDisplayAssets, &Children)>,
    mut char_query: Query<(&mut SegmentedChar, &Children)>,
    mut segment_query: Query<(&mut Segment)>
) {
    for (segmented_string, assets, string_children) in segmented_string_query {
        let mut character_truths: Vec<char> = Vec::with_capacity(segmented_string.char_count as usize);
        for c in segmented_string.string.to_uppercase().chars() {
            character_truths.push(c);
        };
        for &char_entity in string_children {
            let (mut seg_char, char_children) = 
                char_query.get_mut(char_entity).unwrap();
            let c = seg_char.char;
            if character_truths[seg_char.index as usize] != c {
                seg_char.char = character_truths[seg_char.index as usize];
                let seq = seg_char.get_sequence();
                for &segment_entity in char_children {
                    let mut segment = segment_query.get_mut(segment_entity).unwrap();
                    let is_lit = segment.lit;
                    let should_be_lit = seq[segment.id as usize];
                    if is_lit != should_be_lit {
                        if should_be_lit {
                            segment.lit = true;
                            commands.entity(segment_entity).insert(MeshMaterial3d(assets.lit_material.clone()));
                        } else if is_lit {
                            segment.lit = false;
                            commands.entity(segment_entity).insert(MeshMaterial3d(assets.unlit_material.clone()));
                        };
                    };
                };
            };
        };
    };
}
