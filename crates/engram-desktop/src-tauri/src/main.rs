#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod mcp_supervisor;
mod settings;
mod tracing_layer;
mod tray;
mod watcher;

use crate::mcp_supervisor::McpSupervisor;
use crate::tracing_layer::BroadcastLayer;
use engram_core::Db;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tracing_subscriber::prelude::*;

/// DB open / migration 실패를 사용자에게 알리고 진단용 로그를 남긴다.
/// silent panic 으로 앱이 그냥 "열리지 않는" 상황(예: 깨진 마이그레이션)을 방지한다.
fn report_db_open_failure(err: &engram_core::Error) {
    let message = format!(
        "Engram 데이터베이스를 여는 중 오류가 발생했습니다.\n\n\
         {err}\n\n\
         최신 버전으로 업데이트하면 자동으로 복구될 수 있습니다.\n\
         문제가 계속되면 ~/.engram/db-open-error.log 를 확인해 주세요."
    );

    // 1) stderr 로그 (시스템 콘솔에서 확인 가능)
    eprintln!("[engram] DB open failed: {err}");

    // 2) ~/.engram/db-open-error.log 에 append (재발 시 진단용)
    if let Ok(home) = std::env::var("HOME") {
        let _ = std::fs::create_dir_all(format!("{home}/.engram"));
        let log_path = format!("{home}/.engram/db-open-error.log");
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            use std::io::Write;
            let _ = writeln!(f, "[{}] {err}", chrono::Local::now().to_rfc3339());
        }
    }

    // 3) 네이티브 에러 다이얼로그
    show_native_error_dialog(&message);
}

#[cfg(target_os = "macos")]
fn show_native_error_dialog(message: &str) {
    // AppleScript 문자열 리터럴 escape (백슬래시, 큰따옴표). 개행은 그대로 허용된다.
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        "display dialog \"{escaped}\" with title \"Engram\" buttons {{\"확인\"}} default button 1 with icon stop"
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();
}

#[cfg(not(target_os = "macos"))]
fn show_native_error_dialog(_message: &str) {
    // 비-macOS 빌드: stderr/로그 파일로 대체 (현재 배포 타깃은 macOS).
}

#[cfg(target_os = "macos")]
use tauri_nspanel::{tauri_panel, WebviewWindowExt};

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(Panel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false
        }
    })
    panel_event!(PanelEventHandler {})
}

