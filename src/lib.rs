pub use bevy::*;

#[cfg(feature = "input")]
pub use input;

#[cfg(test)]
mod tests;

#[macro_export]
macro_rules! prelude_old {
    () => {
        $crate::prelude!(pub(super));
    };

    ($visibility:vis) => {
        use $crate::prelude::*;

        #[$crate::procedural_macros::module_name(!"main.rs")]
        #[$crate::procedural_macros::module_name(!"lib.rs")]
        $visibility struct Module;
    };
}

pub mod prelude {
    #[cfg(feature = "error_handling")]
    pub use duck_back::Else;

    pub use bevy::{
        ecs::{lifecycle::HookContext, world::DeferredWorld},
        prelude::*,
    };

    pub use crate::plugin_modules;

    #[expect(non_snake_case)]
    pub fn Vec3(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new(x, y, z)
    }

    #[expect(non_snake_case)]
    pub fn Vec2(x: f32, y: f32) -> Vec2 {
        Vec2::new(x, y)
    }

    pub trait Behaviour {
        type A;
        const B: Self::A;
    }
}

#[macro_export]
macro_rules! plugin_modules {
    ($($visibility:vis $module:ident),*) => {
        $(
            $visibility mod $module;
        )*

        pub fn plugins_in_modules(app: &mut App) {
            $(
                $module::plugin(app);
                $module::plugins_in_modules(app);
            )*
        }
    };
}
