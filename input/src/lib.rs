#![feature(splat)]
#![feature(integer_casts)]
#![expect(incomplete_features)]
use std::any::TypeId;

use bevy::{
    input::{
        ButtonState, InputSystems,
        gamepad::{RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent},
        keyboard::KeyboardInput,
    },
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.insert_resource(Input {
        pressed: default(),
        held: default(),
        bindings: default(),

        default_dead_zone: 0.5,
        dead_zones: default(),
    })
    .add_systems(Startup, bindings)
    .add_systems(PreUpdate, input.in_set(InputSystems));
}

// TODO: Axis is just two 0.0..1.0s. Everything can be pressed. Pressed is triggered when the value leaves the dead zone, but not when it changes within the live zone.
#[derive(Resource)]
pub struct Input {
    pressed: foldhash::HashSet<InputSource>,
    held: foldhash::HashMap<InputSource, f32>,
    bindings: foldhash::HashMap<(TypeId, u8), Vec<InputSource>>,

    default_dead_zone: f32,
    dead_zones: foldhash::HashMap<InputSource, f32>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum InputSource {
    GamepadButton(GamepadButton),
    GamepadAxisPositive(GamepadAxis),
    GamepadAxisNegative(GamepadAxis),
    KeyCode(KeyCode),
}
impl From<KeyCode> for InputSource {
    fn from(value: KeyCode) -> Self {
        Self::KeyCode(value)
    }
}
impl From<GamepadButton> for InputSource {
    fn from(value: GamepadButton) -> Self {
        Self::GamepadButton(value)
    }
}

impl Input {
    pub fn event(&mut self, input_source: InputSource, value: f32) {
        let dead_zone = match self.dead_zones.get(&input_source) {
            Some(dead_zone) => *dead_zone,
            None => self.default_dead_zone,
        };

        if value >= dead_zone {
            match input_source {
                InputSource::GamepadAxisNegative(axis) => {
                    self.held.remove(&InputSource::GamepadAxisPositive(axis));
                }
                InputSource::GamepadAxisPositive(axis) => {
                    self.held.remove(&InputSource::GamepadAxisNegative(axis));
                }
                _ => (),
            }

            if self.held.insert(input_source, value).is_none() {
                self.pressed.insert(input_source);
            }
        } else {
            // Avoid situations in which an axis remains held when it shouldn't, due to going  the opposite way too quickly.
            match input_source {
                InputSource::GamepadAxisNegative(axis) => {
                    self.held.remove(&InputSource::GamepadAxisPositive(axis));
                }
                InputSource::GamepadAxisPositive(axis) => {
                    self.held.remove(&InputSource::GamepadAxisNegative(axis));
                }
                _ => (),
            }

            self.held.remove(&input_source);
        }
    }

    pub fn pressed<T: ActionTemplate>(&self) -> bool {
        T::Template::pressed(|i| {
            self.bindings
                .get(&(TypeId::of::<T>(), i))
                .and_then(|bindings| {
                    bindings
                        .iter()
                        .find_map(|binding| self.pressed.get(binding))
                })
                .is_some()
        })
    }
    pub fn held<T: ActionTemplate>(&self) -> <T::Template as Action>::Output {
        T::Template::held(|i| {
            self.bindings
                .get(&(TypeId::of::<T>(), i))
                .and_then(|bindings| bindings.iter().find_map(|binding| self.held.get(binding)))
                .copied()
        })
    }
}

fn input(
    mut input: ResMut<Input>,
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut gamepad_button_input: MessageReader<RawGamepadButtonChangedEvent>,
    mut gamepad_axis_input: MessageReader<RawGamepadAxisChangedEvent>,
) {
    input.bypass_change_detection().pressed.clear();

    for keyboard_input in keyboard_input.read() {
        match keyboard_input.state {
            ButtonState::Pressed => {
                input.event(InputSource::KeyCode(keyboard_input.key_code), 1.);
            }
            ButtonState::Released => {
                input.event(InputSource::KeyCode(keyboard_input.key_code), 0.);
            }
        }
    }

    for gamepad_input in gamepad_button_input.read() {
        input.event(
            InputSource::GamepadButton(gamepad_input.button),
            gamepad_input.value,
        );
    }

    for gamepad_input in gamepad_axis_input.read() {
        if gamepad_input.value.is_sign_positive() {
            input.event(
                InputSource::GamepadAxisPositive(gamepad_input.axis),
                gamepad_input.value,
            );
        } else {
            input.event(
                InputSource::GamepadAxisNegative(gamepad_input.axis),
                -gamepad_input.value,
            );
        }
    }
}

pub trait ActionTemplate: 'static {
    type Template: Action;
}

pub trait Action: Sync + Send + 'static {
    type Output;
    fn pressed(check: impl FnMut(u8) -> bool) -> bool;
    fn held(check: impl FnMut(u8) -> Option<f32>) -> Self::Output;
}
impl<T: Action> ActionTemplate for T {
    type Template = Self;
}

