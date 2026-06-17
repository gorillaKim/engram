# Architecture Decision Records

이 디렉터리는 Engram 의 주요 설계 결정을 기록한다.

## 목록

| 번호 | 제목 | 상태 |
|------|------|------|
| 0001 | Single Central DB vs Per-project DB | Accepted |
| 0002 | SQLite + WAL vs PostgreSQL | Accepted |
| 0003 | `blocks` 단방향 저장 | Accepted |
| 0004 | Claude Code Hook 을 MVP 에 포함 | Accepted |
| 0005 | `tasks.ord` (REAL fractional index) | Accepted |
| 0006 | Desktop: Tauri v2 단일 바이너리 | Accepted |
| 0006 | Streamable HTTP Transport | Accepted *(번호 중복, 별도 정리 예정)* |
| 0007 | Agent Demo Gate | Accepted |
| 0007 | `history.changed_by` actor 감사 | Accepted *(번호 중복, 별도 정리 예정)* |
| 0008 | Embedded MCP Supervisor | Accepted |
| 0009 | Multi-Agent Concurrency | Accepted |
| 0010 | CLI ↔ MCP 패리티 컨벤션 | Accepted |
| 0011 | CLI 배포 경로 (cargo install + GitHub Releases) | Accepted |
| 0012 | Mission 계층 구조 도입 | Accepted |
| 0013 | Mission-Sprint 간 SSOT 단일화 | Accepted |
| 0014 | Epic-Sprint 간 SSOT 단일화 | Accepted |
| 0015 | SSE 및 미확정 프로토콜 계약의 단일 정정 지점 설계 | Accepted |

## 작성 규칙

`.claude/rules/adr-format.md` 참조.

요약:
- 파일명: `NNNN-<kebab-slug>.md`
- 섹션: `Status` / `Context` / `Decision` / `Consequences` / `Trade-offs(선택)`
- 기존 ADR 수정 금지 — 결정을 뒤집을 때는 새 번호 + 원본 Status 를 `Superseded by ADR-XXXX` 로 갱신
