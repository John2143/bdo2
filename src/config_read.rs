//!This is used to parse a list of strings into a list of keybinds
//!
//!```
//!struct SomeConfig {
//!    pub sens: f32,
//!
//!    #[serde(deserialize_with = "keybind_list")]
//!    pub movement: [KeyCode; 4],
//!    #[serde(deserialize_with = "keybind")]
//!    pub jump: KeyCode,
//!}
//!
//!let config = r#"
//!---
//!movement: [W, A, S, D]
//!jump: Space
//!sens: 2.2
//!"#;
//!let config: SomeConfig = serde_yaml::from_str(&config).unwrap();
//!use KeyCode::*;
//!assert_eq!(&config.movement, &[W, A, S, D]);
//!assert_eq!(&config.jump, &Space);
//!```

use std::borrow::Cow;

use bevy::prelude::*;
use serde::{de, Deserializer};

///A list of `N` keybinds. takes each element in the list as a string, parses to
///`KeyCode`
///```
///struct SomeConfig {
///    #[serde(deserialize_with = "keybind_list")]
///    pub movement: [KeyCode; 4],
///}
///```
pub fn keybind_list<'de, D: Deserializer<'de>, const N: usize>(
    de: D,
) -> Result<[KeyCode; N], D::Error> {
    de.deserialize_any(KeybindVisitor)
}

///A single keybind, either represented as a list with a single element or a string
///```
///struct SomeConfig {
///    #[serde(deserialize_with = "keybind")]
///    pub jump: KeyCode,
///}
///```
pub fn keybind<'de, D: Deserializer<'de>>(de: D) -> Result<KeyCode, D::Error> {
    match de.deserialize_any(KeybindVisitor) {
        Ok([keybind]) => return Ok(keybind),
        Err(e) => return Err(e),
    }
}

struct KeybindVisitor<const N: usize>;
impl<'de, const N: usize> de::Visitor<'de> for KeybindVisitor<N> {
    type Value = [KeyCode; N];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "sequence of {} keybinds", N)
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        //Treat strings as arrays with 1 length and fail if the target length is not 1
        match N {
            1 => Ok([str_to_keybind(&s)?; N]),
            _ => Err(E::custom(format!(
                "This field should be a list of {} binds, not a single bind",
                N
            ))),
        }
    }

    fn visit_seq<A>(self, mut v: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut keycodes = [KeyCode::E; N];
        for i in 0..N {
            let next: Option<Cow<str>> = v.next_element()?;
            let next = match next {
                Some(n) => n,
                None => return Err(de::Error::invalid_length(i, &self)),
            };
            keycodes[i] = str_to_keybind(&next)?;
        }
        Ok(keycodes)
    }
}

///expand a list of types to a map with the name stringifed
///keycode_list!{A, B, C} = HashMap [("A", A), ("B", B), ("C", C)]
macro_rules! keycode_list {
    ( $( $x:ident,)+ ) => {{
        &[
        $(
            (stringify!($x), $x),
        )*
        ]
    }}
}

///Available bevy keycodes in a hashmap so it can be specified inside configs
const KEYCODES: &[(&str, KeyCode)] = {
    use KeyCode::*;
    keycode_list! { Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Escape, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24, Snapshot, Scroll, Pause, Insert, Home, Delete, End, PageDown, PageUp, Left, Up, Right, Down, Back, Return, Space, Compose, Caret, Numlock, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9, AbntC1, AbntC2, NumpadAdd, Apostrophe, Apps, Asterix, Plus, At, Ax, Backslash, Calculator, Capital, Colon, Comma, Convert, NumpadDecimal, NumpadDivide, Equals, Grave, Kana, Kanji, LAlt, LBracket, LControl, LShift, LWin, Mail, MediaSelect, MediaStop, Minus, NumpadMultiply, Mute, MyComputer, NavigateForward, NavigateBackward, NextTrack, NoConvert, NumpadComma, NumpadEnter, NumpadEquals, OEM102, Period, PlayPause, Power, PrevTrack, RAlt, RBracket, RControl, RShift, RWin, Semicolon, Slash, Sleep, Stop, NumpadSubtract, Sysrq, Tab, Underline, Unlabeled, VolumeDown, VolumeUp, Wake, WebBack, WebFavorites, WebForward, WebHome, WebRefresh, WebSearch, WebStop, Yen, Copy, Paste, Cut, }
};

pub fn str_to_keybind<'a, E: de::Error>(s: &str) -> Result<KeyCode, E> {
    let map_entry = KEYCODES
        .iter()
        .find_map(|(name, value)| (name == &s).then_some(value));

    match map_entry {
        Some(key) => Ok(*key),
        None => Err(E::custom(format!("{} is not the name of a valid key", &s))),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use serde::Deserialize;

    #[cfg(test)]
    #[derive(Deserialize, Debug)]
    struct ConfigTest {
        pub zoom_sens: f32,
        #[serde(deserialize_with = "keybind_list")]
        pub movement: [KeyCode; 4],
        #[serde(deserialize_with = "keybind")]
        pub jump: KeyCode,
        #[serde(deserialize_with = "keybind")]
        pub crouch: KeyCode,
    }

    #[test]
    fn reads_yaml_keybind_configs() {
        let default_config = r#"---
movement: ["W", "A", "S", "D"]
jump: "Space"
crouch: ["J"]
zoom_sens: 2.2
        "#;
        let config: ConfigTest = serde_yaml::from_str(&default_config).unwrap();
        use KeyCode::*;
        assert_eq!(&config.movement, &[W, A, S, D]);
        assert_eq!(&config.jump, &Space);
        assert_eq!(&config.crouch, &J);
    }

    #[test]
    fn fails_to_read_bad_config_long() {
        let default_config = r#"---
movement: ["W", "A", "S", "D", "F"]
        "#;
        let err = serde_yaml::from_str::<ConfigTest>(&default_config).unwrap_err();
        assert_eq!(
            &err.to_string(),
            "movement: invalid length 5, expected sequence of 4 elements at line 2 column 11"
        );
    }

    #[test]
    fn fails_to_read_bad_config_short() {
        let default_config = r#"---
movement: ["W", "A"]
        "#;
        let err = serde_yaml::from_str::<ConfigTest>(&default_config).unwrap_err();
        assert_eq!(
            &err.to_string(),
            "movement: invalid length 2, expected sequence of 4 keybinds at line 2 column 11"
        );
    }

    #[test]
    fn fails_to_read_bad_config_list_in_single() {
        let default_config = r#"---
jump: ["W", "A"]
        "#;
        let err = serde_yaml::from_str::<ConfigTest>(&default_config).unwrap_err();
        assert_eq!(
            &err.to_string(),
            "jump: invalid length 2, expected sequence of 1 element at line 2 column 7"
        );
    }

    #[test]
    fn fails_invalid_bind() {
        let default_config = r#"---
jump: "NOTKEY"
        "#;
        let err = serde_yaml::from_str::<ConfigTest>(&default_config).unwrap_err();
        assert_eq!(
            &err.to_string(),
            "jump: NOTKEY is not the name of a valid key at line 2 column 7"
        );
    }

    #[test]
    fn quotes_optional() {
        let default_config = r#"---
movement: [W, A, S, D]
jump: Space
crouch: [J]
zoom_sens: 2.2
        "#;
        let config: ConfigTest = serde_yaml::from_str(&default_config).unwrap();
        use KeyCode::*;
        assert_eq!(&config.movement, &[W, A, S, D]);
        assert_eq!(&config.jump, &Space);
        assert_eq!(&config.crouch, &J);
    }
}