#[allow(unexpected_cfgs)]
impl Input {
    #[cfg(not(rust_analyzer))]
    pub fn bind<T: ActionTemplate>(&mut self, #[rustc_splat] binding: impl Binding<T::Template>) {
        binding.bind(|index, input_source| {
            self.bindings
                .entry((TypeId::of::<T>(), index))
                .or_default()
                .push(input_source);
        });
    }

    #[cfg(rust_analyzer)]
    pub extern "c" fn bind<T: ActionTemplate>(&mut self, mut args: ...);
}

pub trait Binding<T>: Sized {
    fn bind(self, single_bind: impl FnMut(u8, InputSource));
}
impl<T, I: Into<InputSource>> Binding<T> for (u8, I) {
    fn bind(self, mut single_bind: impl FnMut(u8, InputSource)) {
        single_bind(self.0, self.1.into());
    }
}

fn bindings(mut input: ResMut<Input>) {
    input.bind::<UiMove>(0, KeyCode::KeyA);
    input.bind::<UiMove>(1, KeyCode::KeyD);

    input.bind::<UiMove>(0, GamepadButton::DPadLeft);
    input.bind::<UiMove>(1, GamepadButton::DPadRight);

    input.bind::<UiMove>(0, InputSource::GamepadAxisNegative(GamepadAxis::LeftStickX));
    input.bind::<UiMove>(1, InputSource::GamepadAxisPositive(GamepadAxis::LeftStickX));

    input.bind::<SoulMove>(0, InputSource::GamepadAxisPositive(GamepadAxis::LeftStickY));
    input.bind::<SoulMove>(1, InputSource::GamepadAxisNegative(GamepadAxis::LeftStickX));
    input.bind::<SoulMove>(2, InputSource::GamepadAxisPositive(GamepadAxis::LeftStickX));
    input.bind::<SoulMove>(3, InputSource::GamepadAxisNegative(GamepadAxis::LeftStickY));

    input.bind::<SoulMove>(Wasd);
    input.bind::<SoulMove>(DPad);

    input.bind::<Confirm>(KeyCode::Enter);
}

struct Wasd;
impl From<Wasd> for [InputSource; 4] {
    fn from(_: Wasd) -> Self {
        [
            InputSource::KeyCode(KeyCode::KeyW),
            InputSource::KeyCode(KeyCode::KeyA),
            InputSource::KeyCode(KeyCode::KeyD),
            InputSource::KeyCode(KeyCode::KeyS),
        ]
    }
}

struct DPad;
impl From<DPad> for [InputSource; 4] {
    fn from(_: DPad) -> Self {
        [
            InputSource::GamepadButton(GamepadButton::DPadUp),
            InputSource::GamepadButton(GamepadButton::DPadLeft),
            InputSource::GamepadButton(GamepadButton::DPadRight),
            InputSource::GamepadButton(GamepadButton::DPadDown),
        ]
    }
}

impl<T: Into<InputSource>> Binding<bool> for (T,) {
    fn bind(self, mut single_bind: impl FnMut(u8, InputSource)) {
        single_bind(0, self.0.into());
    }
}
impl Action for bool {
    type Output = Self;

    fn pressed(mut check: impl FnMut(u8) -> bool) -> bool {
        check(0)
    }

    fn held(mut check: impl FnMut(u8) -> Option<f32>) -> Self::Output {
        check(0).is_some()
    }
}

impl<T: Into<[InputSource; 4]>> Binding<Vec2> for (T,) {
    fn bind(self, mut single_bind: impl FnMut(u8, InputSource)) {
        for (index, input_source) in self.0.into().into_iter().enumerate() {
            single_bind(index.strict_cast(), input_source);
        }
    }
}
impl Action for Vec2 {
    type Output = Self;

    fn pressed(mut check: impl FnMut(u8) -> bool) -> bool {
        check(0) || check(1) || check(2) || check(3)
    }

    fn held(mut check: impl FnMut(u8) -> Option<f32>) -> Self::Output {
        let mut output = Vec2::ZERO;

        if let Some(value) = check(0) {
            output.y += value;
        }
        if let Some(value) = check(1) {
            output.x -= value;
        }
        if let Some(value) = check(2) {
            output.x += value;
        }
        if let Some(value) = check(3) {
            output.y -= value;
        }

        output.normalize_or_zero()
    }
}

#[derive(Debug)]
pub enum UiMove {
    Backwards = -1,
    None = 0,
    Forwards = 1,
}
impl Action for UiMove {
    type Output = Self;

    fn pressed(mut check: impl FnMut(u8) -> bool) -> bool {
        check(0) ^ check(1)
    }
    fn held(mut check: impl FnMut(u8) -> Option<f32>) -> Self::Output {
        match (check(0).is_some(), check(1).is_some()) {
            (true, false) => Self::Backwards,
            (false, true) => Self::Forwards,
            _ => Self::None,
        }
    }
}

pub struct SoulMove;
impl ActionTemplate for SoulMove {
    type Template = Vec2;
}

pub struct Confirm;
impl ActionTemplate for Confirm {
    type Template = bool;
}
