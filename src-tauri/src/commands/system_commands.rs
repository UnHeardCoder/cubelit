use crate::error::AppError;
use crate::ports;
#[cfg(target_os = "windows")]
use serde::Serialize;

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize)]
pub struct WslStatus {
    pub wsl_installed: bool,
    pub wsl2_enabled: bool,
    pub reboot_required: bool,
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn check_wsl_status() -> WslStatus {
    let wsl_exe = r"C:\Windows\System32\wsl.exe";

    // Run wsl --status to detect install state
    let status_output = std::process::Command::new(wsl_exe)
        .args(["--status"])
        .output();

    let (mut wsl_installed, mut wsl2_enabled) = (false, false);

    if let Ok(out) = status_output {
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
        .to_lowercase();

        wsl_installed = out.status.success() || text.contains("default distribution");
        wsl2_enabled = wsl_installed && text.contains("default version: 2");
    }

    // Locale fallback: check wsl -l -v for a line ending in " 2"
    if wsl_installed && !wsl2_enabled {
        if let Ok(out) = std::process::Command::new(wsl_exe).args(["-l", "-v"]).output() {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            wsl2_enabled = text.lines().skip(1).any(|l| l.trim_end().ends_with('2'));
        }
    }

    // Check for pending reboot via registry key presence
    let reboot_required = std::process::Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending",
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    WslStatus { wsl_installed, wsl2_enabled, reboot_required }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn enable_wsl2() -> Result<(), AppError> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let verb = to_wide("runas");
    let file = to_wide("powershell.exe");
    let params = to_wide(
        "-NoProfile -NonInteractive -ExecutionPolicy Bypass -Command \
        \"Enable-WindowsOptionalFeature -Online -FeatureName \
        Microsoft-Windows-Subsystem-Linux -NoRestart; \
        Enable-WindowsOptionalFeature -Online -FeatureName \
        VirtualMachinePlatform -NoRestart\"",
    );

    let result = unsafe {
        ShellExecuteW(
            0,                  // no parent window
            verb.as_ptr(),
            file.as_ptr(),
            params.as_ptr(),
            std::ptr::null(),
            1, // SW_SHOWNORMAL
        )
    };

    if result as usize <= 32 {
        Err(AppError::Validation(format!(
            "Failed to launch elevated PowerShell (code {})",
            result as usize
        )))
    } else {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn set_wsl_default_version() -> Result<(), AppError> {
    let output = std::process::Command::new(r"C:\Windows\System32\wsl.exe")
        .args(["--set-default-version", "2"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::Validation(format!(
            "wsl --set-default-version 2 failed: {err}"
        )))
    }
}

#[tauri::command]
pub fn check_port(port: u16) -> bool {
    ports::is_port_available(port)
}

#[tauri::command]
pub fn suggest_port(default_port: u16) -> u16 {
    ports::suggest_port(default_port)
}

#[tauri::command]
pub async fn get_public_ip() -> Result<String, AppError> {
    let ip = reqwest::Client::new()
        .get("https://api.ipify.org")
        .send()
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?
        .text()
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(ip.trim().to_string())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), AppError> {
    std::fs::create_dir_all(&path)?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer.exe").arg(&path).spawn()?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&path).spawn()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn()?;

    Ok(())
}
