//#![procedural_macros::no_effect]
//#![procedural_macros::plugin]
// use super::Behaviour;
use bevy::{
    prelude::*,
    scene::{ResolveContext, ResolvedScene},
};

use procedural_macros::scene;

#[derive(Component, Default, Clone)]
struct Bad;
impl Bad {
    fn bad(self) -> Self {
        self
    }
}

fn Hah() -> Bad {
    // Useful! Put this in a closure, so you don't have to call it, and you can get the generics you need!
    Bad { ..todo!() }
}

#[inline(always)]
fn cool_get_or_insert_template<
    'a,
    T: Template<Output: Component> + Default + Send + Sync + 'static,
>(
    _: fn() -> T,
    scene: &'a mut ResolvedScene,
    context: &mut ResolveContext,
) -> &'a mut T {
    scene.get_or_insert_template(context)
}

fn normal() -> impl SceneList {
    //bsn_list!(Hah())
}

fn expanded() -> impl SceneList {
    let _res = ::bevy::scene::SceneListScope(
        (::bevy::scene::EntityScene(
            (::bevy::scene::SceneFunction(move |_context, _scene| {
                let _ = _scene.get_or_insert_template:: <<Transform as ::bevy::ecs::template::FromTemplate> ::Template>(_context);
            })),
        )),
    );
    _res
}

fn expanded_with_scale() -> impl SceneList {
    let _expr0 = { Vec3::new(0., 0., 0.) }.into();
    let _res = ::bevy::scene::SceneListScope(
        (::bevy::scene::EntityScene(
            (::bevy::scene::SceneFunction(move |_context, _scene| {
                let __value = _scene.get_or_insert_template:: <<Transform as ::bevy::ecs::template::FromTemplate> ::Template>(_context);
                __value.scale = _expr0;
            })),
        )),
    );
    _res
}

fn expanded_with_name() -> impl SceneList {
    static _CALL_ID: ::bevy::scene::macro_utils::CallCounter =
        ::bevy::scene::macro_utils::CallCounter::new();
    let _call_id = _CALL_ID.increment();
    let _res = ::bevy::scene::SceneListScope(
        (::bevy::scene::EntityScene(
            (::bevy::scene::SceneFunction(move |_context, _scene| {
                ::bevy::scene::NameEntityReference {
                    name: ::bevy::ecs::name::Name("Name".into()),
                    reference: ::bevy::ecs::template::SceneEntityReference::new(
                        ("", 1usize, 1usize),
                        0usize,
                        _call_id,
                    ),
                }
                .resolve_inline(_context, _scene);
                let _ = _scene.get_or_insert_template:: <<Transform as ::bevy::ecs::template::FromTemplate> ::Template>(_context);
            })),
        )),
    );
    _res
}

struct TupleTester(u32, ());

fn tester() -> impl SceneList {
    scene!(
        a(TupleTester(0, (), some_function_call::<(), ()>)),
        bad(),
        (),
        long_name_right_here(a, b, c, d, bad,),
        (what_point, do_we),
    )
}

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
