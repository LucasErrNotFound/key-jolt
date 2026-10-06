#[cfg(target_os = "windows")]
mod windows;

pub(crate) const STARTUP_ARGUMENT: &str = "--startup";

pub(crate) fn is_supported() -> bool {
    cfg!(target_os = "windows")
}

pub(crate) fn launched_at_login() -> bool {
    is_supported()
        && std::env::args_os()
            .skip(1)
            .any(|argument| argument == STARTUP_ARGUMENT)
}

pub(crate) fn set_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        windows::set_enabled(enabled)
    }
    #[cfg(not(target_os = "windows"))]
    {
        if enabled {
            Err("Run at startup is available on Windows.".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(any(target_os = "windows", test))]
fn command_line(executable: &std::path::Path) -> Result<Vec<u16>, String> {
    #[cfg(target_os = "windows")]
    let path = {
        use std::os::windows::ffi::OsStrExt;
        executable.as_os_str().encode_wide().collect::<Vec<_>>()
    };
    #[cfg(not(target_os = "windows"))]
    let path = executable
        .to_string_lossy()
        .encode_utf16()
        .collect::<Vec<_>>();
    if path.is_empty()
        || path
            .iter()
            .any(|unit| *unit == 0 || *unit == u16::from(b'"'))
    {
        return Err("The executable path cannot be registered for startup.".to_string());
    }
    let mut command = vec![u16::from(b'"')];
    command.extend(path);
    command.extend(format!("\" {STARTUP_ARGUMENT}").encode_utf16());
    if command.len() > 260 {
        return Err("The executable path is too long for Windows startup. Move KeyJolt to a shorter path and try again.".to_string());
    }
    command.push(0);
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn startup_command_quotes_spaces_and_preserves_unicode() {
        let path = Path::new(r"C:\My Apps\键盘\key-jolt.exe");
        let command = command_line(path).unwrap();
        assert_eq!(command.last(), Some(&0));
        assert_eq!(
            String::from_utf16(&command[..command.len() - 1]).unwrap(),
            "\"C:\\My Apps\\键盘\\key-jolt.exe\" --startup"
        );
    }

    #[test]
    fn startup_command_rejects_invalid_and_oversized_paths() {
        for value in [
            String::new(),
            "bad\0path".to_string(),
            "bad\"path".to_string(),
            "x".repeat(260),
        ] {
            assert!(command_line(Path::new(&value)).is_err());
        }
    }
}
