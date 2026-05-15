use crate::mcp_supervisor::McpSupervisor;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "open_board", "보드 열기", true, None::<&str>)?,
            &MenuItem::with_id(app, "mcp_open", "MCP 매니저 열기", true, None::<&str>)?,
            &MenuItem::with_id(app, "mcp_restart", "MCP 재시작", true, None::<&str>)?,
            &MenuItem::with_id(app, "mcp_stop", "MCP 정지", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some("종료"))?,
        ],
    )?;

    let icon = Image::from_bytes(include_bytes!("../icons/tray-template.png"))?;

    TrayIconBuilder::with_id("default")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("Engram")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = ev
            {
                show_or_hide_popover(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn handle_menu_event(app: &AppHandle, ev: tauri::menu::MenuEvent) {
    match ev.id().as_ref() {
        "open_board" => {
            show_main_window(app, "board");
        }
        "mcp_open" => {
            show_main_window(app, "mcp");
        }
        "mcp_restart" => {
            if let Some(sup) = app.try_state::<Arc<McpSupervisor>>() {
                let s = Arc::clone(&sup);
                tauri::async_runtime::spawn(async move {
                    let snap = s.status().await;
                    let _ = s.restart(snap.port).await;
                });
            }
        }
        "mcp_stop" => {
            if let Some(sup) = app.try_state::<Arc<McpSupervisor>>() {
                let s = Arc::clone(&sup);
                tauri::async_runtime::spawn(async move {
                    let _ = s.stop().await;
                });
            }
        }
        _ => {}
    }
}

fn show_main_window(app: &AppHandle, _view: &str) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn show_or_hide_popover(app: &AppHandle) {
    if let Some(popover) = app.get_webview_window("tray_popover") {
        if popover.is_visible().unwrap_or(false) {
            let _ = popover.hide();
        } else {
            let _ = popover.show();
            let _ = popover.set_focus();
        }
    }
}
