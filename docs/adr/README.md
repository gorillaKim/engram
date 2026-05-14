# Architecture Decision Records

이 디렉터리는 Engram 의 주요 설계 결정을 기록한다.

## 목록

(아직 비어 있음. [[Engram - Implementation Plan]] §7 의 ADR-001~005 본문을 옮길 예정.)

| 번호 | 제목 | 상태 |
|------|------|------|
| 0001 | Single Central DB vs Per-project DB | (작성 예정) |
| 0002 | SQLite + WAL vs PostgreSQL | (작성 예정) |
| 0003 | `blocks` 단방향 저장 | (작성 예정) |
| 0004 | Claude Code Hook 을 MVP 에 포함 | (작성 예정) |
| 0005 | `tasks.ord` (REAL fractional index) | (작성 예정) |

## 작성 규칙

`.claude/rules/adr-format.md` 참조.

요약:
- 파일명: `NNNN-<kebab-slug>.md`
- 섹션: `Status` / `Context` / `Decision` / `Consequences` / `Trade-offs(선택)`
- 기존 ADR 수정 금지 — 결정을 뒤집을 때는 새 번호 + 원본 Status 를 `Superseded by ADR-XXXX` 로 갱신
