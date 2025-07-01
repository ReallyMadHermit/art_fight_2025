use bevy::prelude::*;

// MAIN
fn main() {
    println!("hello world!");
    // let hues = MaterialWizard::generate_hue_vec(16);
    // for h in hues {
    //     println!("{}", h);
    // };
    App::new()
        .add_plugins(DefaultPlugins)
        .run();
}