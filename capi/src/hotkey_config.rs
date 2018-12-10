//! The configuration to use for a Hotkey System. It describes with keys to use
//! as hotkeys for the different actions.

use super::{output_vec, str, Json};
use livesplit_core::HotkeyConfig;
use serde_json;
use setting_value::OwnedSettingValue;
use std::io::Cursor;

/// type
pub type OwnedHotkeyConfig = Box<HotkeyConfig>;
/// type
pub type NullableOwnedHotkeyConfig = Option<OwnedHotkeyConfig>;

/// drop
#[no_mangle]
pub extern "C" fn HotkeyConfig_drop(this: OwnedHotkeyConfig) {
    drop(this);
}

/// Encodes generic description of the settings available for the hotkey
/// configuration and their current values as JSON.
#[no_mangle]
pub extern "C" fn HotkeyConfig_settings_description_as_json(this: &HotkeyConfig) -> Json {
    output_vec(|o| {
        serde_json::to_writer(o, &this.settings_description()).unwrap();
    })
}

/// Sets a setting's value by its index to the given value.
///
/// <FALSE> is returned if a hotkey is already in use by a different action.
///
/// This panics if the type of the value to be set is not compatible with the
/// type of the setting's value. A panic can also occur if the index of the
/// setting provided is out of bounds.
#[no_mangle]
pub extern "C" fn HotkeyConfig_set_value(
    this: &mut HotkeyConfig,
    index: usize,
    value: OwnedSettingValue,
) -> bool {
    this.set_value(index, *value).is_ok()
}

/// Encodes the hotkey configuration as JSON.
#[no_mangle]
pub extern "C" fn HotkeyConfig_as_json(this: &HotkeyConfig) -> Json {
    output_vec(|o| {
        this.write_json(o).unwrap();
    })
}

/// Parses a hotkey configuration from the given JSON description. <NULL> is
/// returned if it couldn't be parsed.
#[no_mangle]
pub unsafe extern "C" fn HotkeyConfig_parse_json(settings: Json) -> NullableOwnedHotkeyConfig {
    let settings = Cursor::new(str(settings).as_bytes());
    HotkeyConfig::from_json(settings).ok().map(Box::new)
}