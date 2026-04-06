use crate::error::AppError;
use crate::ports;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize)]
/// Basic WSL readiness status returned to the frontend's legacy Windows check.
pub struct WslStatus {
    pub wsl_installed: bool,
    pub wsl2_enabled: bool,
    pub reboot_required: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
/// High-level Docker availability states used by onboarding diagnostics.
pub enum DockerState {
    Ready,
    NotInstalled,
    NotRunning,
    PermissionDenied,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[serde(rename_all = "snake_case")]
/// High-level WSL readiness states used by Windows onboarding diagnostics.
pub enum WslState {
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    NotApplicable,
    Ok,
    NeedsInstall,
    NeedsDefaultV2,
    RebootRequired,
    CheckFailed,
}

#[derive(Debug, Clone, Serialize)]
/// Docker diagnostic details surfaced to the frontend onboarding UI.
pub struct DockerDiagnostic {
    pub state: DockerState,
    pub installed: Option<bool>,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
/// WSL diagnostic details surfaced to the frontend onboarding UI.
pub struct WslDiagnostic {
    pub state: WslState,
    pub wsl_installed: Option<bool>,
    pub wsl2_enabled: Option<bool>,
    pub reboot_required: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
/// Combined onboarding status consumed by the desktop app's setup gate.
pub struct OnboardingStatus {
    pub platform: &'static str,
    pub docker: DockerDiagnostic,
    pub wsl: WslDiagnostic,
}

fn platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        "linux"
    }
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn reboot_pending_windows() -> bool {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("reg")
            .args([
                "query",
                r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending",
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
fn check_wsl_diagnostic() -> WslDiagnostic {
    let reboot_required = reboot_pending_windows();
    let wsl_exe = r"C:\Windows\System32\wsl.exe";

    let status_output = std::process::Command::new(wsl_exe)
        .args(["--status"])
        .output();

    let out = match status_output {
        Ok(out) => out,
        Err(e) => {
            return WslDiagnostic {
                state: WslState::CheckFailed,
                wsl_installed: None,
                wsl2_enabled: None,
                reboot_required,
                error: Some(e.to_string()),
            }
        }
    };

    let status_text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let status_text_lower = status_text.to_lowercase();

    let mut wsl_installed = out.status.success()
        || status_text_lower.contains("default distribution")
        || status_text_lower.contains("default version")
        || status_text_lower.contains("windows subsystem for linux");
    let mut wsl2_enabled = wsl_installed && status_text_lower.contains("default version: 2");

    if let Ok(out) = std::process::Command::new(wsl_exe).args(["-l", "-v"]).output() {
        let list_text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let list_text_lower = list_text.to_lowercase();

        wsl_installed = wsl_installed || out.status.success() || list_text_lower.contains("windows subsystem for linux");
        wsl2_enabled = wsl2_enabled
            || list_text
                .lines()
                .skip(1)
                .any(|line| line.trim_end().ends_with('2'));
    }

    let error = if out.status.success() || status_text.trim().is_empty() {
        None
    } else {
        Some(status_text.trim().to_string())
    };

    let state = if reboot_required {
        WslState::RebootRequired
    } else if !wsl_installed {
        WslState::NeedsInstall
    } else if wsl2_enabled {
        WslState::Ok
    } else {
        WslState::NeedsDefaultV2
    };

    WslDiagnostic {
        state,
        wsl_installed: Some(wsl_installed),
        wsl2_enabled: Some(wsl2_enabled),
        reboot_required,
        error,
    }
}

#[cfg(not(target_os = "windows"))]
fn check_wsl_diagnostic() -> WslDiagnostic {
    WslDiagnostic {
        state: WslState::NotApplicable,
        wsl_installed: None,
        wsl2_enabled: None,
        reboot_required: false,
        error: None,
    }
}

#[cfg(target_os = "windows")]
fn docker_desktop_installed_windows() -> bool {
    let common_paths = [
        r"C:\Program Files\Docker\Docker\Docker Desktop.exe",
        r"C:\Program Files\Docker\Docker\DockerCli.exe",
        r"C:\Program Files\Docker\Docker\resources\bin\docker.exe",
    ];

    if common_paths.iter().any(|path| std::path::Path::new(path).exists()) {
        return true;
    }

    let registry_roots = [
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    registry_roots.iter().any(|root| {
        std::process::Command::new("reg")
            .args(["query", root, "/s", "/f", "Docker Desktop"])
            .output()
            .map(|out| {
                out.status.success()
                    && String::from_utf8_lossy(&out.stdout)
                        .to_lowercase()
                        .contains("docker desktop")
            })
            .unwrap_or(false)
    })
}

fn classify_docker_state(error: &str, installed: Option<bool>) -> DockerState {
    let error_lower = error.to_lowercase();

    if error_lower.contains("permission denied")
        || error_lower.contains("access is denied")
        || error_lower.contains("access denied")
    {
        return DockerState::PermissionDenied;
    }

    if matches!(installed, Some(false)) {
        return DockerState::NotInstalled;
    }

    if error_lower.contains("cannot connect")
        || error_lower.contains("connection refused")
        || error_lower.contains("error during connect")
        || error_lower.contains("deadline has elapsed")
        || error_lower.contains("docker daemon")
        || error_lower.contains("open //./pipe/docker_engine")
        || error_lower.contains("system cannot find the file specified")
    {
        return DockerState::NotRunning;
    }

    if error_lower.contains("no such file") || error_lower.contains("not found") {
        return if matches!(installed, Some(true)) {
            DockerState::NotRunning
        } else {
            DockerState::NotInstalled
        };
    }

    DockerState::Unknown
}

async fn check_docker_diagnostic(
    docker: &bollard::Docker,
) -> DockerDiagnostic {
    match docker.ping().await {
        Ok(_) => match docker.version().await {
            Ok(version) => DockerDiagnostic {
                state: DockerState::Ready,
                installed: Some(true),
                version: version.version,
                error: None,
            },
            Err(e) => {
                tracing::error!("Docker version failed after successful ping: {e}");
                DockerDiagnostic {
                    state: DockerState::Unknown,
                    installed: Some(true),
                    version: None,
                    error: Some(e.to_string()),
                }
            }
        },
        Err(e) => {
            let error = e.to_string();
            tracing::error!("Docker ping failed: {error}");
            #[cfg(target_os = "windows")]
            let installed = Some(docker_desktop_installed_windows());
            #[cfg(not(target_os = "windows"))]
            let installed = None;

            DockerDiagnostic {
                state: classify_docker_state(&error, installed),
                installed,
                version: None,
                error: Some(error),
            }
        }
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
/// Returns the legacy Windows-only WSL status used by older onboarding flows.
pub fn check_wsl_status() -> WslStatus {
    let diagnostic = check_wsl_diagnostic();
    WslStatus {
        wsl_installed: diagnostic.wsl_installed.unwrap_or(false),
        wsl2_enabled: diagnostic.wsl2_enabled.unwrap_or(false),
        reboot_required: diagnostic.reboot_required,
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
/// Launches an elevated PowerShell command to enable the Windows WSL2 features.
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
            std::ptr::null_mut(), // no parent window
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
/// Sets WSL version 2 as the default for new Linux distributions on Windows.
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
/// Returns a full onboarding snapshot combining Docker and platform diagnostics.
pub async fn get_onboarding_status(
    state: State<'_, AppState>,
) -> Result<OnboardingStatus, AppError> {
    Ok(OnboardingStatus {
        platform: platform_name(),
        docker: check_docker_diagnostic(&state.docker).await,
        wsl: check_wsl_diagnostic(),
    })
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
