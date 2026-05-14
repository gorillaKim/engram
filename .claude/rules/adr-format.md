# Rule: ADR (Architecture Decision Record) 작성

## 위치 / 네이밍

- `docs/adr/NNNN-<kebab-slug>.md`
- 번호는 zero-pad 4자리, 절대 재사용 / 재번호 금지
- 슬러그는 결정의 핵심 키워드 (예: `0001-single-central-db.md`, `0003-blocks-one-way.md`)

## 본문 포맷 (고정 섹션)

```markdown
# ADR-NNNN: <Title>

## Status
Accepted | Proposed | Superseded by ADR-XXXX | Deprecated

## Context
<이 결정이 필요한 배경. 1~3 단락.>

## Decision
<무엇을 결정했는가. 1~2 단락, 명령형.>

## Consequences
<이 결정이 만드는 긍정/부정 결과. 불릿 OK.>

## Trade-offs (선택)
<다른 선택지와 비교, 왜 다른 안을 안 했는지.>
```

프론트매터 / 메타데이터 / 작성자 / 날짜 헤더 추가하지 말 것 (git 기록으로 충분).

## 결정 변경 / 폐기

- 기존 결정을 뒤집을 때는 **새 ADR을 작성**한다 (원본 수정 금지).
- 원본 ADR 의 `## Status` 만 `Superseded by ADR-XXXX` 로 갱신.
- 폐기되었지만 대체 안 없으면 `Deprecated`.

## ADR 가 아닌 것

다음은 ADR 로 만들지 말고 다른 곳에 둔다:

| 내용 | 위치 |
|------|------|
| sqlx / MCP / 테스트 컨벤션 | `.claude/rules/` |
| 도구 추가 절차, 디렉터리 구조 | `CLAUDE.md` or `.claude/rules/` |
| 일회성 버그 수정 메모 | git 커밋 메시지 |
| 진행 중인 작업 / TODO | issue 트래커 (Engram 자기 자신) |

## 분량

- 1페이지 이하 (스크롤 없이 보이는 정도).
- Context 가 길어지면 **다른 문서로 링크** 하고 ADR 은 결정만 남긴다.

## 예시 (Phase 1 의 ADR 5개)

| 번호 | 제목 |
|------|------|
| 0001 | Single Central DB vs Per-project DB |
| 0002 | SQLite + WAL vs PostgreSQL |
| 0003 | `blocks` 단방향 저장 |
| 0004 | Claude Code Hook 을 MVP 에 포함 |
| 0005 | `tasks.ord` (REAL fractional index) |

본문은 [[Engram - Implementation Plan]] §7 에서 옮긴다 — 이 작업은 별도 진행.
