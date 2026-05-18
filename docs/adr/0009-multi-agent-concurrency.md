# ADR-0009: Multi-Agent Concurrency — `assigned_agent` 컬럼 + CAS Claim/Release + agent_id 1급 시민화

## Status
Accepted

## Context

ADR-0007 은 "단일 사용자 환경" 을 전제로 demo gate 의 코드 강제 없이 규칙·UI 어포던스로만 막기로 결정했다. 그러나 멀티 에이전트(N≥2) 시나리오에서는 다음 race 가 실제로 발생한다:

- 두 에이전트가 동시에 같은 이슈를 `issue_update(status='working')` 호출 → 둘 다 성공, 작업 중복
- `task_next` 가 read-only 라서 두 에이전트가 동일 task 를 잡을 가능성
- `history.changed_by` 가 모든 MCP 호출에서 `"agent"` 로 하드코딩되어 어느 에이전트가 무엇을 했는지 감사 불가

ADR-0007 의 단일 사용자 가정은 멀티 에이전트에서 무너진다 — 이 ADR 은 그 가정을 깨는 보안/일관성 인프라를 도입한다.

## Decision

세 가지 변경을 한 번에 도입한다:

### 1. `issues.assigned_agent TEXT NULL` 컬럼 추가 (migration 0005)

- `NULL` = 점유되지 않음, `'<agent>'` = 해당 에이전트가 working 으로 점유 중
- 이슈가 working 을 벗어나면 (`issue_update`, `issue_release`) 자동 `NULL` 로 비워진다

### 2. CAS(Compare-And-Set) 기반 `issue_claim` / `issue_release`

- `issue_claim(id, agent_id)` — 한 SQL 의 `UPDATE ... WHERE status IN ('ready','working') AND (assigned_agent IS NULL OR assigned_agent = ?)` 으로 race 차단. `rows_affected=0` 이면 다른 에이전트가 잡은 것 → `Error::Validation`
- `issue_release(id, transition_to, agent_id)` — 자기가 잡은 이슈만 해제 가능, `assigned_agent=NULL` + 지정 상태로 전이
- 동일 `agent_id` 의 재호출은 idempotent (already-held → OK)
- TTL 기반 자동 ready 환원(lease 만료) 은 **이번 ADR 범위에 없음** — N≥3 카오스가 실제로 발생하면 별도 ADR 로 결정

### 3. `agent_id` 1급 시민화

- 모든 변경 API (`issue_update`, `task_update`, `epic_update`, `sprint_update`, `note_resolve`, `issue_set_sprint`, `issue_delete`, `epic_delete`) 의 MCP inputSchema 에 `agent_id` 필드 추가
- MCP 핸들러는 `args["agent_id"].as_str().unwrap_or("agent")` 로 받아 repo 의 `changed_by: &str` 파라미터로 전달
- `history.changed_by` 는 자유 텍스트 (예: `'user'`, `'claude-opus@sess-abc'`) — 형식 강제 없음. ADR-0007 의 후속 작업으로 이미 결정되었던 사항을 이번에 끝까지 구현.

### `session_restore` 확장

- 응답에 `active_workers: Vec<{ issue_id, issue_title, agent_id, project_key, since }>` 추가
- 리더 에이전트가 spawn 결정 / WIP 확인 / 정체 감지에 활용

## Consequences

긍정:
- 멀티 에이전트 working 전이가 race-free
- 누가 무엇을 점유 중인지 단일 쿼리로 가시화 (active_workers)
- `history.changed_by` 가 의미 있는 식별자가 되어 멀티 에이전트 감사가 가능
- 자유로운 칸반 DnD 도 그대로 작동 — `issue_update` 가 working 을 벗어날 때 `assigned_agent` 를 자동 정리

부정:
- lease 만료 자동 ready 환원이 없으므로, 에이전트가 죽으면 `assigned_agent` 가 영구 남는다. 사용자 / 다른 에이전트의 수동 release 필요. (`stalled_issues` 가 이미 이 케이스를 탐지하므로 운영상 보완 가능)
- 칸반 UI 에서 사용자가 카드를 working 으로 끌어왔을 때 `assigned_agent` 가 비어 있는 상태로 들어간다 — 에이전트가 다음 `issue_claim` 호출에서 잡으면 됨

## Trade-offs

대안 A: **lease + renew + release 풀스택**. TTL 만료 시 자동 ready 환원. 더 견고하지만 schema/도구/타이머 인프라 필요. **이번에는 도입하지 않음** — CAS 한 줄로 race 의 90% 가 해결되며 N≥3 카오스가 관찰되기 전까지 over-engineering.

대안 B: **`issue_update(status='working')` 자체를 거부하고 `issue_claim` 만 허용**. 코드 강제 수준. 호출자(에이전트, 사용자, 데스크톱 UI) 가 모두 이 새 도구로 마이그레이션해야 하므로 호환성 부담 큼. 이번 ADR 은 `issue_update` 도 그대로 두되 working 을 벗어날 때 `assigned_agent` 를 자동 정리하는 보완으로 충돌을 피한다.

대안 C: **ADR-0007 의 demo gate 코드 강제 동시 도입**. `task_evidence_attach` (#9) 및 `demo_gate_check` 정책이 선행되어야 의미 있다 — 별도 ADR-0009 에서 다룬다.

## 관련 변경

- `crates/engram-core/migrations/0005_issue_assigned_agent.sql`
- `crates/engram-core/src/models/issue.rs::Issue.assigned_agent`
- `crates/engram-core/src/repository/issue.rs::issue_claim, issue_release`
- `crates/engram-core/src/repository/session.rs::ActiveWorker, SessionSnapshot.active_workers`
- `crates/engram-mcp/src/tools/issue.rs::claim, release` (+ `issue_claim`, `issue_release` tool definitions)
- 모든 MCP `*_update` 핸들러: `agent_id` 파라미터 수용
