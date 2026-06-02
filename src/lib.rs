#![cfg_attr(feature = "error_handling", feature(try_trait_v2))]

pub use bevy::*;

#[cfg(feature = "error_handling")]
pub mod error_handling;

pub mod prelude {
    pub use bevy::{
        ecs::{lifecycle::HookContext, world::DeferredWorld},
        prelude::*,
    };

    pub fn plugin(_: &mut App) {}
    pub fn plugins_in_modules(_: &mut App) {}
    #[cfg(feature = "error_handling")]
    pub use crate::error_handling::ToUnwrapResult;

    // pub use crate::gather::bindings::*;
    pub use crate::plugin_modules;

    #[allow(non_upper_case_globals)]
    pub const Vec3: fn(f32, f32, f32) -> Vec3 = |x, y, z| Vec3::new(x, y, z);
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