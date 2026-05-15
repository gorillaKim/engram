# Engram Desktop & Tray Widget — Overview

## Context

현재 Engram 은 MCP/CLI 만 가지고 있어 Agent 가 만든 이슈/태스크를 사람이 **시각적으로 점검·승인** 할 통로가 없다. 특히 "Agent 는 `demo` 까지만, 사용자가 `finished` 처리" 라는 협업 규칙이 운영되려면, 사람이 한눈에 `demo` 컬럼을 보고 칸을 옮길 수 있는 칸반 UI 가 필수다.

이번 작업의 목표는 세 가지다.

1. **Confluence 스타일 칸반 데스크톱 앱** — 프로젝트별로 이슈를 컬럼(`required/ready/working/demo/finished`)에 띄우고 드래그-앤-드롭으로 상태 전이. Agent 와 사용자 둘 다 변경할 수 있되, 누가 옮겼는지 `history` 에 기록되어야 한다.
2. **macOS 메뉴바 트레이 위젯** — 항상 떠 있는 진행률·알림 인디케이터. 새 `required` 이슈, `demo` 대기, 블로커 발생 시 푸시.
3. **임베디드 MCP 서버 + 관리 UI** — 데스크톱 앱이 켜질 때 **Streamable HTTP** MCP 서버도 함께 기동. 앱 안에서 시작/정지/재시작/포트 변경/로그·호출 이력 확인. Claude Code / claude.ai 웹 등이 이 HTTP 엔드포인트로 붙는다.

Demo→Finished 전이의 사용자 전용 게이트는 **서브에이전트 프롬프트 + `.claude/rules/agent-demo-gate.md`** 로만 강제한다 (DB/MCP 레이어는 손대지 않음). 우발적 호출이 일어나도 칸반 UI 에서 사람이 되돌릴 수 있고, `history` 로 추적된다.

---

## Architecture

### Workspace 변경

```
engram/
├── crates/
│   ├── engram-core/        ← 그대로 (단, *_update 메서드 시그니처에 changed_by 추가)
│   ├── engram-mcp/         ← lib + bin 듀얼, sse.rs → http.rs
│   │   ├── src/lib.rs      ← pub mod tools / server / http  (신규)
│   │   ├── src/http.rs     ← Streamable HTTP transport (신규)
│   │   └── src/main.rs     ← lib 호출 shim
│   ├── engram-cli/         ← 그대로 (호출처에 changed_by="user" 전달)
│   └── engram-desktop/     ← 신규: Tauri v2 앱
│       ├── src/            ← Rust: commands, tray, watcher, mcp_supervisor, settings
│       └── ui/             ← Vite + React + TS + Tailwind + shadcn/ui + dnd-kit
```

### 의존성 흐름

```
engram-desktop  ──► engram-core (AppState 로 Db 보유)
                 └► engram-mcp (lib) — 임베디드 HTTP 서버 호스팅
```

Tauri command 는 `engram-core::Db` 의 기존 메서드를 그대로 호출한다 — 신규 비즈니스 로직 0.

### 핵심 스택

| 영역 | 선택 |
|---|---|
| 네이티브 셸 | Tauri v2 (tray, notification, single-instance 플러그인) |
| 프론트 | React 18 + TypeScript + Vite |
| 스타일 | Tailwind CSS + shadcn/ui |
| 컴포넌트 | lucide-react (아이콘) |
| DnD | @dnd-kit/core, @dnd-kit/sortable |
| 서버 상태 | @tanstack/react-query |
| 클라 상태 | Zustand |
| MCP 전송 | Streamable HTTP (axum + tokio) |

---

## Demo Gate 정책 (코드 변경 없음)

DB·MCP 레이어는 그대로 두고 다음 두 곳에만 명시:

1. **`.claude/rules/agent-demo-gate.md`** — 신규 룰 파일. CLAUDE.md 표에 등록.
2. **`.claude/agents/engram-worker.md`** — 신규 서브에이전트. description 에 강제 언어:
   > "이슈 상태를 `finished` 또는 `cancelled` 로 전환하지 마세요. Working → Demo 까지만 처리하고, 검증 결과를 `note_add` (type=context) 로 남긴 후 사용자에게 검토를 요청하세요. Finished 처리는 사람만 합니다."

칸반 UI 의 `demo` 컬럼은 시각적으로 강조 (노란 배경 + "검토 대기" 라벨) 하여 사용자가 놓치지 않게 한다.

