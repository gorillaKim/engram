# CHANGELOG

## [Unreleased] - 2026-05-24

### Breaking Changes ⚠️
- **스프린트 소속 여부의 SSOT(단일 진실 원천)를 `mission.sprint_id`로 단일화 (Option C)**
  - `issues` 및 `epics` 테이블에서 `sprint_id` 컬럼이 데이터베이스 스키마 상에서 완전히 제거(DROP)되었습니다.
  - 이제 `issue_create` API 및 CLI 명령에서 `sprint_id`를 직접 지정해 생성하는 행위는 제한되며, 지정 시 Validation Error가 반환됩니다.
  - `issue_set_sprint` API 및 CLI 명령은 Deprecated로 전환되었으며 호출 시 ValidationError로 거부됩니다.
  - 이슈의 스프린트는 소속 미션(`mission.sprint_id`)을 통해서만 결정되며, 에픽 및 이슈 조회 시 내부 조인을 거쳐 `sprint_id` derived 필드가 동적으로 계산되어 호환성 있게 응답합니다.
  - 미완료 일감 이관 및 스프린트 완수 시 이관 단위가 `issue`에서 `mission` 단위(`mission_set_sprint`) 일괄 처리로 변경되었습니다.

### Added ➕
- `workflow_test.rs`에 스프린트 소속 미션 변경 시 derived `sprint_id`가 연동되어 변하는지 확인하는 `test_issue_sprint_id_follows_mission` 테스트 케이스 추가.
- `0010_drop_sprint_columns.sql` 마이그레이션 스크립트 작성으로 `issues`, `epics` 테이블에서 `sprint_id` 컬럼 DROP 처리.

### Fixed 🔧
- `engram-desktop` 테스트 헬퍼(`seed_issue`) 및 `engram-mcp` 디스패치 통합 테스트(`dispatch_test.rs`) 내 deprecated된 `issue_set_sprint` 호출 지점 제거 및 미션 연동으로 수정하여 workspace 테스트 정상화.
