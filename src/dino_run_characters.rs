use bevy::prelude::*;
use crate::dino_run_mechanics::PlayerEntity;

enum LegVariant{
    Izella
}

enum LegPart {
    LeftFoot,
    RightFoot,
    LeftKnee,
    RightKnee,
    Hip
}

fn spawn_izella_legs(
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    
}