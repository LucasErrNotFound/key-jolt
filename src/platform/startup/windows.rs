use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "KeyJolt";

struct RegistryKey(HKEY);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        // SAFETY: This handle is owned by this wrapper and is closed exactly once after its last use.
        unsafe { RegCloseKey(self.0) };
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn check(result: u32) -> Result<(), String> {
    if result == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!(
            "Could not update Windows startup: {}",
            std::io::Error::from_raw_os_error(result as i32)
        ))
    }
}

pub(super) fn set_enabled(enabled: bool) -> Result<(), String> {
    let command = if enabled {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        Some(super::command_line(&executable)?)
    } else {
        None
    };
    let path = wide(RUN_KEY);
    let name = wide(VALUE_NAME);
    let mut handle = null_mut();
    // SAFETY: Both paths are null-terminated UTF-16 buffers, the output handle is writable,
    // and optional pointers are null. The returned handle is checked before use and owned below.
    let result = unsafe {
        if enabled {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                path.as_ptr(),
                0,
                null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                null(),
                &mut handle,
                null_mut(),
            )
        } else {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                path.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut handle,
            )
        }
    };
    if !enabled && matches!(result, ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND) {
        return Ok(());
    }
    check(result)?;
    let key = RegistryKey(handle);
    let result = if let Some(command) = command {
        let bytes = u32::try_from(std::mem::size_of_val(command.as_slice()))
            .map_err(|_| "The startup command is too large.".to_string())?;
        // SAFETY: The live key permits value writes; name and command remain valid for this call.
        // The byte count includes the UTF-16 terminator and matches the command allocation.
        unsafe {
            RegSetValueExW(
                key.0,
                name.as_ptr(),
                0,
                REG_SZ,
                command.as_ptr().cast(),
                bytes,
            )
        }
    } else {
        // SAFETY: The live key permits deletion and the value name is a valid terminated UTF-16 buffer.
        unsafe { RegDeleteValueW(key.0, name.as_ptr()) }
    };
    if !enabled && result == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }
    check(result)
}
