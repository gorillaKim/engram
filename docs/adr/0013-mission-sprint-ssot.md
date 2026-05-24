# ADR-0013: Mission-Sprint SSOT

## Status
Accepted

## Context
ADR-0012(Mission 레이어 도입) 이후, `sprint_id` 컬럼이 `missions`, `epics`, `issues` 세 곳에 중복하여 존재하게 되었습니다. 
이로 인해 `issue_create` 호출 시 부모 epic으로부터 `mission_id`는 자동 상속되지만 `sprint_id`는 상속되지 않고 개별 설정되어 누락되거나, `board_issues_query` 등이 `mission.sprint_id` 대신 `issue.sprint_id`만을 필터링하여 특정 미션 산하의 이슈들이 칸반 보드에 노출되지 않는 등의 정합성 문제가 발생하였습니다.
스프린트와 미션이 강하게 결합(coupling)되어 가기로 한 설계 의도에 맞게, 스프린트 소속 여부를 판단하는 단일 진실 원천(SSOT)을 명확히 정의할 필요가 있습니다.

## Decision
`mission.sprint_id`를 스프린트 소속 결정의 단일 진실 원천(SSOT)으로 정의하고, `issues.sprint_id` 및 `epics.sprint_id` 컬럼은 데이터베이스 스키마에서 단계적으로 제거(Option C)합니다.
- **Read 경로**: 모든 스프린트 기반 조회 및 필터링은 `JOIN missions ON i.mission_id = m.id WHERE m.sprint_id = ?`를 기준으로 통일합니다.
- **Write 경로**: `issue_create`에서 `sprint_id` 입력을 차단하고(ValidationError 반환), `issue_set_sprint` API 및 CLI 명령을 Deprecated로 전환하여 호출 시 거부합니다.
- **응답 호환성**: 외부 호출자와 Desktop UI의 호환성을 위해, Rust `Issue` 모델의 `sprint_id` 필드는 유지하되 `m.sprint_id AS sprint_id` 형태로 동적 derived 값을 채워 응답합니다.

## Consequences
- **긍정적 효과**: 
  - 스프린트 소속 정보가 `missions` 테이블 한 곳으로 일원화되어 데이터 캐시 drift 및 불일치 위험이 근본적으로 차단됩니다.
  - 미션의 스프린트를 변경하면 산하의 모든 에픽과 이슈가 자동으로 함께 이동하므로 관리가 극히 단순화됩니다.
- **부정적 효과**: 
  - `issue_create` 및 `issue_set_sprint` 호출처에서 `sprint_id`를 직접 조작하는 로직을 수정해야 하는 Breaking Change가 발생합니다.
  - 테이블 컬럼을 DROP하는 마이그레이션이 포함되므로, 무중단 배포 시 롤백이 불가한 비가역적 성격을 띱니다.

## Trade-offs
- **Option A (sprint_id 자동 상속 보강)**: 중복 컬럼 구조를 유지하되 코드 레벨에서 상속 동기화만 강제하는 대안. 구현이 쉽지만 DB WAL 격리 등으로 인한 상태 불일치 버그가 재발할 여지가 큽니다.
- **Option B (Read 경로만 JOIN 전환)**: DB 컬럼은 그대로 두고 쿼리만 JOIN으로 바꾸는 대안. schema evolution의 마찰은 적지만, 사용되지 않는 쓰레기 데이터 컬럼이 유지되어 스키마 혼란을 가중시킵니다.
- **Option C (Option B + 컬럼 DROP, 채택)**: 아키텍처의 명확성과 데이터 정합성을 가장 확실하게 보장하며 중장기 기술 부채를 제거할 수 있어 최종 선택되었습니다.
