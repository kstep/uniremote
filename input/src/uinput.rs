use std::{collections::HashMap, sync::Mutex};

use evdev::{
    AttributeSet, EventType, InputEvent, KeyCode, RelativeAxisCode, uinput::VirtualDevice,
};
use unicase::UniCase;

use crate::{InputBackend, InputError, MouseButton};

pub struct UInputBackend {
    keyboard_device: Mutex<VirtualDevice>,
    mouse_device: Mutex<VirtualDevice>,
    key_map: HashMap<UniCase<&'static str>, KeyCode>,
}

impl UInputBackend {
    pub fn new() -> Result<Self, InputError> {
        let keyboard_device = Mutex::new(Self::create_keyboard_device()?);
        let mouse_device = Mutex::new(Self::create_mouse_device()?);
        let key_map = Self::build_key_map();

        Ok(Self {
            keyboard_device,
            mouse_device,
            key_map,
        })
    }

    fn create_keyboard_device() -> Result<VirtualDevice, InputError> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for key_code in KeyCode::KEY_ESC.code()..=KeyCode::KEY_MICMUTE.code() {
            keys.insert(KeyCode::new(key_code));
        }

        VirtualDevice::builder()
            .map_err(|e| InputError::InitError(e.to_string()))?
            .name("UniRemote Virtual Keyboard")
            .with_keys(&keys)
            .map_err(|e| InputError::InitError(e.to_string()))?
            .build()
            .map_err(|e| InputError::InitError(e.to_string()))
    }

    fn create_mouse_device() -> Result<VirtualDevice, InputError> {
        let mut buttons = AttributeSet::<KeyCode>::new();
        buttons.insert(KeyCode::BTN_LEFT);
        buttons.insert(KeyCode::BTN_RIGHT);
        buttons.insert(KeyCode::BTN_MIDDLE);

        let mut rel_axes = AttributeSet::<RelativeAxisCode>::new();
        rel_axes.insert(RelativeAxisCode::REL_X);
        rel_axes.insert(RelativeAxisCode::REL_Y);

        VirtualDevice::builder()
            .map_err(|e| InputError::InitError(e.to_string()))?
            .name("UniRemote Virtual Mouse")
            .with_keys(&buttons)
            .map_err(|e| InputError::InitError(e.to_string()))?
            .with_relative_axes(&rel_axes)
            .map_err(|e| InputError::InitError(e.to_string()))?
            .build()
            .map_err(|e| InputError::InitError(e.to_string()))
    }

    fn build_key_map() -> HashMap<UniCase<&'static str>, KeyCode> {
        macro_rules! key_map {
            ($($key:expr => $code:ident),* $(,)?) => {{
                let mut map = HashMap::new();
                $(
                    map.insert(UniCase::new($key), KeyCode::$code);
                )*
                map
            }};
        }

        key_map![
            // Letters
            "a" => KEY_A, "b" => KEY_B, "c" => KEY_C,
            "d" => KEY_D, "e" => KEY_E, "f" => KEY_F,
            "g" => KEY_G, "h" => KEY_H, "i" => KEY_I,
            "j" => KEY_J, "k" => KEY_K, "l" => KEY_L,
            "m" => KEY_M, "n" => KEY_N, "o" => KEY_O,
            "p" => KEY_P, "q" => KEY_Q, "r" => KEY_R,
            "s" => KEY_S, "t" => KEY_T, "u" => KEY_U,
            "v" => KEY_V, "w" => KEY_W, "x" => KEY_X,
            "y" => KEY_Y, "z" => KEY_Z,

            // Numbers
            "0" => KEY_0, "1" => KEY_1, "2" => KEY_2,
            "3" => KEY_3, "4" => KEY_4, "5" => KEY_5,
            "6" => KEY_6, "7" => KEY_7, "8" => KEY_8,
            "9" => KEY_9,

            // Function keys
            "f1" => KEY_F1, "f2" => KEY_F2, "f3" => KEY_F3,
            "f4" => KEY_F4, "f5" => KEY_F5, "f6" => KEY_F6,
            "f7" => KEY_F7, "f8" => KEY_F8, "f9" => KEY_F9,
            "f10" => KEY_F10, "f11" => KEY_F11, "f12" => KEY_F12,

            // Common keys
            "space" => KEY_SPACE,
            "enter" => KEY_ENTER, "return" => KEY_ENTER,
            "tab" => KEY_TAB,
            "escape" => KEY_ESC, "esc" => KEY_ESC,
            "menu" => KEY_MENU,
            "backspace" => KEY_BACKSPACE, "back" => KEY_BACKSPACE,
            "insert" => KEY_INSERT,
            "delete" => KEY_DELETE,

            // Arrow keys
            "up" => KEY_UP, "down" => KEY_DOWN,
            "left" => KEY_LEFT, "right" => KEY_RIGHT,
            "pageup" => KEY_PAGEUP, "pagedown" => KEY_PAGEDOWN,
            "scrollup" => KEY_SCROLLUP, "scrolldown" => KEY_SCROLLDOWN,
            "home" => KEY_HOME, "end" => KEY_END,

            // Modifiers
            "shift" => KEY_LEFTSHIFT,
            "ctrl" => KEY_LEFTCTRL, "control" => KEY_LEFTCTRL,
            "alt" => KEY_LEFTALT, "lalt" => KEY_LEFTALT,
            "ralt" => KEY_RIGHTALT,
            "super" => KEY_LEFTMETA, "lsuper" => KEY_LEFTMETA,
            "rsuper" => KEY_RIGHTMETA,
            "meta" => KEY_LEFTMETA, "lmeta" => KEY_LEFTMETA,
            "rmeta" => KEY_RIGHTMETA,
            "win" => KEY_LEFTMETA, "lwin" => KEY_LEFTMETA,
            "rwin" => KEY_RIGHTMETA,
            "cmd" => KEY_LEFTCTRL,

            // Media keys
            "volumeup" => KEY_VOLUMEUP, "volume_up" => KEY_VOLUMEUP,
            "volumedown" => KEY_VOLUMEDOWN, "volume_down" => KEY_VOLUMEDOWN,
            "volumemute" => KEY_MUTE, "volume_mute" => KEY_MUTE,
            "mediaplaypause" => KEY_PLAYPAUSE,
            "mediastop" => KEY_STOP,
            "medianext" => KEY_NEXT,
            "mediaprevious" => KEY_PREVIOUS,

            // Keypad
            "oem_plus" => KEY_KPPLUS, "oem_minus" => KEY_KPMINUS,
            "oem_0" => KEY_KP0, "oem_1" => KEY_KP1,
            "oem_2" => KEY_KP2, "oem_3" => KEY_KP3,
            "oem_4" => KEY_KP4, "oem_5" => KEY_KP5,
            "oem_6" => KEY_KP6, "oem_7" => KEY_KP7,
            "oem_8" => KEY_KP8, "oem_9" => KEY_KP9,
            "plus" => KEY_KPPLUS, "minus" => KEY_KPMINUS,
            "divide" => KEY_KPSLASH, "multiply" => KEY_KPASTERISK,
            "decimal" => KEY_KPDOT, "separator" => KEY_KPCOMMA,
            "kpplus" => KEY_KPPLUS, "kpminus" => KEY_KPMINUS,
            "kpslash" => KEY_KPSLASH, "kpasterisk" => KEY_KPASTERISK,
            "kpdot" => KEY_KPDOT, "kpcomma" => KEY_KPCOMMA,
            "num0" => KEY_KP0, "num1" => KEY_KP1,
            "num2" => KEY_KP2, "num3" => KEY_KP3,
            "num4" => KEY_KP4, "num5" => KEY_KP5,
            "num6" => KEY_KP6, "num7" => KEY_KP7,
            "num8" => KEY_KP8, "num9" => KEY_KP9,
        ]
    }

    fn get_key(&self, key: &str) -> Result<KeyCode, InputError> {
        self.key_map
            .get(&UniCase::new(key))
            .copied()
            .ok_or_else(|| InputError::SendError(format!("unknown key: {key}")))
    }

    fn emit_key(&self, key: KeyCode, value: i32) -> Result<(), InputError> {
        let events = [
            InputEvent::new(EventType::KEY.0, key.code(), value),
            InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
        ];

        self.keyboard_device
            .lock()
            .unwrap()
            .emit(&events)
            .map_err(|e| InputError::SendError(e.to_string()))
    }

    fn emit_mouse_button(&self, button: MouseButton, value: i32) -> Result<(), InputError> {
        let key = match button {
            MouseButton::Left => KeyCode::BTN_LEFT,
            MouseButton::Right => KeyCode::BTN_RIGHT,
            MouseButton::Middle => KeyCode::BTN_MIDDLE,
        };

        let events = [
            InputEvent::new(EventType::KEY.0, key.code(), value),
            InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
        ];

        self.mouse_device
            .lock()
            .unwrap()
            .emit(&events)
            .map_err(|e| InputError::SendError(e.to_string()))
    }
}

