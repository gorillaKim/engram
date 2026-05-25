# ADR-0014: Epic-Sprint SSOT (Mission 분리)

## Status
Accepted (supersedes ADR-0013)

## Context
ADR-0012 와 ADR-0013 은 `mission.sprint_id` 를 sprint 소속의 SSOT 로 두어 Mission 을 sprint 시간 박스 안에서 완수되는 단위로 정의했다. 그러나 실제 제품 개발에서 Mission(이니셔티브)은 분기·연 단위로 여러 sprint 를 가로지르는 전략 목표인 경우가 많고, "한 미션 = 한 sprint" 라는 묵시적 제약은 자연스럽지 않다. 동일 미션을 둘 이상의 sprint 에 분산해 진행하려면 미션을 인위적으로 분할하거나 sprint 종료 직전 이월 미션을 새로 만드는 우회가 필요했다.

Mission 개념이 본격적으로 운영에 쓰이기 전 (ADR-0013 이 머지된 직후) 인 지금이 SSOT 재설계 비용이 가장 낮은 시점이다.

## Decision
Sprint 소속 SSOT 를 **Mission 에서 Epic 으로 이관**한다.

- `missions.sprint_id` 컬럼 제거. Mission 은 sprint-agnostic 한 thematic/전략 그룹.
- `epics.sprint_id` 컬럼 신설 (`REFERENCES sprints(id) ON DELETE SET NULL`). Epic 이 실제 SSOT.
- `issues.mission_id` 컬럼 제거. Issue 는 부모 Epic 을 통해서만 mission 에 귀속.
- `issues.sprint_id` 컬럼은 이미 ADR-0013 에서 제거된 상태 유지. 응답 호환성을 위해 Rust `Issue` 모델 필드는 보존하고 `JOIN epics e ON i.epic_id = e.id` 로 derive.
- 마이그레이션 `0011_epic_owns_sprint.sql` 가 백필:
  - `epic.sprint_id = (SELECT sprint_id FROM missions WHERE missions.id = epics.mission_id)`.
  - 이후 `missions.sprint_id` / `issues.mission_id` 컬럼 DROP.
- API 변경:
  - 제거: `mission_set_sprint`, `issue_set_sprint`.
  - 신규: `epic_set_sprint(epic_id, sprint_id, agent_id)`.
  - `mission_create` / `mission_update` 에서 `sprint_id` 입력 제거.
  - `epic_create` / `epic_update` 에 `sprint_id` 추가 (`update_sprint_id=true` 일 때만 적용).
  - `issue_create` / `issue_update` 에서 `mission_id` 입력 제거. Issue 가 다른 mission 에 속하려면 epic 을 옮겨야 함.

## Consequences
- **긍정적 효과**:
  - 한 Mission 을 여러 sprint 에 분산해 진행하는 장기 이니셔티브를 자연스럽게 표현.
  - Sprint 이월 시 mission 을 쪼개거나 placeholder mission 을 만드는 우회가 사라짐.
  - "Epic 을 다른 sprint 로 옮긴다" 가 1급 시민 — 단일 `epic_set_sprint` 호출로 산하 이슈가 함께 이동 (`JOIN` 으로 derive 되기 때문에 cascade 별도 필요 없음).
- **부정적 효과**:
  - ADR-0013 직후 SSOT 재배치 → 마이그레이션·API breaking change.
  - 서브에이전트·Desktop UI 의 `mission_set_sprint` / `issue_set_sprint` 호출처가 모두 `epic_set_sprint` 로 이동해야 함.
  - "이번 스프린트에 어떤 mission 이 살아있는가?" 같은 뷰는 `JOIN epics e ON e.mission_id = m.id WHERE e.sprint_id = ?` 처럼 한 단계 더 들어가야 함.

## Trade-offs
- **Option 유지 (ADR-0013 그대로)**: 변경 비용 0, 그러나 multi-sprint mission 표현 불가 — 핵심 도메인 모델이 실제 사용 패턴과 어긋남.
- **Option A — Mission 에 sprint 다대다 매핑 테이블**: 더 유연하나 칸반/필터 쿼리가 복잡해지고 "현재 sprint 의 mission 이 무엇이냐" 가 모호해진다.
- **Option B (채택) — Epic 이 SSOT**: 자연스러운 실행 단위 매핑. Mission 은 순수 thematic grouping 으로 축소. Epic 단위 sprint 이동이 1급 시민. 마이그레이션 비용은 한 번, 향후 모델 직관성은 영구.
