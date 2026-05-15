# M1 — 스캐폴딩 + 보드 읽기

> **상위 문서**: [overview.md](./overview.md) · **이전**: [m0-foundations.md](./m0-foundations.md) · **다음**: [m2-dnd-drawer.md](./m2-dnd-drawer.md)

**예상 기간**: 1주

## 전제

[M0](./m0-foundations.md) 의 모든 변경 사항 머지됨 — `engram-mcp` lib 사용 가능, `Db::*_update(.., changed_by)` 시그니처 사용 가능, `max_connections=5` 적용됨.

## 목표

`engram-desktop` 크레이트를 생성하고 Tauri v2 + React + Tailwind + shadcn/ui 토대를 깐다. 칸반 보드의 **읽기 전용** 5컬럼 뷰까지 표시.

## Scope

### 1. 크레이트 스캐폴딩

```bash
mkdir -p crates/engram-desktop
cd crates/engram-desktop
# Tauri v2 CLI 로 생성
cargo install create-tauri-app --locked
cargo create-tauri-app --template react-ts --manager pnpm engram-desktop-ui
```

**`crates/engram-desktop/Cargo.toml`**:
```toml
[package]
name = "engram-desktop"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2" }

[dependencies]
engram-core = { path = "../engram-core" }
engram-mcp  = { path = "../engram-mcp" }   # lib 만 사용
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-single-instance = "2"
tauri-plugin-notification = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "registry"] }
chrono = { version = "0.4", features = ["serde"] }
```

**Workspace `Cargo.toml`**: `members` 에 `"crates/engram-desktop"` 추가.

### 2. Tauri 부트

**`crates/engram-desktop/src/main.rs`**:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod settings;

use engram_core::Db;
use std::sync::Arc;
use tracing_subscriber::prelude::*;

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // 두 번째 인스턴스 진입 시 기존 창 포커스
            if let Some(w) = app.get_webview_window("main") { let _ = w.show(); let _ = w.set_focus(); }
        }))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let rt = tauri::async_runtime::block_on(async {
                let db = Db::open_default().await.expect("DB open");
                Ok::<_, anyhow::Error>(Arc::new(db))
            })?;
            app.manage(rt);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session_restore,
            commands::board_status,
            commands::issue_list,
            commands::issue_get,
            commands::epic_list,
            commands::sprint_current,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

> **주의**: `#[tokio::main]` 사용 금지. Tauri v2 가 자체 runtime 을 소유.

### 3. Read-only Tauri Commands

`crates/engram-desktop/src/commands.rs` — 읽기 명령만 (쓰기 명령은 M2 부터):

```rust
use engram_core::{Db, repository::session::*};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn session_restore(
    db: State<'_, Arc<Db>>,
    project_key: Option<String>,
) -> Result<SessionSnapshot, String> {
    db.session_restore(project_key.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn board_status(/* ... */) -> Result<BoardStatus, String> { /* ... */ }
#[tauri::command]
pub async fn issue_list(/* ... */) -> Result<Vec<Issue>, String> { /* ... */ }
#[tauri::command]
pub async fn issue_get(/* ... */) -> Result<Issue, String> { /* ... */ }
#[tauri::command]
pub async fn epic_list(/* ... */) -> Result<Vec<Epic>, String> { /* ... */ }
#[tauri::command]
pub async fn sprint_current(/* ... */) -> Result<Option<Sprint>, String> { /* ... */ }
```

### 4. UI 스택 셋업

```bash
cd crates/engram-desktop/ui
pnpm add @tanstack/react-query zustand lucide-react
pnpm add @radix-ui/react-slot class-variance-authority clsx tailwind-merge
pnpm add -D tailwindcss postcss autoprefixer @types/react
pnpm dlx tailwindcss init -p
pnpm dlx shadcn@latest init
pnpm dlx shadcn@latest add button card badge dropdown-menu drawer separator
```

**`tailwind.config.ts`** 의 `content` 에 `./src/**/*.{ts,tsx,html}` 포함.

### 5. 읽기 전용 칸반 UI

