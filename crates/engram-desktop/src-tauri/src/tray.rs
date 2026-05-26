use crate::mcp_supervisor::McpSupervisor;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

/// 팝오버가 마지막으로 show 된 시각(ms since UNIX_EPOCH).
///
/// fullscreen Space 에서 set_focus 가 OS 의 거부로 즉시 Focused(false) 를 트리거하면
/// `on_window_event` 의 auto-hide 가 팝오버를 곧바로 닫아버린다. show 직후 짧은
/// grace period 동안 hide 를 막기 위해 main.rs 의 핸들러에서 이 값을 참조한다.
pub static POPOVER_SHOWN_AT_MS: AtomicU64 = AtomicU64::new(0);
static POPOVER_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Focused(false) 이벤트를 무시할 grace period (ms).
pub const POPOVER_AUTO_HIDE_GRACE_MS: u64 = 200;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

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

    let tray = TrayIconBuilder::with_id("default")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("Engram")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = ev
            {
                let (px, py) = match rect.position {
                    tauri::Position::Physical(p) => (p.x as f64, p.y as f64),
                    tauri::Position::Logical(p) => (p.x, p.y),
                };
                let (sw, sh) = match rect.size {
                    tauri::Size::Physical(s) => (s.width as f64, s.height as f64),
                    tauri::Size::Logical(s) => (s.width, s.height),
                };
                let icon_cx = px + sw / 2.0;
                let icon_bottom = py + sh;
                show_or_hide_popover(tray.app_handle(), icon_cx, icon_bottom);
            }
        })
        .build(app)?;

    // macOS: 흰 사각형 아이콘 제거 — title 텍스트만 메뉴바에 표시
    // watcher가 첫 tick(5s 후) set_title 을 호출하기 전에도 아이콘이 보이지 않도록 즉시 제거
    #[cfg(target_os = "macos")]
    let _ = tray.set_icon(None);
    drop(tray);

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

fn show_or_hide_popover(app: &AppHandle, icon_cx: f64, icon_bottom: f64) {
    if let Some(popover) = app.get_webview_window("tray_popover") {
        if popover.is_visible().unwrap_or(false) {
            let _ = popover.hide();
        } else {
            const POPUP_W: f64 = 380.0;
            let x = (icon_cx - POPUP_W / 2.0).max(0.0);
            let y = icon_bottom;
            let _ = popover.set_position(tauri::Position::Physical(
                tauri::PhysicalPosition::new(x as i32, y as i32),
            ));

            if !POPOVER_INITIALIZED.load(Ordering::Relaxed) {
                let _ = popover.set_visible_on_all_workspaces(true);
                #[cfg(target_os = "macos")]
                macos_enable_fullscreen_popover(&popover);
                POPOVER_INITIALIZED.store(true, Ordering::Relaxed);
            }

            POPOVER_SHOWN_AT_MS.store(now_ms(), Ordering::Relaxed);
            let _ = popover.show();
            let _ = popover.set_focus();

            // fullscreen Space 에서 OS 가 첫 set_focus 를 거부할 수 있으므로
            // 200ms 후 한 번 더 재시도한다.
            let popover_clone = popover.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let _ = popover_clone.set_focus();
            });
        }
    }
}

/// macOS 에서 트레이 팝오버가 사용자가 fullscreen 상태인 다른 앱의 Space 위에도 떠야 한다.
///
/// `set_visible_on_all_workspaces(true)` 는 `NSWindowCollectionBehaviorCanJoinAllSpaces`
/// 만 세팅하지만, fullscreen Space 침투에는 `FullScreenAuxiliary` 가 추가로 필요하다.
/// 추가로 `Stationary | IgnoresCycle` 을 켜 Mission Control 탭 사이클에서 빼고,
/// 윈도우 레벨을 NSPopUpMenuWindowLevel(101) 로 올려 fullscreen 앱이 띄우는
/// 어떤 시스템 UI 위에도 표시되게 한다.
///
/// **호출 시점**: 앱 초기화(setup) 단계에서 최초 1회 설정하여 윈도우의 속성을 고정한다.
#[cfg(target_os = "macos")]
pub fn macos_enable_fullscreen_popover(popover: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    let Ok(ns_ptr) = popover.ns_window() else { return };
    if ns_ptr.is_null() {
        return;
    }
    let ns_window: *const NSWindow = ns_ptr as *const NSWindow;
    unsafe {
        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Transient
            | NSWindowCollectionBehavior::FullScreenAuxiliary;
        (*ns_window).setCollectionBehavior(behavior);
        // 전체화면 앱 레이어 위에 팝오버가 확실히 표시될 수 있도록 팝업 메뉴 레벨(101)로 설정함.
        // extern static 상수는 dynamic link 주소 매핑 문제로 예외를 던질 수 있어 정수 리터럴을 직접 사용함.
        (*ns_window).setLevel(101);
    }
}
