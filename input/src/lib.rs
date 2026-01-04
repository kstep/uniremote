use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Failed to initialize input backend: {0}")]
    InitError(String),
    #[error("Failed to send input event: {0}")]
    SendError(String),
}

pub trait InputBackend: Send + Sync {
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

#[cfg(all(target_os = "linux", feature = "input-uinput"))]
mod uinput;

#[cfg(all(target_os = "linux", feature = "input-uinput"))]
pub use uinput::UInputBackend;
