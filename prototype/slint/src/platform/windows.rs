use std::ffi::OsString;
use std::iter;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use crate::models::TrackerState;

/// Get the name of the currently active user application on Windows.
///
/// Returns None if there is no active user application (e.g., system lock screen,
/// or only system applications are running).
pub fn get_active_user_app() -> Option<String> {
    // Safety: These functions are safe to call as they are Windows API functions
    // that do not require unsafe Rust when called via windows-sys.
    let hwnd = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
    if hwnd == 0 {
        return None;
    }

    let mut pid: u32 = 0;
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if pid == 0 {
        return None;
    }

    let path = get_process_image_path(pid)?;
    let app_name = extract_app_name(&path)?;

    if is_system_app(&app_name, &path) {
        return None;
    }

    Some(app_name)
}

/// Get the executable image path for a given process ID.
fn get_process_image_path(pid: u32) -> Option<PathBuf> {
    let handle = unsafe {
        windows_sys::Win32::System::Threading::OpenProcess(
            windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            pid,
        )
    };
    if handle == 0 {
        return None;
    }

    let mut size: u32 = 1024;
    let mut buffer = vec![0u16; size as usize];

    let ok = unsafe {
        windows_sys::Win32::System::Threading::QueryFullProcessImageNameW(
            handle,
            windows_sys::Win32::System::Threading::PROCESS_NAME_WIN32,
            buffer.as_mut_ptr(),
            &mut size,
        )
    };

    // Safety: We obtained the handle from OpenProcess and we are closing it below.
    unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };

    if ok == 0 || size == 0 {
        return None;
    }

    // The buffer contains a UTF-16 string.
    let mut v = buffer;
    v.truncate(size as usize);
    let path = OsString::from_wide(&v);
    Some(PathBuf::from(path))
}

/// Extract the application name (without extension) from a full path.
fn extract_app_name(path: &PathBuf) -> Option<String> {
    let name = path
        .file_name()?
        .to_str()?
        .trim_end_matches(".exe")
        .to_string();

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Determine if the given application is a system application that should be ignored.
///
/// This logic is taken directly from the original Tauri implementation.
fn is_system_app(app_name: &str, path: &PathBuf) -> bool {
    let app = app_name.to_ascii_lowercase();
    let p = path
        .to_string_lossy()
        .to_ascii_lowercase();

    let blocked = [
        "taskmgr",
        "dwm",
        "idle",
        "system",
        "svchost",
        "explorer",
        "sihost",
        "searchhost",
        "searchapp",
        "startmenuexperiencehost",
        "textinputhost",
        "runtimebroker",
        "ctfmon",
        "fontdrvhost",
        "applicationframehost",
        "lockapp",
        "shellexperiencehost",
    ];

    if blocked.contains(&app.as_str()) {
        return true;
    }

    p.starts_with("c:\\windows\\") || p.contains("\\windowsapps\\")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_extract_app_name() {
        assert_eq!(extract_app_name(&PathBuf::from("C:\\Windows\\notepad.exe")), Some("notepad".to_string()));
        assert_eq!(extract_app_name(&PathBuf::from("C:\\Program Files\\App\\myapp.exe")), Some("myapp".to_string()));
        assert_eq!(extract_app_name(&PathBuf::from("C:\\App")), Some("App".to_string())); // no extension but still valid app name
        assert_eq!(extract_app_name(&PathBuf::from("")), None);
    }

    #[test]
    fn test_is_system_app() {
        // Blocked names
        assert!(is_system_app("taskmgr", &PathBuf::from("C:\\temp\\taskmgr.exe")));
        assert!(is_system_app("dwm", &PathBuf::from("C:\\temp\\dwm.exe")));
        assert!(is_system_app("explorer", &PathBuf::from("C:\\temp\\explorer.exe")));
        assert!(is_system_app("svchost", &PathBuf::from("C:\\temp\\svchost.exe")));
        assert!(is_system_app("sihost", &PathBuf::from("C:\\temp\\sihost.exe")));
        assert!(is_system_app("searchhost", &PathBuf::from("C:\\temp\\searchhost.exe")));
        assert!(is_system_app("searchapp", &PathBuf::from("C:\\temp\\searchapp.exe")));
        assert!(is_system_app("startmenuexperiencehost", &PathBuf::from("C:\\temp\\startmenuexperiencehost.exe")));
        assert!(is_system_app("textinputhost", &PathBuf::from("C:\\temp\\textinputhost.exe")));
        assert!(is_system_app("runtimebroker", &PathBuf::from("C:\\temp\\runtimebroker.exe")));
        assert!(is_system_app("ctfmon", &PathBuf::from("C:\\temp\\ctfmon.exe")));
        assert!(is_system_app("fontdrvhost", &PathBuf::from("C:\\temp\\fontdrvhost.exe")));
        assert!(is_system_app("applicationframehost", &PathBuf::from("C:\\temp\\applicationframehost.exe")));
        assert!(is_system_app("lockapp", &PathBuf::from("C:\\temp\\lockapp.exe")));
        assert!(is_system_app("shellexperiencehost", &PathBuf::from("C:\\temp\\shellexperiencehost.exe")));

        // Windows directory
        assert!(is_system_app("someapp", &PathBuf::from("C:\\Windows\\someapp.exe")));
        assert!(is_system_app("someapp", &PathBuf::from("C:\\Windows\\System32\\someapp.exe")));

        // WindowsApps directory
        assert!(is_system_app("someapp", &PathBuf::from("C:\\Program Files\\WindowsApps\\someapp.exe")));

        // System app (in Windows directory)
        assert!(is_system_app("notepad", &PathBuf::from("C:\\Windows\\notepad.exe")));
        // Note: notepad.exe in C:\Windows\System32 is considered system because the path starts with "c:\\windows\\"

        // Let's test with a path that is not in Windows and not blocked by name.
        assert!(!is_system_app("myapp", &PathBuf::from("C:\\Program Files\\MyApp\\myapp.exe")));
        assert!(!is_system_app("myapp", &PathBuf::from("D:\\Apps\\myapp.exe")));
    }
}