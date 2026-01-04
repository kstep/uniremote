use std::{collections::HashMap, sync::Mutex};

use evdev::{
    AttributeSet, EventType, InputEvent, KeyCode, RelativeAxisCode, uinput::VirtualDevice,
};

use crate::{InputBackend, InputError, MouseButton};

pub struct UInputBackend {
    keyboard_device: Mutex<VirtualDevice>,
    mouse_device: Mutex<VirtualDevice>,
    key_map: HashMap<String, KeyCode>,
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

    fn build_key_map() -> HashMap<String, KeyCode> {
        let mut map = HashMap::new();

        // Letters
        map.insert("a".to_string(), KeyCode::KEY_A);
        map.insert("b".to_string(), KeyCode::KEY_B);
        map.insert("c".to_string(), KeyCode::KEY_C);
        map.insert("d".to_string(), KeyCode::KEY_D);
        map.insert("e".to_string(), KeyCode::KEY_E);
        map.insert("f".to_string(), KeyCode::KEY_F);
        map.insert("g".to_string(), KeyCode::KEY_G);
        map.insert("h".to_string(), KeyCode::KEY_H);
        map.insert("i".to_string(), KeyCode::KEY_I);
        map.insert("j".to_string(), KeyCode::KEY_J);
        map.insert("k".to_string(), KeyCode::KEY_K);
        map.insert("l".to_string(), KeyCode::KEY_L);
        map.insert("m".to_string(), KeyCode::KEY_M);
        map.insert("n".to_string(), KeyCode::KEY_N);
        map.insert("o".to_string(), KeyCode::KEY_O);
        map.insert("p".to_string(), KeyCode::KEY_P);
        map.insert("q".to_string(), KeyCode::KEY_Q);
        map.insert("r".to_string(), KeyCode::KEY_R);
        map.insert("s".to_string(), KeyCode::KEY_S);
        map.insert("t".to_string(), KeyCode::KEY_T);
        map.insert("u".to_string(), KeyCode::KEY_U);
        map.insert("v".to_string(), KeyCode::KEY_V);
        map.insert("w".to_string(), KeyCode::KEY_W);
        map.insert("x".to_string(), KeyCode::KEY_X);
        map.insert("y".to_string(), KeyCode::KEY_Y);
        map.insert("z".to_string(), KeyCode::KEY_Z);

        // Numbers
        map.insert("0".to_string(), KeyCode::KEY_0);
        map.insert("1".to_string(), KeyCode::KEY_1);
        map.insert("2".to_string(), KeyCode::KEY_2);
        map.insert("3".to_string(), KeyCode::KEY_3);
        map.insert("4".to_string(), KeyCode::KEY_4);
        map.insert("5".to_string(), KeyCode::KEY_5);
        map.insert("6".to_string(), KeyCode::KEY_6);
        map.insert("7".to_string(), KeyCode::KEY_7);
        map.insert("8".to_string(), KeyCode::KEY_8);
        map.insert("9".to_string(), KeyCode::KEY_9);

        // Common keys
        map.insert("space".to_string(), KeyCode::KEY_SPACE);
        map.insert("enter".to_string(), KeyCode::KEY_ENTER);
        map.insert("return".to_string(), KeyCode::KEY_ENTER);
        map.insert("tab".to_string(), KeyCode::KEY_TAB);
        map.insert("escape".to_string(), KeyCode::KEY_ESC);
        map.insert("esc".to_string(), KeyCode::KEY_ESC);
        map.insert("menu".to_string(), KeyCode::KEY_MENU);
        map.insert("backspace".to_string(), KeyCode::KEY_BACKSPACE);
        map.insert("back".to_string(), KeyCode::KEY_BACKSPACE);
        map.insert("insert".to_string(), KeyCode::KEY_INSERT);
        map.insert("delete".to_string(), KeyCode::KEY_DELETE);

        // Arrow keys
        map.insert("up".to_string(), KeyCode::KEY_UP);
        map.insert("down".to_string(), KeyCode::KEY_DOWN);
        map.insert("left".to_string(), KeyCode::KEY_LEFT);
        map.insert("right".to_string(), KeyCode::KEY_RIGHT);
        map.insert("pageup".to_string(), KeyCode::KEY_PAGEUP);
        map.insert("pagedown".to_string(), KeyCode::KEY_PAGEDOWN);
        map.insert("scrollup".to_string(), KeyCode::KEY_SCROLLUP);
        map.insert("scrolldown".to_string(), KeyCode::KEY_SCROLLDOWN);
        map.insert("home".to_string(), KeyCode::KEY_HOME);
        map.insert("end".to_string(), KeyCode::KEY_END);

        // Modifiers
        map.insert("shift".to_string(), KeyCode::KEY_LEFTSHIFT);
        map.insert("ctrl".to_string(), KeyCode::KEY_LEFTCTRL);
        map.insert("control".to_string(), KeyCode::KEY_LEFTCTRL);
        map.insert("alt".to_string(), KeyCode::KEY_LEFTALT);
        map.insert("lalt".to_string(), KeyCode::KEY_LEFTALT);
        map.insert("ralt".to_string(), KeyCode::KEY_RIGHTALT);
        map.insert("super".to_string(), KeyCode::KEY_LEFTMETA);
        map.insert("lsuper".to_string(), KeyCode::KEY_LEFTMETA);
        map.insert("rsuper".to_string(), KeyCode::KEY_RIGHTMETA);
        map.insert("meta".to_string(), KeyCode::KEY_LEFTMETA);
        map.insert("lmeta".to_string(), KeyCode::KEY_LEFTMETA);
        map.insert("rmeta".to_string(), KeyCode::KEY_RIGHTMETA);
        map.insert("win".to_string(), KeyCode::KEY_LEFTMETA);
        map.insert("lwin".to_string(), KeyCode::KEY_LEFTMETA);
        map.insert("rwin".to_string(), KeyCode::KEY_RIGHTMETA);
        map.insert("cmd".to_string(), KeyCode::KEY_LEFTCTRL);

        // Media keys
        map.insert("volumeup".to_string(), KeyCode::KEY_VOLUMEUP);
        map.insert("volumedown".to_string(), KeyCode::KEY_VOLUMEDOWN);
        map.insert("volumemute".to_string(), KeyCode::KEY_MUTE);
        map.insert("volume_up".to_string(), KeyCode::KEY_VOLUMEUP);
        map.insert("volume_down".to_string(), KeyCode::KEY_VOLUMEDOWN);
        map.insert("volume_mute".to_string(), KeyCode::KEY_MUTE);
        map.insert("mediaplaypause".to_string(), KeyCode::KEY_PLAYPAUSE);
        map.insert("mediastop".to_string(), KeyCode::KEY_STOP);
        map.insert("medianext".to_string(), KeyCode::KEY_NEXT);
        map.insert("mediaprevious".to_string(), KeyCode::KEY_PREVIOUS);

        map.insert("oem_plus".to_string(), KeyCode::KEY_KPPLUS);
        map.insert("oem_minus".to_string(), KeyCode::KEY_KPMINUS);
        map.insert("oem_0".to_string(), KeyCode::KEY_KP0);
        map.insert("oem_1".to_string(), KeyCode::KEY_KP1);
        map.insert("oem_2".to_string(), KeyCode::KEY_KP2);
        map.insert("oem_3".to_string(), KeyCode::KEY_KP3);
        map.insert("oem_4".to_string(), KeyCode::KEY_KP4);
        map.insert("oem_5".to_string(), KeyCode::KEY_KP5);
        map.insert("oem_6".to_string(), KeyCode::KEY_KP6);
        map.insert("oem_7".to_string(), KeyCode::KEY_KP7);
        map.insert("oem_8".to_string(), KeyCode::KEY_KP8);
        map.insert("oem_9".to_string(), KeyCode::KEY_KP9);

        // Function keys
        for n in 1..=12 {
            let key_code = KeyCode::KEY_F1.code() + (n - 1);
            let key = KeyCode::new(key_code);
            map.insert(format!("f{}", n), key);
        }

        map
    }

    fn get_key(&self, key: &str) -> Result<KeyCode, InputError> {
        self.key_map
            .get(&key.to_lowercase())
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
        self.key_map.contains_key(&key.to_lowercase())
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
