# CHANGELOG

## [0.1.51] - 2026-06-18

### Optimized ⚡
- **`session_restore` 페이로드 최적화 (Finished 이슈 상세 배제)**
  - 활성 스프린트 내 완료된(`Finished`) 이슈들의 상세 정보 및 요약 데이터를 `session_restore` 반환 목록(`active_epics.active_issues` 및 `active_issues_compact`)에서 안전하게 제외하여 전송 토큰 크기를 획기적으로 줄였습니다.
  - 상위 수준의 진행률 집계(`EpicProgress.done` 및 `total`)는 온전히 보존되어 미션/에픽 대시보드 진척율 무결성을 지켰습니다.

### Added ➕
- `workflow_test.rs`에 완료된 이슈가 `session_restore` 상세 목록에서 필터링되는지 검증하는 `test_session_restore_excludes_finished_issues` 단위 테스트 추가.

---

## [0.1.50] - 2026-06-18

### Fixed 🔧
- **`session_restore` Truncation 우선순위 역전 결함 수정**
  - 응답 크기 제한(`size_limit`) 초과 시 가장 중요한 핵심 정보인 `active_epics`가 가장 먼저 잘려 나가던 결함을 수정하여, 상대적으로 덜 중요한 `active_caveats`, `pending_drafts`부터 순차적으로 절단되도록 순서를 바로잡았습니다.
- **MCP compact 이슈 스키마-구현 불일치 정정**
  - `issue_get` 및 `issue_list` 의 `compact` 인자가 `true`일 때 스키마 명세(NULL 반환)와 실제 Rust 구현(200자 truncation)이 다르던 문제를 스키마 텍스트 수정으로 일치화했습니다.

---

## [0.1.49] - 2026-06-18

### Added ➕
- **데스크톱 가이드 및 QnA 섹션 추가**
  - 사용자 가이드 및 FAQ 조회를 위한 전용 Drawer/마크다운 렌더러 컴포넌트를 데스크톱 앱 내에 탑재했습니다.

---

## [0.1.48] - 2026-06-18

### Fixed 🔧
- **데스크톱 앱 자동 업데이트 루프 버그 수정**
  - Tauri 버전 및 배포 워크플로우 내 버전 명세를 완벽히 일치시켜 앱 기동 시 계속 중복 업데이트 모달이 노출되던 무한 루프 오류를 해결했습니다.
- **배포 자동화 스크립트 도입**
  - Cargo.toml, tauri.conf.json 버전 범프, 커밋, 원격 푸시 및 깃 태그 배포를 일괄 자동화하는 `release.sh` 스크립트를 작성하여 릴리즈 파이프라인의 수동 실수를 방지했습니다.

---

## [0.1.47] - 2026-06-17

### Added ➕
- **API 데이터 미니멀리즘 최적화 및 SSE 격리**
  - `session_restore` 시 caveat detail 제외 및 count 기반 count-only 모드(`compact=true`)를 통해 API 페이로드 크기를 70% 이상 절감했습니다.
  - `note_add` API의 detail echo 생략 기능(`omit_detail` 플래그) 추가.
  - SSE transport 개발의 단일 정정 지점(Fixtures/상수화)을 Mock 데이터로 캡슐화 설계.

---

## [0.1.46] - 2026-06-16

### Fixed 🔧
- **`session_restore` size limit 우회 수정 및 매트릭스 통합 테스트 도입**
  - `compact` 모드 및 CLI `session restore --json` 경로에서 size guard 가 무시/우회되던 문제를 해결하고, CLI/MCP × compact × size_limit 조합에 대한 회귀 방지 통합 테스트를 수립했습니다.

---

## [0.1.45] - 2026-06-16

### Fixed 🔧
- **데스크톱 노트 개행 및 마크다운 렌더링 개선**
  - 데스크톱 앱의 마크다운 리스트(ul, ol) 들여쓰기 렌더링 스타일 수정 및 텍스트 내 개행문자(`\n`)가 정상 렌더링되도록 수정했습니다.

---

## [0.1.44] - 2026-06-16

### Fixed 🔧
- **CLI 입력 및 데스크톱 노트 개행 처리 버그 수정**
  - CLI 인자 파싱 및 렌더러 연동 시 줄바꿈 문자가 유실되는 문제를 전반적으로 수정한 패치 버전입니다.

---

## [0.1.43] - 2026-05-24

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
