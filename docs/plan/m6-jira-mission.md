# Milestone 6: Jira Mission (지라 이슈 연동 및 추적성 개선)

> 상위 계획: [Jira Mission integration plan](file:///Users/madup/.gemini/antigravity/brain/37d80d06-b369-429e-877c-05f0fa770a37/implementation_plan.md)

지라(Jira) 원본 이슈를 가져와 작업을 세분화할 때, 지라 이슈 하나가 여러 에픽(Epic)과 이슈(Issue)로 파편화되면서 전체 일감을 일목요연하게 추적하기 어려운 페인 포인트를 해결하기 위한 마일스톤 6(M6) 계획입니다.

---

## 1. 아키텍처 및 데이터 흐름

### A. 데이터 계층 구조 (1:N 매핑)
기존의 `Epic ➔ Issue ➔ Task` 흐름을 유지하며, 최상위에 Jira 원본 이슈와 매핑되는 `Mission` 계층을 도입합니다.
```
Sprint (시간 단위)
 └── Mission (최상위: Jira 이슈 1:1 매핑)
      ├── Epic (중간 계층: mission_id 추가)
      │    └── Issue (실행 계층: mission_id 추가)
      │         └── Task (세부 실행)
      └── Issue (Epic 없이 Mission에 직접 연동)
```

### B. mission_id 자동 상속 규칙
에이전트가 도구 호출 시 미션 ID 지정을 깜빡하더라도 시스템이 아래 규칙에 따라 자동으로 `mission_id`를 추론 및 상속합니다.
1. **이슈 생성 시:** `epic_id`를 기반으로 부모 에픽의 `mission_id`를 조회하여 자동 상속.
2. **에픽 생성 시:** 에픽을 생성하는 에이전트(`agent_id`)가 현재 `working` 상태로 작업 중인 이슈의 `mission_id`를 세션 맥락에서 조회하여 자동 상속.

---

## 2. 개발 범위 및 서브태스크 분할

이 마일스톤은 크게 3개의 에픽(Epic)과 하위 이슈들로 나누어 진행합니다.

### 📁 에픽 1: Core Engine & DB Schema (코어 인프라 개발)
Jira Mission을 수용하기 위한 DB 스키마 마이그레이션과 CRUD 및 진척도 집계 백엔드 로직을 구현합니다.
* **이슈 1.1:** `0008_missions.sql` 스키마 마이그레이션 작성 (`missions` 테이블 추가 및 `epics`/`issues` 외래키 추가)
* **이슈 1.2:** `Mission` 도메인 모델 및 CRUD 리포지토리 메서드 구현 (`mission_create`, `mission_get`, `mission_list`, `mission_update`, `mission_delete`)
* **이슈 1.3:** 미션별 종합 진척도 산출을 위한 `mission_progress_query` 구현 (소속 에픽/이슈 상태별 집계 및 완료율 계산)
* **이슈 1.4:** `epic_create`/`issue_create` 시 `mission_id` 자동 상속(Inheritance) 메커니즘 백엔드 연동
* **이슈 1.5:** M6 핵심 로직 검증을 위한 통합 테스트(`mission_inheritance_workflow`) 구현

### 📁 에픽 2: MCP & CLI Interface (인터페이스 확장)
에이전트와 사용자가 터미널 및 MCP 클라이언트를 통해 Mission을 관리하고 구조적 트리를 조회할 수 있게 돕습니다.
* **이슈 2.1:** `mission_create`, `mission_list`, `mission_update`, `mission_delete` MCP 도구 구현 및 라우팅 추가
* **이슈 2.2:** 에이전트 인지력 향상을 위한 **`mission_get_tree` MCP 도구 개발** (하향식 트리 구조 JSON 데이터 빌더 구현)
* **이슈 2.3:** 기존 `epic_create`/`issue_create` MCP 도구의 inputSchema에 `mission_id` 파라미터 옵션 지원
* **이슈 2.4:** `engram mission <command>` CLI 서브커맨드 구현 및 `--mission-id` 옵션 추가
* **이슈 2.5:** `crates/engram-cli/tests/parity_test.rs`에 Mission 패리티 회귀 방지 통합 테스트 시나리오 추가

### 📁 에픽 3: Desktop UI & Visualization (가시성 고도화)
사용자가 데스크톱 앱에서 미션별 카드 트리 뷰와 필터를 통해 진행 상황을 한눈에 통제하도록 개선합니다.
* **이슈 3.1:** `commands.rs`에 Mission 관련 Tauri IPC 커맨드 노출 및 연동
* **이슈 3.2:** 보드 상단 `FilterBar.tsx`에 Mission 선택 필터 드롭다운 연동
* **이슈 3.3:** **좌측 `Missions` 네비게이션 메뉴 및 가로형 계층 카드 트리 뷰(Horizontal Card Tree Board) 화면 신규 구축**
* **이슈 3.4:** 카드 트리 내에서 개별 이슈 클릭 시 우측 상세 Drawer가 연동되도록 연결
* **이슈 3.5:** 에픽/이슈 생성 모달 및 상세 정보 뷰에서 Mission 매핑/수정 인터페이스 제공

---

## 3. 검증 계획

* **단위 테스트:** `cargo test -p engram-core`를 통해 자동 상속 및 트리 쿼리 로직 검증.
* **통합 패리티 테스트:** CLI와 MCP 동치성 테스트 통과 확인.
* **UI 검증:** 데스크톱 앱 실행 후 Mission 보드 렌더링 상태 확인 및 필터 정상 작동 확인.