**`crates/engram-desktop/ui/src/ipc/invoke.ts`**:
```ts
import { invoke } from '@tauri-apps/api/core';
import type { SessionSnapshot, BoardStatus, Issue, Epic, Sprint } from './types';

export const sessionRestore = (project_key?: string) =>
  invoke<SessionSnapshot>('session_restore', { project_key });
export const boardStatus = (project_key?: string) =>
  invoke<BoardStatus>('board_status', { project_key });
export const issueList = (filter: IssueFilter) =>
  invoke<Issue[]>('issue_list', { filter });
// ...
```

타입은 `engram-core` 의 Serialize 출력을 그대로 받아 TS 로 미러링. 첫 단계는 hand-written, 추후 `ts-rs` 등으로 자동화 검토.

**`useBoardStatus.ts`** (Tanstack Query):
```ts
export const useBoardStatus = (projectKey?: string) => useQuery({
  queryKey: ['boardStatus', projectKey],
  queryFn: () => boardStatus(projectKey),
  refetchInterval: 5000,
});
```

**컴포넌트**:
- `KanbanBoard.tsx` — 5컬럼 그리드 (`required/ready/working/demo/finished`)
- `KanbanColumn.tsx` — 컬럼 헤더 + 카드 리스트. Demo 만 amber 강조
- `IssueCard.tsx` — priority dot + title + #id + epic chip + task progress
- `PriorityBadge.tsx`, `EpicChip.tsx`

DnD 없음. 카드 클릭은 console.log 로만 (M2 에서 Drawer 연결).

### 6. 라우팅

shadcn drawer + 간단한 라우팅은 React Router 없이 Zustand 로 충분 (route count 적음):

```ts
// store/ui.ts
type View = 'board' | 'sprint' | 'mcp';
type UIState = { view: View; selectedIssueId: number | null; ... };
```

## 변경 파일 목록

```
Cargo.toml                                            (M)  members += engram-desktop
crates/engram-desktop/                                (+)  새 크레이트 전체
  Cargo.toml                                          (+)
  tauri.conf.json                                     (+)
  src/main.rs                                         (+)
  src/commands.rs                                     (+)
  src/settings.rs                                     (+)  M1 에서는 stub (M3 에서 채움)
  ui/                                                 (+)  Vite + React + TS 트리
    package.json, tsconfig.json, vite.config.ts
    tailwind.config.ts, postcss.config.js
    src/main.tsx, src/App.tsx
    src/ipc/invoke.ts, src/ipc/types.ts
    src/hooks/useBoardStatus.ts, useSessionRestore.ts
    src/store/ui.ts
    src/components/KanbanBoard.tsx, KanbanColumn.tsx, IssueCard.tsx,
                   PriorityBadge.tsx, EpicChip.tsx
    src/routes/Board.tsx
```

## Verification

1. **빌드**
   ```bash
   cd crates/engram-desktop/ui && pnpm install && pnpm build
   cargo build -p engram-desktop
   ```
2. **개발 모드**
   ```bash
   pnpm --filter engram-desktop-ui dev
   cargo tauri dev -p engram-desktop
   ```
3. **수동 시나리오**
   - CLI 로 sprint/epic/issue 만들고 데스크톱 새로 띄움 → 5컬럼에 카드 표시
   - 두 번째 인스턴스 실행 시 기존 창 포커스 (single-instance 동작)
   - 카드 클릭 시 console.log (M2 에서 Drawer 로 교체 예정)
4. **테스트**
   - 프론트: `pnpm test` (Vitest) — `IssueCard` 렌더링, `KanbanBoard` 컬럼 분배 로직
   - Rust: `cargo test -p engram-desktop` — Tauri command 가 `Db` 메서드를 올바르게 위임하는지 (mock Db 활용)

## Out of Scope

- DnD / 상태 전이 (→ M2)
- Issue Detail Drawer (→ M2)
- MCP Supervisor (→ M3)
- Tray (→ M4)

## 완료 기준

- [ ] Tauri 앱 기동 + 메인 윈도우에 칸반 5컬럼 표시
- [ ] CLI 로 만든 데이터가 5초 내 자동 반영 (Tanstack Query refetch)
- [ ] single-instance 플러그인 동작
- [ ] `pnpm build` 및 `cargo build -p engram-desktop` clean
