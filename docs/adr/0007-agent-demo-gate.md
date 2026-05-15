# ADR-0007: Agent Demo Gate — 코드 강제 없이 규칙·UI 어포던스로 구현

## Status
Accepted

## Context

Engram 의 이슈 상태 흐름에서 `finished` 와 `cancelled` 는 사용자만 결정해야 한다. Agent 가 실수로 이슈를 `finished` 처리하면 사용자의 검토 기회가 사라진다. 이를 방지하기 위해 얼마나 강하게 강제할지 결정이 필요했다.

## Decision

코드 수준 강제(MCP 서버에서 `finished`/`cancelled` 호출을 거부) 없이 다음 세 가지 레이어로 구현한다:

1. **`.claude/rules/agent-demo-gate.md`** — 에이전트 행동 규칙 (프롬프트 수준)
2. **`.claude/agents/engram-worker.md`** — 서브에이전트 정의에 금지 사항 명시
3. **`history.changed_by`** 감사 — `agent` 로 기록된 상태 전이를 사후 탐지

데스크톱 UI 에서는 demo 컬럼 시각적 강조 + `Finished` 버튼을 demo 상태에서만 표시하여 사용자 실수도 방지한다.

## Consequences

- 구현 단순: 서버 사이드 검증 로직 없음
- 단일 사용자 환경에서 코드 게이트는 과공학 — 사용자가 직접 MCP 를 호출하면 어차피 우회 가능
- 위반 시 즉각 되돌리기 가능 (칸반 DnD 로 상태 복구)
- `history.changed_by='agent' AND new_value='finished'` 쿼리로 사후 감사 가능

## Trade-offs

코드 강제를 원할 경우: MCP 서버의 `issue_update` 핸들러에 `if args["status"] == "finished" && changed_by == "agent" { return Err(...) }` 추가. 단, 이는 MCP 클라이언트의 능동적 협조(changed_by 전달)를 전제하며, 현재 단계에서는 과잉 복잡성이다.
