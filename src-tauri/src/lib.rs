mod commands;
mod event_sink;
mod state;

pub use cubelit_core::error;

use commands::docker_commands::*;
use commands::file_commands::*;
use commands::minecraft_commands::*;
use commands::recipe_commands::*;
use commands::server_commands::*;
use commands::system_commands::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize Sentry crash reporting. The DSN is read from the SENTRY_DSN
    // environment variable at runtime. Release builds set this via release.yml;
    // if the var is absent or empty, Sentry is a no-op and the guard drops harmlessly.
    let _sentry_guard = sentry::init((
        std::env::var("SENTRY_DSN").unwrap_or_default(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

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

                let log_file = data_dir.join("cubelit.log");
                if let Ok(file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file)
                {
                    let _ = tracing_subscriber::fmt()
                        .with_writer(std::sync::Mutex::new(file))
                        .with_env_filter("warn,cubelit=info")
                        .try_init();
                }

                let recipes_dir = resolve_recipes_dir(&app_handle);

                let state = state::AppState::new(data_dir, recipes_dir)
                    .await
                    .expect("failed to initialize app state");

                // Sync server statuses with Docker reality
                let _ = cubelit_core::server::sync_all_servers(&state.host.docker, &state.host.db)
                    .await;

                // Promote any server stuck in "starting" to "running" — the
                // readiness watcher task dies with the process, so after a
                // restart there is no watcher to advance the status.
                let _ = cubelit_core::server::reconcile_orphaned_starting_servers(
                    &state.host.docker,
                    &state.host.db,
                )
                .await;

                // Clone handles before moving state into manage()
                let watcher_docker = state.host.docker.clone();
                let watcher_db = state.host.db.clone();
                let watcher_events = event_sink::TauriEventSink::shared(app_handle.clone());

                app_handle.manage(state);

                // Spawn background watcher to detect unexpected container crashes
                cubelit_core::server::spawn_crash_watcher(
                    watcher_docker,
                    watcher_db,
                    watcher_events,
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
            get_onboarding_status,
            get_public_ip,
            open_folder,
            #[cfg(target_os = "windows")]
            check_wsl_status,
            #[cfg(target_os = "windows")]
            enable_wsl2,
            #[cfg(target_os = "windows")]
            set_wsl_default_version,
            list_server_files,
            read_server_file,
            copy_file_to_server,
            delete_server_file,
            write_server_file,
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

fn resolve_recipes_dir(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
    #[cfg(debug_assertions)]
    {
        let source_recipes = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("recipes");
        if source_recipes.exists() {
            return source_recipes;
        }
    }

    app_handle
        .path()
        .resource_dir()
        .expect("failed to resolve resource dir")
        .join("recipes")
}
