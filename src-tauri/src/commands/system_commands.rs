use cubelit_core::ports;
use serde::Serialize;
use tauri::State;

use crate::error::CoreError;
use crate::state::AppState;

#[cfg(any(target_os = "windows", test))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    pub features_enabled: Option<bool>,
    pub default_version_2: Option<bool>,
    pub distro_version_2_present: Option<bool>,
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

#[cfg(any(target_os = "windows", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionalFeatureState {
    Enabled,
    Disabled,
    EnablePending,
    DisablePending,
    Unknown,
}

#[cfg(any(target_os = "windows", test))]
struct WslProbe {
    subsystem_feature: OptionalFeatureState,
    vm_platform_feature: OptionalFeatureState,
    default_version_2: Option<bool>,
    distro_version_2_present: bool,
    reboot_required: bool,
    status_error: Option<String>,
    list_error: Option<String>,
}

#[cfg(any(target_os = "windows", test))]
fn parse_dism_feature_state(text: &str) -> OptionalFeatureState {
    for line in text.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };

        if name.trim().eq_ignore_ascii_case("state") {
            return match value.trim().to_ascii_lowercase().as_str() {
                "enabled" => OptionalFeatureState::Enabled,
                "disabled" => OptionalFeatureState::Disabled,
                "enable pending" => OptionalFeatureState::EnablePending,
                "disable pending" => OptionalFeatureState::DisablePending,
                _ => OptionalFeatureState::Unknown,
            };
        }
    }

    OptionalFeatureState::Unknown
}

#[cfg(any(target_os = "windows", test))]
fn parse_wsl_default_version_2(text: &str) -> Option<bool> {
    for line in text.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };

        if name.trim().eq_ignore_ascii_case("default version") {
            return match value.trim() {
                "2" => Some(true),
                "1" => Some(false),
                _ => None,
            };
        }
    }

    None
}

#[cfg(any(target_os = "windows", test))]
fn parse_wsl_list_has_v2(text: &str) -> bool {
    text.lines()
        .skip(1)
        .any(|line| line.split_whitespace().last() == Some("2"))
}

#[cfg(any(target_os = "windows", test))]
fn classify_wsl_probe(probe: WslProbe) -> WslDiagnostic {
    use OptionalFeatureState::{EnablePending, Enabled};

    let features_enabled =
        probe.subsystem_feature == Enabled && probe.vm_platform_feature == Enabled;

    let state =
        if probe.subsystem_feature == EnablePending || probe.vm_platform_feature == EnablePending {
            WslState::RebootRequired
        } else if probe.subsystem_feature != Enabled && probe.vm_platform_feature != Enabled {
            WslState::NeedsInstall
        } else if probe.reboot_required && !features_enabled {
            WslState::RebootRequired
        } else if probe.default_version_2 == Some(false) {
            WslState::NeedsDefaultV2
        } else if features_enabled && probe.default_version_2 != Some(false) {
            WslState::Ok
        } else {
            WslState::NeedsInstall
        };

    let error = match (probe.status_error, probe.list_error) {
        (Some(status_error), Some(list_error)) => Some(format!(
            "wsl --status failed: {status_error}\nwsl -l -v failed: {list_error}"
        )),
        (Some(status_error), None) => Some(format!("wsl --status failed: {status_error}")),
        (None, Some(list_error)) => Some(format!("wsl -l -v failed: {list_error}")),
        (None, None) => None,
    };

    let wsl2_enabled = matches!(state, WslState::Ok);
    let reboot_required = probe.reboot_required || matches!(state, WslState::RebootRequired);

    WslDiagnostic {
        state,
        wsl_installed: Some(features_enabled),
        wsl2_enabled: Some(wsl2_enabled),
        reboot_required,
        error,
        features_enabled: Some(features_enabled),
        default_version_2: probe.default_version_2,
        distro_version_2_present: Some(probe.distro_version_2_present),
    }
}

#[cfg(target_os = "windows")]
struct CommandProbe {
    text: String,
    success: bool,
    error: Option<String>,
}

