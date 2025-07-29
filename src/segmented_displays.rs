use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};

const SEGMENT_LENGTH: f32 = 0.42;
const SEGMENT_HALF_LENGTH: f32 = 0.18;
const SEGMENT_DIAMETER: f32 = 0.05;
const SPACING_MAX: f32 = 0.48;
const SPACING_MID: f32 = 0.24;
const SPACING_MIN: f32 = 0.12;
const CHAR_SPACING: f32 = 0.2;
const ANGLED_ANGLE: f32 = PI / 10.0;

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
}

#[derive(Copy, Clone)]
pub enum SegmentedAnchor {
    Top,
    TopLeft,
    Left,
    Right,
}

#[derive(Component)]
pub struct SegmentedDisplayString {
    pub string: String,
    font_size: f32,
    padding_char: char,
    digit_count: u8,
    casts_shadows: bool
} impl SegmentedDisplayString {
    pub fn new(
        string: &str, font_size: f32, padding_char: char, digit_count: u8, casts_shadows: bool
    ) -> Self {
        Self {string: string.to_string(), font_size, padding_char, digit_count, casts_shadows }
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
pub struct SegmentDigit {
    char: char,
    index: u8
} impl SegmentDigit {
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
            'D' => "0,1,2,3,9,12",
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
pub struct DisplaySegment {
    id: u8,
    lit: bool
} impl DisplaySegment {
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
            10 | 11 => FRAC_PI_2 - ANGLED_ANGLE,
            8 | 13 => FRAC_PI_2 + ANGLED_ANGLE,
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
    segmented_string: SegmentedDisplayString,
    display_assets: SegmentedDisplayAssets,
    commands: &mut Commands
) -> Entity {
    let chars = spawn_digits(
        commands, segmented_string.digit_count, segmented_string.font_size
    );
    for &char_entity in &chars {
        spawn_segments(
            commands, &display_assets, segmented_string.font_size,
            char_entity, segmented_string.casts_shadows
        );
    };
    let segmented_entity = commands.spawn((
        segmented_string,
        display_assets,
        transfrom,
        Visibility::Visible
    )).id();
    for char in chars {
        commands.entity(char).insert(ChildOf(segmented_entity));
    };
    segmented_entity
}

fn spawn_digits(
    commands: &mut Commands,
    digit_count: u8,
    font_size: f32
) -> Vec<Entity> {
    let mut digits: Vec<Entity> = Vec::with_capacity(digit_count as usize);
    let (start_x, dx) = if digit_count > 1 {
        let length = SegmentedDisplayString::get_rendered_length(digit_count, font_size);
        let step = length / digit_count as f32;
        let start = -(length / 2.0) + (font_size / 2.0);
        (start, step)
    } else {
        (0.0f32, 0.0f32)
    };
    for i in 0..digit_count {
        let entity = commands.spawn((
            SegmentDigit {
                char: '0',
                index: i
            },
            Transform::from_xyz(start_x + dx * i as f32, 0.0, 0.0),
            Visibility::Inherited
        )).id();
        digits.push(entity)
    };
    digits
}

fn spawn_segments(
    commands: &mut Commands,
    assets: &SegmentedDisplayAssets,
    font_size: f32,
    parent: Entity,
    casts_shadows: bool
) {
    for i in 0..14u8 {
        let segment = DisplaySegment {id: i, lit: false};
        let transform = DisplaySegment::get_transform(i, font_size);
        let mesh = if DisplaySegment::is_short(i) {
            assets.short_mesh.clone()
        } else {
            assets.long_mesh.clone()
        };
        let e = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(assets.unlit_material.clone()),
            segment,
            transform,
            ChildOf(parent),
            NotShadowReceiver,
            Visibility::Inherited
        )).id();
        if !casts_shadows {
            commands.entity(e).insert(NotShadowCaster);
        };
    };
}

pub fn update_segmented_strings(
    mut commands: Commands,
    segmented_string_query: Query<(&SegmentedDisplayString, &SegmentedDisplayAssets, &Children), Changed<SegmentedDisplayString>>,
    mut digit_query: Query<(&mut SegmentDigit, &Children)>,
    mut segment_query: Query<(&mut DisplaySegment)>
) {
    for (segmented_string, assets, string_children) in segmented_string_query {
        // this is for resolving when the string is not the same length as the number of digits spawned
        let d = segmented_string.digit_count as i8 - segmented_string.string.len() as i8;
        let cap = if d < 0 {
            segmented_string.string.len()
        } else {
            segmented_string.digit_count as usize
        };
        let mut character_truths: Vec<char> = Vec::with_capacity(cap);
        
        // "padding" the array with empty spaces will right-align text shorter than our char count
        if d > 0 {
            for _ in 0..d {
                character_truths.push(segmented_string.padding_char);
            };
        };
        
        // whereas just letting it go will mean anything longer than char count gets truncated
        // ... hypothetically
        for c in segmented_string.string.to_uppercase().chars() {
            character_truths.push(c);
        };
        // prep for digit iteration
        for &digit_entity in string_children {
            let (mut digit, digit_children) = 
                digit_query.get_mut(digit_entity).unwrap();
            let c = digit.char;
            
            // this checks to see, per character, if the digits are accurate to the string
            if character_truths[digit.index as usize] != c {
                digit.char = character_truths[digit.index as usize];
                
                // lookup the correct sequence to represent the character
                let seq = digit.get_sequence();
                for &segment_entity in digit_children {
                    let mut segment = segment_query.get_mut(segment_entity).unwrap();
                    let is_lit = segment.lit;
                    let should_be_lit = seq[segment.id as usize];
                    
                    // this is a real duct tape or WD40 kind of flowchart
                    if is_lit != should_be_lit {
                        if should_be_lit {  // if it should be lit, light it
                            segment.lit = true;
                            commands.entity(segment_entity).insert(MeshMaterial3d(assets.lit_material.clone()));
                        } else if is_lit {  // if not, unlight it
                            segment.lit = false;
                            commands.entity(segment_entity).insert(MeshMaterial3d(assets.unlit_material.clone()));
                        };
                    };
                };
            };
        };
    };
}
