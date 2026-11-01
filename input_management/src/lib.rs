#![feature(integer_casts)]
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
    pub fn bind<T: ActionTemplate>(&mut self, binding: impl Binding<T::Template>) {
        binding.bind(|index, input_source| {
            self.bindings
                .entry((TypeId::of::<T>(), index))
                .or_default()
                .push(input_source);
        });
    }
}

pub trait InputSourceArray<const LENGTH: usize> {
    fn into(self) -> [InputSource; LENGTH];
}

pub trait Binding<T>: Sized {
    fn bind(self, single_bind: impl FnMut(u8, InputSource));
}
impl<T, I: Into<InputSource>> Binding<T> for (u8, I) {
    fn bind(self, mut single_bind: impl FnMut(u8, InputSource)) {
        single_bind(self.0, self.1.into());
    }
}

pub struct Wasd;
impl InputSourceArray<4> for Wasd {
    fn into(self) -> [InputSource; 4] {
        [
            InputSource::KeyCode(KeyCode::KeyW),
            InputSource::KeyCode(KeyCode::KeyA),
            InputSource::KeyCode(KeyCode::KeyD),
            InputSource::KeyCode(KeyCode::KeyS),
        ]
    }
}

pub struct DPad;
impl InputSourceArray<4> for DPad {
    fn into(self) -> [InputSource; 4] {
        [
            InputSource::GamepadButton(GamepadButton::DPadUp),
            InputSource::GamepadButton(GamepadButton::DPadLeft),
            InputSource::GamepadButton(GamepadButton::DPadRight),
            InputSource::GamepadButton(GamepadButton::DPadDown),
        ]
    }
}

impl<T: Into<InputSource>> Binding<bool> for T {
    fn bind(self, mut single_bind: impl FnMut(u8, InputSource)) {
        single_bind(0, self.into());
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

impl<T: InputSourceArray<4>> Binding<Vec2> for T {
    fn bind(self, mut single_bind: impl FnMut(u8, InputSource)) {
        for (index, input_source) in self.into().into_iter().enumerate() {
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
