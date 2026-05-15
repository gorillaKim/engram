# M4 — 메뉴바 트레이 + 알림

> **상위 문서**: [overview.md](./overview.md) · **이전**: [m3-mcp-supervisor.md](./m3-mcp-supervisor.md) · **다음**: [m5-polish.md](./m5-polish.md)

**예상 기간**: 3~4일

## 전제

- M1~M3 완료 — 보드 + Drawer + MCP Supervisor 동작.

## 목표

macOS 메뉴바에 항상 떠 있는 트레이 인디케이터. 클릭하면 프로젝트별 진행률·최근 알림·MCP 상태가 떠 있는 popover. `required` 진입 / `demo` 진입 / 신규 blocker 발생 시 NSNotificationCenter 푸시.

## Scope

### 1. Tauri Tray

**`crates/engram-desktop/src/tray.rs`** (신규):

```rust
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(app, &[
        &MenuItem::with_id(app, "open_board",    "보드 열기",        true, None::<&str>)?,
        &MenuItem::with_id(app, "session_ctx",   "세션 컨텍스트",     true, None::<&str>)?,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "mcp_open",      "MCP 매니저 열기",  true, None::<&str>)?,
        &MenuItem::with_id(app, "mcp_restart",   "MCP 재시작",       true, None::<&str>)?,
        &MenuItem::with_id(app, "mcp_stop",      "MCP 정지",         true, None::<&str>)?,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "settings",      "환경설정",         true, None::<&str>)?,
        &PredefinedMenuItem::quit(app, Some("종료"))?,
    ])?;

    let _tray = TrayIconBuilder::new()
        .icon(Image::from_path("icons/tray-template.png")?)
        .icon_as_template(true)   // macOS dark mode 대응
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, ev| handle_menu_event(app, ev))
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = ev {
                let app = tray.app_handle();
                show_popover(&app);
            }
        })
        .build(app)?;
    Ok(())
}
```

### 2. 메뉴바 텍스트/뱃지

Tauri v2 의 tray title 은 동적 set 가능. `Db::board_status_query` 를 5초마다 폴링해서 결과를 텍스트로 반영.

```rust
// watcher.rs
let summary = compute_summary(&db).await?;  // { inbox: u32, demo_review: u32, blockers: u32 }
tray.set_title(Some(&format!("📦 {} · ⚠ {}", summary.inbox, summary.demo_review)))?;
```

App Nap 시 throttling 되는 것 정상 (Opus 권고 E — 별도 대응 안 함).

### 3. Popover 윈도우

메인 윈도우와 별개의 작은 panel-style 윈도우 (`tauri.conf.json` 의 `windows` 에 `tray_popover` 추가, `decorations: false`, `transparent: true`, `always_on_top: true`, `skip_taskbar: true`):

```jsonc
{
  "label": "tray_popover",
  "url": "tray.html",
  "width": 380,
  "height": 520,
  "visible": false,
  "decorations": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "transparent": true,
  "resizable": false
}
```

`show_popover(app)` 가 트레이 아이콘 위치 기준으로 윈도우를 띄움.

**`ui/tray.html`** + **`ui/src/tray.tsx`** (Vite 빌드 entry 분리):
- 프로젝트별 progress bar (`board_status` 사용)
- 최근 알림 5개 (Zustand `notificationLog` 에서 끌어옴)
- MCP 상태 라인 + [열기/재시작/정지] 버튼

### 4. Watcher + 알림 디바운스

**`crates/engram-desktop/src/watcher.rs`**:

```rust
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

pub async fn run(app: AppHandle, db: Arc<Db>) {
    let mut last = BoardSnapshot::default();
    let mut last_notif_at = HashMap::<String, Instant>::new();   // key → last time
    let cooldown = Duration::from_secs(30);

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let cur = compute_summary(&db).await.unwrap_or_default();
        let diff = cur.diff(&last);

        for ev in diff.into_events() {
            let key = ev.dedupe_key();
            if last_notif_at.get(&key).map_or(true, |t| t.elapsed() > cooldown) {
                app.notification().builder()
                    .title(ev.title())
                    .body(ev.body())
                    .show()?;
                last_notif_at.insert(key, Instant::now());
            }
        }

        app.emit("tray://summary", &cur).ok();
        last = cur;
    }
}
```

