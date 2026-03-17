mod commands;
mod db;
mod docker;
mod error;
mod ports;
mod recipes;
mod state;

use commands::docker_commands::*;
use commands::file_commands::*;
use commands::minecraft_commands::*;
use commands::recipe_commands::*;
use commands::server_commands::*;
use commands::system_commands::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::block_on(async move {
                let data_dir = app_handle
                    .path()
                    .app_data_dir()
                    .expect("failed to resolve app data dir");

                let recipes_dir = app_handle
                    .path()
                    .resource_dir()
                    .expect("failed to resolve resource dir")
                    .join("recipes");

                let state = state::AppState::new(data_dir, recipes_dir)
                    .await
                    .expect("failed to initialize app state");

                // Sync server statuses with Docker reality
                let _ = sync_all_servers(&state.docker, &state.db).await;

                // Clone handles before moving state into manage()
                let watcher_docker = state.docker.clone();
                let watcher_db = state.db.clone();
                let watcher_handle = app_handle.clone();

                app_handle.manage(state);

                // Spawn background watcher to detect unexpected container crashes
                commands::docker_commands::spawn_crash_watcher(
                    watcher_docker,
                    watcher_db,
                    watcher_handle,
                );
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_docker_status,
            create_server,
            start_server,
            stop_server,
            restart_server,
            delete_server,
            sync_server_status,
            sync_all_statuses,
            list_cubelits,
            get_cubelit,
            list_recipes,
            get_recipe_detail,
            check_port,
            suggest_port,
            get_public_ip,
            open_folder,
            list_server_files,
            copy_file_to_server,
            delete_server_file,
            get_server_logs,
            get_server_stats,
            update_server_settings,
            rename_server,
            send_minecraft_command,
            backup_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
