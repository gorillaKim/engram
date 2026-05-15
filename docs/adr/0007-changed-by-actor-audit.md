# ADR-0007: changed_by 파라미터로 행위자 감사(Audit Trail)

## Status
Accepted

## Context
Demo Gate 정책의 핵심 fallback은 `history.changed_by` 컬럼을 통한 사후 감사다. 기존 구현은 모든 `*_update` / `note_resolve` 메서드 내부에서 `changed_by`를 `"agent"`로 하드코딩하고 있어, 데스크톱 앱(사용자)이 직접 호출해도 `"agent"`로 기록되었다. 이 상태에서는 누가 `finished` 전이를 시켰는지 audit trail로 구분할 수 없다.

## Decision
`issue_update`, `task_update`, `epic_update`, `sprint_update`, `note_resolve` 다섯 메서드에 `changed_by: &str` 파라미터를 추가한다. 호출처가 actor를 명시적으로 전달한다:
- MCP 도구 (Agent 경로) → `"agent"`
- CLI 커맨드 (사용자 직접 실행) → `"user"`
- 데스크톱 앱 Tauri command (M1~) → `"user"`

## Consequences
- DB 레이어 변경 없이 `history.changed_by`로 agent/user 전이를 구분 가능
- 기존 테스트 및 호출처 전체에 파라미터 추가 필요 (일회성 마이그레이션)
- `test_history_records_changed_by_actor` 통합 테스트로 회귀 방지
