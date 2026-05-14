# Engram

Agent Issue Management System — Sprint / Epic / Issue / Task / Note 를 SQLite에 저장하고
**MCP 서버 (stdio JSON-RPC) + CLI** 로 노출한다. 향후 Tauri Desktop 추가 예정.

- 설계: `doxus://brain/Ideas/agent/Engram - Agent Issue Management System.md`
- 구현 계획: `doxus://brain/Ideas/agent/Engram - Implementation Plan.md`

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

## 핵심 원칙 (ADR 요약)

| # | 결정 | 문서 |
|---|------|------|
| 1 | 단일 중앙 DB `~/.engram/engram.db` — `epics.project_key` 컬럼으로 프로젝트 분리 | `docs/adr/0001-single-central-db.md` |
| 2 | SQLite + WAL + `busy_timeout=5000`, PostgreSQL 미사용 | `docs/adr/0002-sqlite-wal.md` |
| 3 | `issue_links` 는 `blocks` 단방향만 저장, 역방향은 쿼리로 도출 | `docs/adr/0003-blocks-one-way.md` |
| 4 | Claude Code Hook 통합은 MVP(Phase 1) 필수 | `docs/adr/0004-hook-in-mvp.md` |
| 5 | `tasks.ord` (REAL, fractional index) — `order` 예약어 회피 | `docs/adr/0005-fractional-ord.md` |

## 개발 명령

```bash
cargo build                                # 전체 빌드
cargo test                                 # 전체 테스트
cargo test -p engram-core                  # 특정 크레이트 테스트
cargo run -p engram-cli -- sprint list     # CLI 실행
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

새 규칙이 필요하다고 판단되면 `.claude/rules/<slug>.md` 로 추가하고 이 표에 등록한다.

## 현재 진행 상황 요약

- ✅ Phase 1 코어: models / repository / migrations / MCP tools 18개 / CLI 7 서브커맨드
- ❌ tests/ 디렉터리 비어 있음 — `.claude/rules/testing-strategy.md` 기준으로 통합 테스트 추가 필요
- ❌ Hook 통합 (`engram hook install`) 구현 미검증 — `crates/engram-cli/src/commands/hook.rs` 확인 필요
