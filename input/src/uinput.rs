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
        for c in 'a'..='z' {
            let key_code = KeyCode::KEY_A.code() + (c as u16 - 'a' as u16);
            let key = KeyCode::new(key_code);
            map.insert(c.to_string(), key);
            map.insert(c.to_uppercase().to_string(), key);
        }

        // Numbers
        for n in 0..=9 {
            let key_code = KeyCode::KEY_1.code() + n - 1;
            let key = KeyCode::new(if n == 0 {
                KeyCode::KEY_0.code()
            } else {
                key_code
            });
            map.insert(n.to_string(), key);
        }

        // Common keys
        map.insert("space".to_string(), KeyCode::KEY_SPACE);
        map.insert("enter".to_string(), KeyCode::KEY_ENTER);
        map.insert("return".to_string(), KeyCode::KEY_ENTER);
        map.insert("backspace".to_string(), KeyCode::KEY_BACKSPACE);
        map.insert("tab".to_string(), KeyCode::KEY_TAB);
        map.insert("escape".to_string(), KeyCode::KEY_ESC);
        map.insert("esc".to_string(), KeyCode::KEY_ESC);

        // Arrow keys
        map.insert("up".to_string(), KeyCode::KEY_UP);
        map.insert("down".to_string(), KeyCode::KEY_DOWN);
        map.insert("left".to_string(), KeyCode::KEY_LEFT);
        map.insert("right".to_string(), KeyCode::KEY_RIGHT);

        // Modifiers
        map.insert("shift".to_string(), KeyCode::KEY_LEFTSHIFT);
        map.insert("ctrl".to_string(), KeyCode::KEY_LEFTCTRL);
        map.insert("control".to_string(), KeyCode::KEY_LEFTCTRL);
        map.insert("alt".to_string(), KeyCode::KEY_LEFTALT);
        map.insert("super".to_string(), KeyCode::KEY_LEFTMETA);
        map.insert("meta".to_string(), KeyCode::KEY_LEFTMETA);

        // Media keys
        map.insert("volumeup".to_string(), KeyCode::KEY_VOLUMEUP);
        map.insert("volumedown".to_string(), KeyCode::KEY_VOLUMEDOWN);
        map.insert("volumemute".to_string(), KeyCode::KEY_MUTE);
        map.insert("mediaplaypause".to_string(), KeyCode::KEY_PLAYPAUSE);
        map.insert("mediastop".to_string(), KeyCode::KEY_STOP);
        map.insert("medianext".to_string(), KeyCode::KEY_NEXT);
        map.insert("mediaprevious".to_string(), KeyCode::KEY_PREVIOUS);

        map.insert("scrollup".to_string(), KeyCode::KEY_SCROLLUP);
        map.insert("scrolldown".to_string(), KeyCode::KEY_SCROLLDOWN);

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
            .ok_or_else(|| InputError::SendError(format!("Unknown key: {}", key)))
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
