//#![procedural_macros::no_effect]
//#![procedural_macros::plugin]
// use super::Behaviour;
// use bevy::prelude::*;

// struct Module;

// impl Behaviour for Module {
//     fn once(mut commands: Commands) {
//         let bad = 0;
//     }
// }

// impl bevy::prelude::Plugin for Module {
//     fn build(&self, app: &mut bevy::app::App) {
//         //app.add_systems(schedule, systems)
//     }
// }

// fn bad() {
//     let bad: f32 = 1.;
//     //bad = 0.;
// }

//procedural_macros::prelude!();

//struct Module;

//impl Module {}

// use procedural_macros::bad;

// struct Module {}

// trait Behaviour {
//     fn once();
// }

// #[behaviour]
// impl Behaviour for Module {
//     fn once() {}
// }
