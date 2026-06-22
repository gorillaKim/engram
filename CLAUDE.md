# Engram

Agent Issue Management System — Sprint / Epic / Issue / Task / Note 를 SQLite에 저장하고
**MCP 서버 (stdio JSON-RPC) + CLI** 로 노출한다. 향후 Tauri Desktop 추가 예정.

- 설계: `doxus://brain/Ideas/agent/Engram - Agent Issue Management System.md`
- 구현 계획: `doxus://brain/Ideas/agent/Engram - Implementation Plan.md`
- 에이전트 플레이북: [agent-playbook.md](file:///Users/madup/gorillaProject/engram/docs/agent-playbook.md)


## Workspace 구조

```
engram/
├── crates/
│   ├── engram-core/   ← 도메인 모델 + sqlx SQLite repository (의존성 0)
│   ├── engram-mcp/    ← JSON-RPC stdio MCP 서버 (engram-core 사용)
│   └── engram-cli/    ← clap 기반 CLI + Claude Code Hook 통합
├── migrations/        ← engram-core/migrations/NNNN_*.sql (sqlx-migrate 내장)
├── docs/adr/          ← 설계 결정 기록 (Architecture Decision Records)
└── .claude/rules/     ← 작업 시 참조할 코딩 규칙
```

**의존성 방향은 한방향**: `engram-cli`, `engram-mcp` → `engram-core`.
`engram-core` 가 MCP/CLI 의 타입을 import 하는 일은 **금지**.
`engram-mcp`는 6개의 mission 도구를 포함하여 57개 MCP 도구를 제공하고, `engram-cli`는 15개의 서브커맨드를 제공해 모두 `engram-core`를 참조합니다.

Phase 3 Desktop: `crates/engram-desktop/` — Tauri v2 앱. `engram-core`, `engram-mcp` 만 참조.

## 핵심 원칙 (ADR 요약)

| # | 결정 | 문서 |
|---|------|------|
| 1 | 단일 중앙 DB `~/.engram/engram.db` — `epics.project_key` 컬럼으로 프로젝트 분리 | `docs/adr/0001-single-central-db.md` |
| 2 | SQLite + WAL + `busy_timeout=5000`, PostgreSQL 미사용 | `docs/adr/0002-sqlite-wal.md` |
| 3 | `issue_links` 는 `blocks` 단방향만 저장, 역방향은 쿼리로 도출 | `docs/adr/0003-blocks-one-way.md` |
| 4 | Claude Code Hook 통합은 MVP(Phase 1) 필수 | `docs/adr/0004-hook-in-mvp.md` |
| 5 | `tasks.ord` (REAL, fractional index) — `order` 예약어 회피 | `docs/adr/0005-fractional-ord.md` |
| 6 | Desktop: Tauri v2 + React + Tailwind, 단일 바이너리 | `docs/adr/0006-desktop-tauri.md` |
| 7 | Agent Demo Gate — 규칙+UI 어포던스, 코드 강제 없음 | `docs/adr/0007-agent-demo-gate.md` |
| 10 | CLI ↔ MCP 패리티 컨벤션 (`engram <area> <verb>`, `--json`, exit code 0/2/3/4) | `docs/adr/0010-cli-mcp-parity.md` |
| 11 | CLI 배포 — `cargo install` 1차 + GitHub Releases prebuilt 2차 | `docs/adr/0011-cli-distribution.md` |
| 12 | Mission 레이어 도입 — Sprint→Mission→Epic 계층, cross-project, 0008 마이그레이션 | `docs/adr/0012-mission-layer.md` |
| 13 | Mission.sprint_id SSOT 단일화 — issues/epics.sprint_id 컬럼 drop (Option C) | `docs/adr/0013-mission-sprint-ssot.md` |
| 14 | Epic-Sprint SSOT — Mission 은 sprint-agnostic 으로 전환, Epic 이 sprint 보유 (supersedes ADR-0013) | `docs/adr/0014-epic-sprint-ssot.md` |


## 개발 명령

```bash
cargo build                                # 전체 빌드
cargo test                                 # 전체 테스트
cargo test -p engram-core                  # 특정 크레이트 테스트
cargo run -p engram-cli -- sprint list     # CLI 실행 (사람용)
cargo run -p engram-cli -- sprint list --json   # 머신 파싱용 JSON
echo '<json>' | cargo run -p engram-mcp    # MCP stdio 수동 시험
```

마이그레이션은 `sqlx::migrate!("./migrations").run(&pool)` 가 `Db::open` 안에서 자동 실행한다. 별도 CLI 호출 불필요.

## 작업 시 규칙

코드를 추가하기 전에 **해당 영역의 규칙 파일을 먼저 읽는다**:

| 작업 | 참조할 규칙 |
|------|------------|
| DB 쿼리 / Repository 추가 | `.claude/rules/sqlx-pattern.md` |
| 새 마이그레이션 추가 | `.claude/rules/schema-evolution.md` |
| 새 MCP 도구 추가 | `.claude/rules/mcp-tool-shape.md` |
| 테스트 작성 | `.claude/rules/testing-strategy.md` |
| 설계 결정 추가 / 변경 | `.claude/rules/adr-format.md` |
| `tasks.ord` 조작 | `.claude/rules/fractional-index.md` |
| Demo→Finished 전이 / Agent 상태 전이 제한 | `.claude/rules/agent-demo-gate.md` |

새 규칙이 필요하다고 판단되면 `.claude/rules/<slug>.md` 로 추가하고 이 표에 등록한다.

## 현재 진행 상황 요약

- ✅ Phase 1 코어: models / repository / migrations / **MCP tools 57개 ↔ CLI 15 서브커맨드 (1:1 패리티, ADR-0010)**
- ✅ Mission 레이어 도입 (ADR-0012) — Sprint→Mission→Epic 계층 구조로 cross-project 미션 추적 및 mission_* 도구 7개 추가
- ✅ CLI ↔ MCP 동치 통합 테스트 `crates/engram-cli/tests/parity_test.rs` 15건 — read-only 9 + 변경 도구 6. 회귀 방지 자동화.
- ✅ 통합 테스트 `crates/engram-core/tests/workflow_test.rs` 7건 (full_sprint / blocked_by / fractional_ord / session_filter / task_next_priority / cross_project_blocking / scope_expansion)
- ✅ MCP dispatch round-trip 테스트 `crates/engram-mcp/src/tools/dispatch_test.rs` 8건 — `.claude/rules/mcp-tool-shape.md` 준수
- ✅ CLI clap 파싱 테스트 (sprint / epic / issue / task / note / hook / board / blocked / stalled / history) 40+건
- ✅ CLI 글로벌 `--json` flag + `OutputFormat` 인프라 (`crates/engram-cli/src/output.rs`) + exit code 매핑 (0/1/2/3/4)
- ✅ CLI 패리티 매트릭스 문서 `docs/cli-mcp-parity.md` (45 도구 ↔ verb 매핑 SSOT)
- ✅ CLI 배포: `cargo install --path crates/engram-cli` + `.github/workflows/release.yml` (macOS arm64/x64 + linux x64 prebuilt)
- ✅ Hook 통합 — `engram hook install / uninstall / post-session-check` 동작 검증됨
- ✅ Phase 2 선행 구현: `my_blocked_issues` (BFS + 사이클), 스코프 팽창 감지, `engram retro` 리포트, SSE transport
- ✅ Phase 3 Desktop (M0~M5): Tauri v2 칸반보드, Drawer, MCP Supervisor, 트레이+알림, 필터, BlockingGraph
- ✅ 에이전트 친화성 개선 (Mission #11): note_id→id 통일, SessionStart hook 수정, Demo Gate 코드 강제, Error::Conflict (exit 4), dead code 제거, Plan 문서 상태 enum 동기화
- ✅ `session_restore(compact=true)` — N+1→벌크 COUNT 쿼리, 페이로드 70%+ 감소, `engram session restore --compact` CLI 패리티
- ✅ 삭제 계열 MCP 응답 통일 — `issue_unlink` 응답에 `deleted_id` 추가, 전 도구 `{ ok: true, deleted_id: N }` 준수
- 📊 `cargo test --workspace`: **263 passed / 0 failed** (기존 258 + P2 compact/delete tests 5)

### 알려진 한계

- `note_type` 의 custom_type 컬럼은 미도입 (Phase 2 잔여 항목).

## 서브에이전트 / 외부 호출 (CLI fallback)

플러그인의 서브에이전트가 MCP 도구를 직접 못 받는 환경에서는 동일한 동작을 셸 호출로 수행한다:

```bash
engram session restore --project myproj --json
engram issue claim 12 --agent-id "$AGENT_ID" --json
engram issue release 12 --agent-id "$AGENT_ID" --transition-to demo --json
engram note add --issue 12 --type context --summary "..." --agent-id "$AGENT_ID" --json
engram board status --project myproj --json
engram stalled --threshold-minutes 10 --json
engram history by-agent --agent-id "$AGENT_ID" --limit 20 --json
```

전체 매핑은 `docs/cli-mcp-parity.md`, 서브에이전트 setup 은 `docs/plugin-setup.md` 참조.
CLI ↔ MCP 동치성은 `crates/engram-cli/tests/parity_test.rs` 가 자동 회귀 방지.