impl InputBackend for UInputBackend {
    fn is_key(&self, key: &str) -> bool {
        self.key_map.contains_key(&UniCase::new(key))
    }

    fn is_modifier(&self, key: &str) -> bool {
        const MODIFIERS: &[&str] = &[
            "shift", "ctrl", "control", "alt", "lalt", "ralt", "super", "lsuper", "rsuper", "meta",
            "lmeta", "rmeta", "win", "lwin", "rwin", "cmd",
        ];
        MODIFIERS.iter().any(|&m| key.eq_ignore_ascii_case(m))
    }

    fn key_press(&self, key: &str) -> Result<(), InputError> {
        let key = self.get_key(key)?;
        self.emit_key(key, 1)
    }

    fn key_release(&self, key: &str) -> Result<(), InputError> {
        let key = self.get_key(key)?;
        self.emit_key(key, 0)
    }

    fn key_click(&self, key: &str) -> Result<(), InputError> {
        self.key_press(key)?;
        self.key_release(key)
    }

    fn mouse_move(&self, dx: i32, dy: i32) -> Result<(), InputError> {
        let events = [
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, dx),
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, dy),
            InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
        ];

        self.mouse_device
            .lock()
            .unwrap()
            .emit(&events)
            .map_err(|e| InputError::SendError(e.to_string()))
    }

    fn mouse_button_press(&self, button: MouseButton) -> Result<(), InputError> {
        self.emit_mouse_button(button, 1)
    }

    fn mouse_button_release(&self, button: MouseButton) -> Result<(), InputError> {
        self.emit_mouse_button(button, 0)
    }

    fn mouse_button_click(&self, button: MouseButton) -> Result<(), InputError> {
        self.mouse_button_press(button)?;
        self.mouse_button_release(button)
    }
}
