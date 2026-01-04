use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("failed to initialize input backend: {0}")]
    InitError(String),
    #[error("failed to send input event: {0}")]
    SendError(String),
}

pub trait InputBackend: Send + Sync {
    fn is_key(&self, key: &str) -> bool;

    fn key_press(&self, key: &str) -> Result<(), InputError>;
    fn key_release(&self, key: &str) -> Result<(), InputError>;
    fn key_click(&self, key: &str) -> Result<(), InputError>;

    fn mouse_move(&self, dx: i32, dy: i32) -> Result<(), InputError>;
    fn mouse_button_press(&self, button: MouseButton) -> Result<(), InputError>;
    fn mouse_button_release(&self, button: MouseButton) -> Result<(), InputError>;
    fn mouse_button_click(&self, button: MouseButton) -> Result<(), InputError>;
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl FromStr for MouseButton {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "left" => Ok(MouseButton::Left),
            "right" => Ok(MouseButton::Right),
            "middle" => Ok(MouseButton::Middle),
            _ => Err(InputError::SendError(format!("unknown mouse button: {s}",))),
        }
    }
}

#[cfg(all(target_os = "linux", feature = "input-uinput"))]
mod uinput;

#[cfg(all(target_os = "linux", feature = "input-uinput"))]
pub use uinput::UInputBackend;