fn main() {
    let settings = settings::load().unwrap_or_default();

    // DB open + migration 자동 적용. 실패 시 silent crash 대신
    // 크래시 로그 + 네이티브 다이얼로그로 원인을 안내한 뒤 종료한다.
    let db = match tauri::async_runtime::block_on(Db::open_default()) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            report_db_open_failure(&e);
            std::process::exit(1);
        }
    };
    let supervisor = McpSupervisor::new(Arc::clone(&db), settings.mcp.autostart);

    // Layered tracing: fmt stderr + env-filter + broadcast for McpManager UI
    let log_tx = supervisor.log_sender();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(BroadcastLayer { tx: log_tx })
        .init();

    // Autostart MCP server
    if settings.mcp.autostart {
        let port = settings.mcp.port;
        let s = Arc::clone(&supervisor);
        tauri::async_runtime::block_on(async move {
            if let Err(e) = s.start(port).await {
                tracing::warn!("MCP autostart failed: {e}");
            }
        });
    }

    let supervisor_for_setup = Arc::clone(&supervisor);

    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(move |app| {
            app.manage(db);
            app.manage(supervisor_for_setup);

            #[cfg(target_os = "macos")]
            {
                if let Some(popover) = app.get_webview_window("tray_popover") {
                    match popover.to_panel::<Panel>() {
                        Ok(panel) => {
                            panel.set_hides_on_deactivate(false);
                            panel.set_floating_panel(true);
                            panel.set_level(tauri_nspanel::PanelLevel::Status.value());
                            panel.set_style_mask(
                                tauri_nspanel::StyleMask::empty().nonactivating_panel().into(),
                            );
                            let mut behavior = tauri_nspanel::CollectionBehavior::new();
                            behavior = behavior.can_join_all_spaces().full_screen_auxiliary().stationary();
                            panel.set_collection_behavior(behavior.into());
                        }
                        Err(e) => {
                            tracing::error!("Failed to convert tray_popover to NSPanel: {:?}", e);
                        }
                    }
                }
            }

            // Build tray icon + menu
            tray::build(app.handle())?;

            // Spawn board watcher (emits tray://summary, sends notifications)
            {
                let db_for_watcher = app.state::<Arc<Db>>().inner().clone();
                let ah_watcher = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    watcher::run(ah_watcher, db_for_watcher).await;
                });
            }

            let ah = app.handle().clone();
            let sup = app.state::<Arc<McpSupervisor>>().inner().clone();

            // Heartbeat: emit mcp://status every 10s
            {
                let ah2 = ah.clone();
                let s2 = Arc::clone(&sup);
                tauri::async_runtime::spawn(async move {
                    let mut interval = tokio::time::interval(
                        std::time::Duration::from_secs(10),
                    );
                    loop {
                        interval.tick().await;
                        let snap = s2.status().await;
                        let _ = ah2.emit("mcp://status", &snap);
                    }
                });
            }

            // Log pump: broadcast::Receiver → mcp://log event
            {
                let ah_log = ah.clone();
                let mut log_rx = sup.subscribe_logs();
                let sup_log = Arc::clone(&sup);
                tauri::async_runtime::spawn(async move {
                    while let Ok(line) = log_rx.recv().await {
                        sup_log.push_log(line.clone()).await;
                        let _ = ah_log.emit("mcp://log", &line);
                    }
                });
            }

            // Call pump: broadcast::Receiver<CallRecord> → mcp://call event
            {
                let ah_call = ah.clone();
                let mut call_rx = sup.call_broadcast_sender().subscribe();
                tauri::async_runtime::spawn(async move {
                    while let Ok(rec) = call_rx.recv().await {
                        let _ = ah_call.emit("mcp://call", &rec);
                    }
                });
            }

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
            commands::epic_get,
            commands::sprint_current,
            commands::sprint_list,
            commands::sprint_create,
            commands::sprint_update,
            commands::sprint_delete,
            commands::sprint_progress_list,
            commands::epic_set_sprint,
            commands::task_list,
            commands::task_set_status,
            commands::note_list,
            commands::note_get,
            commands::note_add,
            commands::note_resolve,
            commands::blocked_issues_graph,
            commands::blocking_graph_for_issue,
            commands::epic_create,
            commands::issue_create,
            commands::task_create,
            commands::task_delete,
            commands::issue_link,
            commands::issue_unlink,
            commands::issue_links,
            commands::epic_set_status,
            commands::issue_update,
            commands::epic_update,
            commands::epic_delete,
            commands::issue_delete,
            commands::history_list,
            commands::mcp_status,
            commands::mcp_start,
            commands::mcp_stop,
            commands::mcp_restart,
            commands::mcp_recent_calls,
            commands::mcp_recent_logs,
            commands::mcp_get_tool_definitions,
            commands::mcp_set_autostart,
            commands::get_activity_settings,
            commands::set_activity_settings,
            commands::hide_tray_popover,
            commands::show_main_window,
            commands::get_app_version,
            commands::relaunch_app,
            commands::mission_list,
            commands::mission_create,
            commands::mission_get,
            commands::mission_update,
            commands::mission_delete,
            commands::mission_get_progress,
            commands::mission_get_tree,
        ])
        .on_window_event(|window, event| {
            // Auto-hide tray popover when it loses focus (native popover behaviour).
            // 단, show 직후 grace period 안에 발생한 Focused(false) 는 무시한다 —
            // fullscreen Space 에서 OS 가 우리 팝오버에 focus 를 못 줘서 즉시
            // Focused(false) 가 튀는 시나리오를 막기 위함.
            if window.label() == "tray_popover" {
                if let tauri::WindowEvent::Focused(false) = event {
                    use std::sync::atomic::Ordering;
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    let elapsed = {
                        let shown = tray::POPOVER_SHOWN_AT_MS.load(Ordering::Relaxed);
                        now.saturating_sub(shown)
                    };
                    tracing::info!("Tray popover lost focus. Elapsed: {}ms, Grace: {}ms", elapsed, tray::POPOVER_AUTO_HIDE_GRACE_MS);
                    if elapsed >= tray::POPOVER_AUTO_HIDE_GRACE_MS {
                        let _ = window.hide();
                        tray::POPOVER_HIDDEN_AT_MS.store(now, Ordering::Relaxed);
                    }
                }
                return;
            }
            // Hide main window on close instead of destroying it (menu bar app pattern)
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
