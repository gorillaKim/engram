# ADR-0012: Mission 레이어 도입 (Sprint → Mission → Epic 계층)

## Status
Superseded in part by ADR-0013 (mission-sprint coupling)

## Context
Phase 1~2에서 Sprint → Epic → Issue → Task 4단계 계층으로 이슈를 관리했으나,
에이전트와 사용자가 Jira 스타일의 "어떤 출시 목표에 속하는가"를 표현할 수단이 없었다.
여러 에픽(다른 project_key 포함)을 하나의 목표로 묶는 Mission 레이어가 필요해졌다.

## Decision
Sprint 하위에 Mission 레이어를 추가한다. 계층은 Sprint → Mission → Epic → Issue → Task.

- `missions` 테이블: `sprint_id` (nullable, NULL=백로그), `title`, `description`, `status`, `jira_key`
- `epics.mission_id` (required), `issues.mission_id` (epic에서 자동 상속)
- Mission은 cross-project: 하나의 Mission이 서로 다른 `project_key`의 Epic을 가질 수 있음
- Mission 상태 전이(`completed`/`cancelled`)는 사용자 전용 — 에이전트 호출 금지

## Consequences
- 에이전트가 `session_restore` 한 번에 미션별 진척도(`progress_rate`) 파악 가능
- `epic_create` 시 `mission_id` 필수 → 기존 API 호환성 변경 필요
- 0008 마이그레이션으로 기존 Epic/Issue에 `project_key`별 placeholder mission 자동 생성/백필
- `epics.sprint_id` 컬럼 제거 (Mission이 sprint 연결을 담당)

## Trade-offs
- Mission 없이 Epic만으로도 그루핑 가능하나, Jira 연동(`jira_key`) 및 cross-project 묶음을 위해 별도 레이어 채택
- Per-project Mission(`missions.project_key` 컬럼 추가) 검토했으나 cross-project 출시 목표 표현을 위해 미채택
- `missions.project_key` 없이도 `epic.project_key`로 프로젝트 분리 유지됨
