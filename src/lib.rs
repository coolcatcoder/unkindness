#![feature(custom_inner_attributes)]
#![feature(proc_macro_hygiene)]
#![feature(final_associated_functions)]

pub use procedural_macros;
pub use procedural_macros::prelude;

//#[procedural_macros::plugin]
mod tester;

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

    #[allow(non_upper_case_globals)]
    pub const Vec3: fn(f32, f32, f32) -> Vec3 = Vec3::new;
    #[allow(non_upper_case_globals)]
    pub const Vec2: fn(f32, f32) -> Vec2 = Vec2::new;

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

trait Behaviour {
    fn once();
}

#[macro_export]
macro_rules! prepend_final {
    ($($any:tt)*) => {
        final $($any)*
    };
}

#[macro_export]
macro_rules! Behaviour {
    () => {};
}