**이벤트 종류**:
- `new_required`: `🆕 새 이슈가 승인 대기: #{id} {title}` (epic_key 기준 dedupe)
- `entered_demo`: `👀 검토 대기: #{id} {title}` (issue_id 기준 dedupe)
- `new_blocker`: `🚫 새 블로커: #{a} blocks #{b}` (link_id 기준 dedupe)

쿨다운 30초로 같은 키 반복 알림 억제.

### 5. MCP 상태와 연동

`mcp://status` 이벤트를 트레이 윈도우도 구독 → ●/○ 색과 카운트 갱신. 메뉴바 텍스트는 변경하지 않음 (이슈 카운트 우선).

### 6. 메뉴 액션

| 메뉴 id | 동작 |
|---|---|
| `open_board` | 메인 윈도우 show + focus, view='board' |
| `session_ctx` | 메인 윈도우 show + view='session' (Drawer 형태) |
| `mcp_open` | 메인 윈도우 show + view='mcp' |
| `mcp_restart` | `mcp_supervisor.restart(current_port)` |
| `mcp_stop` | `mcp_supervisor.stop()` |
| `settings` | 환경설정 윈도우 (M5 에서 본격 디자인. M4 에서는 단순 dialog) |

## 변경 파일 목록

```
crates/engram-desktop/src/main.rs                     (M)  tray::build, watcher::run spawn
crates/engram-desktop/src/tray.rs                     (+)  신규
crates/engram-desktop/src/watcher.rs                  (+)  실제 구현 (M1 stub 교체)
crates/engram-desktop/tauri.conf.json                 (M)  tray_popover 윈도우 추가
crates/engram-desktop/Cargo.toml                      (M)  tauri-plugin-notification 추가
crates/engram-desktop/ui/                             (+)
  tray.html
  src/tray.tsx
  src/components/TraySummary.tsx
  src/components/TrayNotificationList.tsx
  src/components/TrayMcpStatus.tsx
icons/tray-template.png                               (+)  모노크롬 16x16
```

## Verification

1. **빌드**
   ```bash
   cargo build -p engram-desktop
   pnpm --filter engram-desktop-ui build
   ```
2. **수동 시나리오**
   - 앱 기동 → 메뉴바 우측에 아이콘 등장 (예: `📦 12 · ⚠ 2`)
   - 클릭 → popover 윈도우가 트레이 아이콘 근처에 표시
   - CLI 로 `engram issue create` → 5초 이내 NSNotificationCenter 알림 등장
   - 같은 이슈를 또 만들면 30초 안에는 알림 안 옴 (cooldown)
   - 5초 안에 5개를 연속 만들면 5개 알림 모두 (다른 issue_id)
   - 트레이 메뉴 `MCP 재시작` 클릭 → McpManager UI 의 status 가 갱신, curl 로 응답 복귀
3. **App Nap**
   - 메인 윈도우 숨기고 10분간 두기 → throttled 되어 폴링 빈도 떨어지지만 정상
4. **dark mode**
   - macOS 다크 모드 토글 → 트레이 아이콘 색 자동 반전 (`icon_as_template(true)`)

## Out of Scope

- 알림 카테고리별 음소거 설정 (→ M5)
- 도크 뱃지 (검토 대기 카운트) (→ M5 검토)
- SQLite update hook 기반 push (Phase 2 이후, 폴링이 충분히 가벼우면 보류)

## 완료 기준

- [x] 메뉴바 아이콘 + 동적 텍스트 표시
- [x] 좌클릭 popover 표시
- [x] 우클릭 컨텍스트 메뉴 동작
- [x] 3종 알림 (required/demo/blocker) + 30초 cooldown
- [x] MCP 상태 popover/메뉴 반영
- [x] App Nap 환경에서도 충돌 없음
