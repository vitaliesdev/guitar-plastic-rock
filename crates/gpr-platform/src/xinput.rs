//! XInput: lo que Windows ve en cada mando Xbox (para comprobar lo que recibe el mando virtual).

pub use windows::Win32::UI::Input::XboxController::XINPUT_GAMEPAD;
use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE};

pub fn state(slot: u32) -> Option<XINPUT_GAMEPAD> {
    let mut st = XINPUT_STATE::default();
    let r = unsafe { XInputGetState(slot, &mut st) };
    (r == 0).then_some(st.Gamepad)
}