`history.changed_by` 필드로 사후 감사 (audit) — MCP 호출은 `"agent"`, 데스크톱/CLI 호출은 `"user"` 로 기록.

---

## Status 전이 표 (기존)

`crates/engram-core/src/models/issue.rs::IssueStatus::can_transition_to` 가 이미 정의:

```
required → ready
ready    → working
working  → demo
working  → finished      ← Agent 가 호출하면 안 됨 (rule 로 차단)
demo     → finished      ← 사용자 전용 (rule 로 차단)
demo     → working       ← 재작업
어디서든 → cancelled
```

Agent 와 사용자 모두 같은 메서드를 쓰지만, **agent 는 finished 로의 전이를 호출하지 않음** 이 규약. 칸반 UI 는 사용자 입장에서 모든 전이를 자유롭게 허용한다.

---

## 칸반 UI 디자인 시안

### 메인 보드 — Confluence Cloud 스타일

```
┌─ Engram ─────────────────────────────────────────────────────────────────────┐
│ Sprint #1 · "JWT 전환 첫 주" · D-3                          🔍  ⚙   👤      │
├─ 필터 ────────────────────────────────────────────────────────────────────── │
│ Project: [xpert-da-web ▾]   Epic: [모두 ▾]   Priority: [⬤critical ⬤high]   │
│ ⚠ 미승인 draft 2건 · 스코프 팽창 감지 1건                       [상세보기] │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  REQUIRED 2     READY 1      WORKING 3       DEMO 1 ⚠         FINISHED 7   │
│ ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐  ┌──────────┐  │
│ │ ⬤ JWT 라 │  │ ⬤ 토큰   │  │ ⬤ 로그인 │  │ ⬤ 비번 검증 │  │ ⬤ ESLint │  │
│ │ 이브러리 │  │ 갱신 정책│  │ API 마이그│  │ 리팩터     │  │ 룰 추가  │  │
│ │ 검토     │  │          │  │ 레이션   │  │              │  │          │  │
│ │ #43      │  │ #41      │  │ #38      │  │ #36 [검토]  │  │ #29      │  │
│ │ 📎 epic  │  │ 📎 epic  │  │ ⏱ 2일   │  │ 🔔 검토대기 │  │          │  │
│ │ 🚫 1     │  │          │  │ ✓3/5    │  │ ✓5/5       │  │          │  │
│ └──────────┘  └──────────┘  └──────────┘  └──────────────┘  └──────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 디자인 토큰

| 요소 | 토큰 |
|------|------|
| 컬럼 헤더 텍스트 | `text-xs font-semibold uppercase tracking-wider text-slate-500` |
| 컬럼 배경 | `bg-slate-50` (Demo 만 `bg-amber-50 ring-1 ring-amber-200`) |
| 카드 | `bg-white rounded-md shadow-sm hover:shadow-md border border-slate-200` |
| Priority dot | critical=`bg-red-500`, high=`bg-orange-500`, medium=`bg-amber-400`, low=`bg-slate-400` |
| Scope expansion warning | `text-amber-600` + ⚠ |
| Demo column "검토대기" 배지 | `bg-amber-100 text-amber-900` |

### Issue Detail Drawer

```
┌──── #36 비번 검증 리팩터 ─────────────────────────── × ─┐
│ 상태 [demo ▾]    Priority [high ▾]    Epic 로그인     │
│ 목표 (goal): bcrypt v2 → v4 마이그레이션이 ...        │
│ 태스크 (5/5)                              ✓ ──────     │
│ ☑ 인증 분석              [agent_discovered]            │
│ ☑ 비번 비교 함수 분리                                  │
│ ...                                                    │
│ 노트                                                   │
│ ⚠ [caveat]  bcrypt v2 hash 와 v4 hash 동시 허용 필요   │
│ ★ [decision] new login 만 v4, legacy 는 lazy migration │
│ ✎ [context] (Agent 가 demo 직전 남긴 검토 가이드)     │
│ [Working 으로 되돌리기]      [완료로 표시 (Finished)]  │
└────────────────────────────────────────────────────────┘
```

`Finished` 버튼은 demo 상태에서만 활성. 누르면 `invoke('issue_set_status', { id, status: 'finished' })` 호출 + history 에 `changed_by='user'` 기록.

---

## Tray Widget 디자인 시안 (macOS 메뉴바)

```
┌─ ⌘  📦 12 · ⚠ 2  ────┐    ← 메뉴바 아이콘: 인박스 + 알림 뱃지
└────────────────────────┘
        │ 클릭
        ▼