#[cfg(target_os = "windows")]
fn command_probe(command: &str, args: &[&str]) -> CommandProbe {
    match std::process::Command::new(command).args(args).output() {
        Ok(out) => {
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            let error = if out.status.success() {
                None
            } else {
                Some(text.trim().to_string())
            };

            CommandProbe {
                text,
                success: out.status.success(),
                error,
            }
        }
        Err(e) => CommandProbe {
            text: String::new(),
            success: false,
            error: Some(e.to_string()),
        },
    }
}

#[cfg(target_os = "windows")]
fn dism_feature_probe(feature_name: &str) -> (OptionalFeatureState, Option<String>) {
    let out = command_probe(
        "dism.exe",
        &[
            "/online",
            "/Get-FeatureInfo",
            &format!("/FeatureName:{feature_name}"),
        ],
    );
    let state = parse_dism_feature_state(&out.text);

    let error = if out.success {
        None
    } else {
        Some(format!(
            "dism.exe /online /Get-FeatureInfo /FeatureName:{feature_name} failed: {}",
            out.error.unwrap_or_else(|| out.text.trim().to_string())
        ))
    };

    (state, error)
}

/// A failed DISM probe means the feature state is UNKNOWN, not disabled.
/// Classifying an unknown as NeedsInstall would send the user through feature
/// enablement they may not need — any probe failure becomes CheckFailed with
/// the underlying error(s) surfaced.
#[cfg(any(target_os = "windows", test))]
fn dism_failure_diagnostic(
    subsystem_error: Option<String>,
    vm_platform_error: Option<String>,
    reboot_required: bool,
) -> Option<WslDiagnostic> {
    if subsystem_error.is_none() && vm_platform_error.is_none() {
        return None;
    }
    Some(WslDiagnostic {
        state: WslState::CheckFailed,
        wsl_installed: None,
        wsl2_enabled: None,
        reboot_required,
        error: Some(
            [subsystem_error, vm_platform_error]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        features_enabled: None,
        default_version_2: None,
        distro_version_2_present: None,
    })
}

#[cfg(target_os = "windows")]
fn check_wsl_diagnostic() -> WslDiagnostic {
    let reboot_required = reboot_pending_windows();
    let wsl_exe = r"C:\Windows\System32\wsl.exe";

    let (subsystem_feature, subsystem_error) =
        dism_feature_probe("Microsoft-Windows-Subsystem-Linux");
    let (vm_platform_feature, vm_platform_error) = dism_feature_probe("VirtualMachinePlatform");

    if let Some(diag) =
        dism_failure_diagnostic(subsystem_error, vm_platform_error, reboot_required)
    {
        return diag;
    }

    let status = command_probe(wsl_exe, &["--status"]);
    let mut list = command_probe(wsl_exe, &["-l", "-v"]);
    if list
        .text
        .to_ascii_lowercase()
        .contains("no installed distributions")
    {
        list.error = None;
    }

    classify_wsl_probe(WslProbe {
        subsystem_feature,
        vm_platform_feature,
        default_version_2: parse_wsl_default_version_2(&status.text),
        distro_version_2_present: parse_wsl_list_has_v2(&list.text),
        reboot_required,
        status_error: status.error,
        list_error: list.error,
    })
}

#[cfg(not(target_os = "windows"))]
fn check_wsl_diagnostic() -> WslDiagnostic {
    WslDiagnostic {
        state: WslState::NotApplicable,
        wsl_installed: None,
        wsl2_enabled: None,
        reboot_required: false,
        error: None,
        features_enabled: None,
        default_version_2: None,
        distro_version_2_present: None,
    }
}

#[cfg(target_os = "windows")]
fn docker_desktop_installed_windows() -> bool {
    let common_paths = [
        r"C:\Program Files\Docker\Docker\Docker Desktop.exe",
        r"C:\Program Files\Docker\Docker\DockerCli.exe",
        r"C:\Program Files\Docker\Docker\resources\bin\docker.exe",
    ];

    if common_paths
        .iter()
        .any(|path| std::path::Path::new(path).exists())
    {
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

async fn check_docker_diagnostic(docker: &bollard::Docker) -> DockerDiagnostic {
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
    wsl_status_from_diagnostic(check_wsl_diagnostic())
}

#[cfg(any(target_os = "windows", test))]
fn wsl_status_from_diagnostic(diagnostic: WslDiagnostic) -> WslStatus {
    WslStatus {
        wsl_installed: diagnostic.wsl_installed.unwrap_or(false),
        wsl2_enabled: matches!(diagnostic.state, WslState::Ok),
        reboot_required: diagnostic.reboot_required,
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
/// Launches an elevated PowerShell command to enable the Windows WSL2 features.
pub fn enable_wsl2() -> Result<(), CoreError> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
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
        Err(CoreError::Validation(format!(
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
pub fn set_wsl_default_version() -> Result<(), CoreError> {
    let output = std::process::Command::new(r"C:\Windows\System32\wsl.exe")
        .args(["--set-default-version", "2"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        Err(CoreError::Validation(format!(
            "wsl --set-default-version 2 failed: {err}"
        )))
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
/// Opens Docker Desktop without waiting for the engine to finish starting.
pub fn open_docker_desktop() -> Result<(), CoreError> {
    let common_paths = [
        r"C:\Program Files\Docker\Docker\Docker Desktop.exe",
        r"C:\Program Files\Docker\Docker\DockerCli.exe",
    ];

    for path in common_paths {
        if std::path::Path::new(path).exists() {
            std::process::Command::new(path).spawn()?;
            return Ok(());
        }
    }

    Err(CoreError::Validation(
        "Docker Desktop is not installed or could not be found".to_string(),
    ))
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
) -> Result<OnboardingStatus, CoreError> {
    Ok(OnboardingStatus {
        platform: platform_name(),
        docker: check_docker_diagnostic(&state.host.docker).await,
        wsl: check_wsl_diagnostic(),
    })
}

#[tauri::command]
pub async fn get_public_ip() -> Result<String, CoreError> {
    let ip = reqwest::Client::new()
        .get("https://api.ipify.org")
        .send()
        .await
        .map_err(|e| CoreError::Validation(e.to_string()))?
        .text()
        .await
        .map_err(|e| CoreError::Validation(e.to_string()))?;
    Ok(ip.trim().to_string())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), CoreError> {
    std::fs::create_dir_all(&path)?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer.exe")
        .arg(&path)
        .spawn()?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&path).spawn()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(
        subsystem_feature: OptionalFeatureState,
        vm_platform_feature: OptionalFeatureState,
        default_version_2: Option<bool>,
    ) -> WslProbe {
        WslProbe {
            subsystem_feature,
            vm_platform_feature,
            default_version_2,
            distro_version_2_present: false,
            reboot_required: false,
            status_error: None,
            list_error: None,
        }
    }

    #[test]
    fn parse_dism_feature_state_enabled() {
        assert_eq!(
            parse_dism_feature_state("Feature Name : VirtualMachinePlatform\nState : Enabled"),
            OptionalFeatureState::Enabled
        );
    }

    #[test]
    fn parse_dism_feature_state_disabled() {
        assert_eq!(
            parse_dism_feature_state(
                "Feature Name : Microsoft-Windows-Subsystem-Linux\nState : Disabled"
            ),
            OptionalFeatureState::Disabled
        );
    }

    #[test]
    fn parse_dism_feature_state_enable_pending() {
        assert_eq!(
            parse_dism_feature_state("State : Enable Pending"),
            OptionalFeatureState::EnablePending
        );
    }

    #[test]
    fn parse_wsl_default_version_2_true() {
        assert_eq!(
            parse_wsl_default_version_2("Default Distribution: Ubuntu\nDefault Version: 2"),
            Some(true)
        );
    }

    #[test]
    fn parse_wsl_default_version_2_false() {
        assert_eq!(
            parse_wsl_default_version_2("Default Version: 1"),
            Some(false)
        );
    }

    #[test]
    fn parse_wsl_default_version_2_unknown_when_no_distro_text() {
        assert_eq!(
            parse_wsl_default_version_2(
                "Windows Subsystem for Linux has no installed distributions."
            ),
            None
        );
    }

    #[test]
    fn parse_wsl_list_has_v2_true() {
        assert!(parse_wsl_list_has_v2(
            "  NAME      STATE           VERSION\n* Ubuntu    Running         2"
        ));
    }

    #[test]
    fn parse_wsl_list_has_v2_false_when_no_distributions() {
        assert!(!parse_wsl_list_has_v2(
            "Windows Subsystem for Linux has no installed distributions."
        ));
    }

    #[test]
    fn dism_single_probe_failure_is_check_failed_not_needs_install() {
        let diag = dism_failure_diagnostic(Some("dism failed: boom".into()), None, false)
            .expect("one failed probe must produce a diagnostic");
        assert_eq!(diag.state, WslState::CheckFailed);
        assert_eq!(diag.error.as_deref(), Some("dism failed: boom"));

        let diag = dism_failure_diagnostic(None, Some("vm probe failed".into()), true)
            .expect("one failed probe must produce a diagnostic");
        assert_eq!(diag.state, WslState::CheckFailed);
        assert!(diag.reboot_required);

        assert!(dism_failure_diagnostic(None, None, false).is_none());
    }

    #[test]
    fn classify_wsl_needs_install_when_features_disabled() {
        let diagnostic = classify_wsl_probe(probe(
            OptionalFeatureState::Disabled,
            OptionalFeatureState::Disabled,
            None,
        ));

        assert_eq!(diagnostic.state, WslState::NeedsInstall);
        assert_eq!(diagnostic.features_enabled, Some(false));
        assert_eq!(diagnostic.wsl2_enabled, Some(false));
    }

    #[test]
    fn classify_wsl_reboot_required_when_enable_pending() {
        let diagnostic = classify_wsl_probe(probe(
            OptionalFeatureState::EnablePending,
            OptionalFeatureState::Enabled,
            None,
        ));

        assert_eq!(diagnostic.state, WslState::RebootRequired);
        assert!(diagnostic.reboot_required);
    }

    #[test]
    fn classify_wsl_needs_default_v2_when_default_explicitly_v1() {
        let diagnostic = classify_wsl_probe(probe(
            OptionalFeatureState::Enabled,
            OptionalFeatureState::Enabled,
            Some(false),
        ));

        assert_eq!(diagnostic.state, WslState::NeedsDefaultV2);
        assert_eq!(diagnostic.features_enabled, Some(true));
        assert_eq!(diagnostic.default_version_2, Some(false));
    }

    #[test]
    fn classify_wsl_ok_when_features_enabled_default_v2_no_distro() {
        let diagnostic = classify_wsl_probe(probe(
            OptionalFeatureState::Enabled,
            OptionalFeatureState::Enabled,
            Some(true),
        ));

        assert_eq!(diagnostic.state, WslState::Ok);
        assert_eq!(diagnostic.wsl_installed, Some(true));
        assert_eq!(diagnostic.wsl2_enabled, Some(true));
        assert_eq!(diagnostic.distro_version_2_present, Some(false));
    }

    #[test]
    fn classify_wsl_ok_when_features_enabled_default_unknown_no_distro() {
        let diagnostic = classify_wsl_probe(probe(
            OptionalFeatureState::Enabled,
            OptionalFeatureState::Enabled,
            None,
        ));

        assert_eq!(diagnostic.state, WslState::Ok);
        assert_eq!(diagnostic.default_version_2, None);
        assert_eq!(diagnostic.distro_version_2_present, Some(false));
    }

    #[test]
    fn legacy_wsl_status_maps_ok_to_wsl2_enabled_true() {
        let diagnostic = classify_wsl_probe(probe(
            OptionalFeatureState::Enabled,
            OptionalFeatureState::Enabled,
            None,
        ));
        let status = wsl_status_from_diagnostic(diagnostic);

        assert!(status.wsl_installed);
        assert!(status.wsl2_enabled);
        assert!(!status.reboot_required);
    }
}
