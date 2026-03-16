use crate::error::AppError;
use crate::ports;

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
