#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod settings;

use engram_core::Db;
use std::sync::Arc;
use tauri::Manager;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let db = tauri::async_runtime::block_on(async {
                Db::open_default().await.expect("DB open failed")
            });
            app.manage(Arc::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session_restore,
            commands::board_status,
            commands::issue_list,
            commands::issue_get,
            commands::issue_set_status,
            commands::issue_set_priority,
            commands::epic_list,
            commands::sprint_current,
            commands::task_list,
            commands::task_set_status,
            commands::note_list,
            commands::note_get,
            commands::note_add,
            commands::note_resolve,
            commands::blocked_issues_graph,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