┌────────────────────────────────────────┐
│ Engram                       Sprint #1 │
│ ──────────────────────────────────────│
│ 📋 xpert-da-web                        │
│ ▓▓▓▓▓▓░░░░ 6/10  · demo 1 · blocker 0│
│                                        │
│ 📋 doxus                               │
│ ▓▓▓▓░░░░░░ 4/12 · demo 0 · blocker 1 │
│ ──────────────────────────────────────│
│ 🔔 최근 알림                           │
│   • #36 demo 검토 대기 (4시간 전)      │
│   • #44 새 draft 이슈 (오늘)            │
│   • #38 blocker 발견 (어제)             │
│ ──────────────────────────────────────│
│ MCP Server                             │
│   ● Running · :3456 · 24 calls         │
│   [열기]  [재시작]  [정지]              │
│ ──────────────────────────────────────│
│ [보드 열기]   [세션 컨텍스트 보기]      │
│ [환경설정]                  [종료]      │
└────────────────────────────────────────┘
```

---

## MCP Manager UI 시안

```
┌─ Engram › MCP Server ───────────────────────────────────────────────────────┐
│ 상태  ● Running                          Uptime  00:18:23                    │
│ 포트  [3456    ]  Transport [http ▾]     Endpoint http://127.0.0.1:3456/mcp  │
│                                          [복사] [Claude Code 설정 보기]      │
│ ☑ 앱 시작 시 자동 기동                                                       │
│ ────────────────────────────────────────────────────────────────────────────│
│ [▶ 시작]  [■ 정지]  [↻ 재시작]                                              │
│ ────────────────────────────────────────────────────────────────────────────│
│ 최근 호출 (24)                                                              │
│  10:42:11  session_restore     ok  12ms                                     │
│  10:41:55  task_next            ok   3ms                                    │
│  10:41:54  issue_update(#36)    ok   8ms     status: working → demo         │
│  10:41:30  note_add(caveat)     ok   5ms                                    │
│ ────────────────────────────────────────────────────────────────────────────│
│ 로그 (tail)                                                                  │
│  INFO  engram_mcp::http   client connected: session_id=7                    │
│  INFO  engram_mcp::server initialize ok                                      │
│  WARN  engram_mcp::tools::issue  invalid status transition: finished→ready   │
└──────────────────────────────────────────────────────────────────────────────┘
```

Claude Code 설정 modal:

```jsonc
// ~/.claude.json → "mcpServers" 안에 붙여넣으세요 (Streamable HTTP)
"engram": {
  "type": "http",
  "url": "http://127.0.0.1:3456/mcp"
}
```

---

## Tauri Commands (Rust → JS)

`crates/engram-desktop/src/commands.rs`. 모두 `engram-core` 메서드 래퍼 + `McpSupervisor` 핸들.

| Command | 호출 메서드 | 반환 |
|---|---|---|
| `session_restore(project_key?)` | `Db::session_restore` | `SessionSnapshot` |
| `board_status(project_key?)` | `Db::board_status_query` | `BoardStatus` |
| `issue_list(filter)` | `Db::issue_list` | `Vec<Issue>` |
| `issue_get(id)` | `Db::issue_get` | `Issue` |
| `issue_set_status(id, status)` | `Db::issue_update` (`changed_by="user"`) | `Issue` |
| `issue_set_priority(id, p)` | `Db::issue_update` | `Issue` |
| `issue_create(input)` | `Db::issue_create` | `Issue` |
| `task_list(issue_id)` | `Db::task_list` | `Vec<Task>` |
| `task_set_status(id, status)` | `Db::task_update` | `Task` |
| `note_list(issue_id)` | `Db::note_list` | `Vec<NoteSummary>` |
| `note_get(id)` | `Db::note_get` | `Note` |
| `note_add(input)` | `Db::note_add` | `Note` |
| `note_resolve(id)` | `Db::note_resolve` | `()` |
| `epic_list(project_key?)` | `Db::epic_list` | `Vec<Epic>` |
| `sprint_current()` | `Db::sprint_current` | `Option<Sprint>` |
| `blocked_issues_graph(project_key)` | `Db::blocked_issues_graph` | `BlockingGraph` |
| `mcp_status()` | `McpSupervisor::status` | `SupervisorStatusSnapshot` |
| `mcp_start(port)` | `McpSupervisor::start` | `SupervisorStatusSnapshot` |
| `mcp_stop()` | `McpSupervisor::stop` | `SupervisorStatusSnapshot` |
| `mcp_restart(port)` | `McpSupervisor::restart` | `SupervisorStatusSnapshot` |
| `mcp_recent_calls()` | `McpSupervisor::recent_calls` | `Vec<CallRecord>` |
| `mcp_set_autostart(on)` | `settings::set_autostart` | `()` |

---

## Verification (전체 통합)

1. **빌드**
   ```bash
   cd crates/engram-desktop/ui && pnpm install && pnpm build
   cargo build -p engram-desktop --release
   ```
2. **개발 모드**
   ```bash
   pnpm --filter engram-desktop-ui dev   # Vite dev server
   cargo tauri dev -p engram-desktop     # Tauri shell + watch
   ```
3. **End-to-end**
   - 새 sprint/epic/issue 를 CLI 로 만들고 데스크톱에서 카드 표시 확인
   - 카드를 `working` → `demo` 로 드래그 → DB `issues.status='demo'`, `history.changed_by='user'`
   - `demo` 카드 클릭 → Drawer → `Finished` 클릭 → 검증
   - 잘못된 전이 → toast 표시 + 카드 복귀
4. **트레이 위젯**
   - CLI 로 `engram issue create` → 5초 이내 macOS 알림
   - 상태 변경 → 메뉴바 뱃지 카운트 갱신
5. **임베디드 MCP 서버 (Streamable HTTP)**
   - `curl -X POST http://127.0.0.1:3456/mcp -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'` → 200 + `Mcp-Session-Id` 헤더
   - 시작/정지/재시작 토글 + 포트 변경
   - WAL 동시성: 데스크톱 카드 이동(쓰기) + HTTP `task_next`(읽기) 동시 → 둘 다 성공
   - History 감사: `SELECT changed_by FROM history WHERE entity_id=? ORDER BY id DESC LIMIT 1` 로 actor 검증
6. **회귀**
   - `cargo test --workspace` 기존 36건 + 신규 데스크톱/HTTP 테스트 모두 green

---

## Critical Files Already in Place (재사용)

| 기능 | 파일 |
|---|---|
| 상태 전이 검증 | `crates/engram-core/src/models/issue.rs::IssueStatus::can_transition_to` |
| 보드 집계 + blocked chains | `crates/engram-core/src/repository/session.rs::board_status_query` |
| 세션 복원 + 경고 | `crates/engram-core/src/repository/session.rs::session_restore` |
| 블로킹 그래프 BFS | `crates/engram-core/src/repository/blocking.rs::blocked_issues_graph` |
| 회고 리포트 | `crates/engram-core/src/repository/retro.rs::retro_report` |
| 태스크 fractional ord | `.claude/rules/fractional-index.md` 절차 그대로 |
| 마이그레이션 패턴 | `.claude/rules/schema-evolution.md` |

신규 도메인 로직은 거의 없음 — 데스크톱은 **순수 UI 레이어** 로 그치는 것이 이 계획의 핵심.

---

## 마일스톤 분할

| ID | 제목 | 기간 | 문서 |
|---|---|---|---|
| ✅ M0 | 선행 정비 (lib 분리, HTTP, changed_by, pool) | 완료 | [m0-foundations.md](./m0-foundations.md) |
| ✅ M1 | 스캐폴딩 + 보드 읽기 | 완료 | [m1-scaffold-board.md](./m1-scaffold-board.md) |
| ✅ M2 | DnD + Drawer + Finished 필터 | 완료 | [m2-dnd-drawer.md](./m2-dnd-drawer.md) |
| ✅ M3 | 임베디드 MCP Supervisor | 완료 | [m3-mcp-supervisor.md](./m3-mcp-supervisor.md) |
| ✅ M4 | 메뉴바 트레이 + 알림 | 완료 | [m4-tray-notifications.md](./m4-tray-notifications.md) |
| ✅ M5 | 폴리시 (필터/그래프/ADR/규칙) | 완료 | [m5-polish.md](./m5-polish.md) |

**총 예상**: 4.5~5주.
