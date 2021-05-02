use bevy::prelude::*;

use crate::config::Config;
use std::hash::Hash;

#[macro_export]
macro_rules! keys_basic {
    ($matrix: expr => $( $key: ident is $( $state: pat )|+ ),+) => {
        {
            use InputState::*;
            $(
                matches!($matrix . $key, $( $state )|*)
            )&&*
        }
    };
}

///This macro is used to create a DSL for checking what keys are pressed.
/// this lets you define tekken-like keybinds
#[macro_export]
macro_rules! keys {
    ($matrix: expr => $( $keys: ident )|+ is $( $state: pat )|+ $(,)?) => {
        {
            keys!(@recurse_and $matrix => $( $keys )|+ is $( $state )|+)
        }
    };

    ($matrix: expr => $( $keys: ident )|+ is $( $state: pat )|+ , $( $( $keys2: ident )|+ is $( $state2: pat )|+ ),* $(,)?) => {
        {
            keys!(@recurse_and $matrix => $( $keys )|+ is $( $state )|+)
            &&
            keys!($matrix => $( $( $keys2 )|+ is $( $state2 )|+ ),*)
        }
    };

    (@recurse_and $matrix: expr => $( $keys: ident )|+ is $( $state: pat )|+) => {
        {
            use InputState::*;
            keys!(@in @or $matrix => $( $keys );* is $( $state )|+)
        }
    };

    (@in @or $matrix: expr => $key: ident ; $( $keys: ident );* is $( $state: pat )|+) => {
        matches!($matrix . $key, $( $state )|+)
            ||
        keys!(@in @or $matrix => $( $keys )* is $( $state )|+)
    };

    (@in @or $matrix: expr => $key: ident is $( $state: pat )|+) => {
        matches!($matrix . $key, $( $state )|+)
    };
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum InputEvent {
    LRClick,
    BClick,
    FClick,
    FSpec1,
}

pub enum InputState {
    Unpressed,
    JustPressed,
    JustReleased,
    Held,
}

pub struct InputMatrix {
    pub back: InputState,
    pub forward: InputState,
    pub left: InputState,
    pub right: InputState,
    pub dash: InputState,

    pub spec1: InputState,
    pub spec2: InputState,
    pub click: InputState,
}

pub fn input_matrix(
    c: &Config,
    key_in: &Input<KeyCode>,
    mouse_in: &Input<MouseButton>,
) -> InputMatrix {
    fn do_match<T: Copy + Eq + Hash>(i: &Input<T>, k: T) -> InputState {
        use InputState::*;
        match (i.just_pressed(k), i.just_released(k), i.pressed(k)) {
            (true, _, _) => JustPressed,
            (_, true, _) => JustReleased,
            (_, _, true) => Held,
            (_, _, _) => Unpressed,
        }
    }

    InputMatrix {
        back: do_match(&key_in, c.movement[2]),
        forward: do_match(&key_in, c.movement[0]),
        left: do_match(&key_in, c.movement[1]),
        right: do_match(&key_in, c.movement[3]),
        dash: do_match(&key_in, c.dash),

        spec1: do_match(&key_in, c.specials[0]),
        spec2: do_match(&key_in, c.specials[1]),

        click: do_match(&mouse_in, MouseButton::Left),
    }
}

fn update(
    //mut commands: Commands,
    //time: Res<Time>,
    config: Res<Config>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,

    mut input_events: EventWriter<InputEvent>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    //mut player_query: Query<(&crate::CameraOrientation, &Transform)>,
) {
    let key_matrix = input_matrix(&config, &keyboard_input, &mouse_input);

    if keys!(key_matrix =>
        click is JustPressed,
        left | right is Held | JustPressed,
    ) {
        input_events.send(InputEvent::LRClick);
    } else if keys!(key_matrix =>
        click is JustPressed,
        back is Held | JustPressed,
    ) {
        input_events.send(InputEvent::BClick);
    } else if keys!(key_matrix =>
        dash is JustPressed,
        forward is Held,
    ) {

    } else if keys!(key_matrix =>
        spec1 is JustPressed,
        forward is Held | JustPressed,
    ) {
        input_events.send(InputEvent::FSpec1);
    }
}

fn setup() {}

pub fn build(app: &mut AppBuilder) {
    app
        //.init_resource::<>()
        //.add_resource(NetworkingTimer(Timer::from_seconds(1.0 / 120.0, true)))
        .add_startup_system(setup.system())
        .add_system(update.system())
        .add_event::<InputEvent>();
}
