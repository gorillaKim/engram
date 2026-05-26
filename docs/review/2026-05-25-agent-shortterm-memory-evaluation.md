# Engram 평가 — AI 에이전트 단기기억/작업 트래커 적합성

- **작성일**: 2026-05-25
- **작성자**: yhkim@madup.com (Claude Opus 4.7 보조)
- **평가 대상**:
  - `/Users/madup/gorillaProject/engram` (Rust 코어: engram-cli, engram-core, engram-desktop, engram-mcp)
  - `/Users/madup/gorillaProject/jake-marketplace/plugins/engram-orchestrator` (Claude Code 플러그인)
- **평가 관점**: engram = 단기기억(작업 계획·진행·완료 관리) / doxus = 장기기억(지식 베이스) 역할 분리 전제

---

## 1. 평가 배경

사용자가 engram 에 기대한 핵심 역할:

1. 해야 할 작업을 체계적으로 계획·추적·달성 관리
2. 이슈별 노트로 히스토리 보존 → 다음 작업자/리뷰어가 이어서 작업
3. 사용자가 언제든 개입해 진단·해결할 수 있는 가시성과 제어권

이 문서는 위 3가지 역할에 대한 engram 의 적합성과 운영 시 토큰 비용을 평가한다.

---

## 2. 역할 적합성 평가

### 2.1 작업 계획·추적·달성 관리 ★★★★★

| 요구 | engram 구현 | 평가 |
|---|---|---|
| 계층적 분해 | Sprint → Mission → Epic → Issue → Task (`tasks.ord` REAL 분수 인덱스 O(1) 삽입) | ★★★★★ 5계층은 다소 깊지만 mission/sprint 는 옵션. solo-track 으로 가볍게도 사용 가능 |
| 상태 머신 | `required → ready → working → demo → finished` (+ `cancelled`) | ★★★★★ 명확. `demo → finished`, `cancelled` 는 user-only 게이트 |
| 멀티 에이전트 안전성 | `issue_claim` CAS 락 + `agent_id` 강제 + history 감사 | ★★★★★ 동시 작업 시 race condition 물리적 차단 |
| 완료 검증 | leader 가 worker WORKER_RESULT YAML 의 evidence 를 Bash 로 재검증 (`task_list`/`git diff`/`test_check`) | ★★★★☆ 환각 방어 우수. 단, 재검증 비용은 토큰 소비 |
| 진행 가시화 | `board_status`, `stalled_issues`, `my_blocked_issues`, `planning_review_queue` | ★★★★★ 실시간 모니터링 도구 풍부 |

### 2.2 이슈별 노트로 히스토리·인계 ★★★★★

| 요구 | engram 구현 | 평가 |
|---|---|---|
| 노트 타입 | 7종: `comment`, `caveat`, `decision`, `discovery`, `blocker_detail`, `context`, `reference` | ★★★★★ 인계·결정 근거·발견·경고·차단 사유 전 범위 커버 |
| 인계 가능성 | `demo → finished` 시 `context` 노트 필수. `session_restore` 가 `active_caveats` 자동 노출 | ★★★★★ 다음 작업자가 빈손으로 들어와도 맥락 복원 |
| 스코프 | `project / sprint / epic / issue / task` 5단 broadcast | ★★★★☆ 강력. 프로젝트 차원 경고를 개별 이슈에 중복 기록 불필요 |
| 해결 추적 | `note.resolved` + `note_resolve(agent_id)`. 기본은 미해결만 노출 | ★★★★★ 노이즈 누적 방지 |
| [EVALUATION] 회고 입력 | worker 가 demo 전 [EVALUATION] 노트로 자기평가 → retro 자동 수집 | ★★★★☆ 회고 자동화 핵심 입력. 모델별 일관성은 프롬프트 의존 |

### 2.3 사용자 개입 가능성 ★★★★★

| 요구 | engram 구현 | 평가 |
|---|---|---|
| 최종 승인 게이트 | `demo → finished`, `cancelled` 는 `agent_id="user"` 만 통과. 에이전트 시도 시 `Error::Conflict` | ★★★★★ 가장 중요한 안전 장치 |
| 강제 회수 | `issue_release(force=true)` 로 stalled 락 해제 가능 (history 기록) | ★★★★☆ CLI/MCP 모두 가능. UX 는 Desktop UI 미완성 |
| 감사 추적 | `history_for`, `history_by_agent`, `history_recent` | ★★★★★ 사후 진단 가능 |
| 진단 도구 | `stalled_issues`, `my_blocked_issues`, `board_status` | ★★★★★ "지금 막힌 게 뭔지" 한 번에 파악 |
| 양방향 대화 | `comment` 노트 + AskUserQuestion 2-tier 에스컬레이션 (10분 코멘트 → 30분 AskUserQuestion) | ★★★☆☆ 폴링 기반 (`/loop 10m`). 푸시 알림 없음 |

**역할 적합성 종합: ★★★★★ (4.6/5)**

---

## 3. 아키텍처 강점 요약

1. **단일 SSOT**: `~/.engram/engram.db` SQLite (WAL). Epic 이 sprint 의 SSOT (ADR-0014) — 비정규화 없음.
2. **CLI/MCP 1:1 페리티** (ADR-0010): 14 CLI 서브커맨드 ↔ 56 MCP 도구 1:1 매핑. 사용자 개입 시에도 동일 도구 사용.
3. **v0.4.0 하이브리드 패턴**: worker(코드+노트) ↔ leader(상태 전이+검증) 책임 분리. 서브에이전트의 "가짜 MCP 호출" 환각을 worker 에서 도구 자체를 제거해 물리적으로 차단.
4. **Token-economy 설계**: `session_restore(compact=true)` summary-only 70% 페이로드 절감. note summary/detail 분리. fractional `ord` 인덱스로 task 재정렬 O(1).

---

## 4. 토큰 사용량 평가 ★★★★☆

### 4.1 정적 컨텍스트 비용

| 항목 | 추정 토큰 | 비고 |
|---|---|---|
| 7개 SKILL.md 메타데이터 | ~500 | 항상 로드됨 |
| 56개 MCP tool 스키마 | ~0 | ✅ deferred 처리 (ToolSearch 로 lazy load) |
| MCP 서버 instructions | ~0 | engram 은 별도 instructions 없음 |
| **정적 합계** | **~500** | 매우 낮음 |

→ **정적 비용 우수**. 56개 도구가 컨텍스트 미점유.

### 4.2 동적 컨텍스트 비용

| 시나리오 | 토큰 비용 | 분석 |
|---|---|---|
| **solo-track 1 이슈 처리** | ~5~15K | ✅ 가벼움 |
| **intake-as-issue → analyzer 분기** | ~15K | 중간. 서브에이전트 별도 컨텍스트 |
| **work-journaling 표준 흐름** (analyzer→leader→worker→leader) | **~40~80K** ⚠️ | 무거움. leader 의 evidence 재검증 추가 비용 |
| **sprint-retro** | ~30~100K | 스프린트 크기 비례. 50+ 이슈 시 주의 |
| **session_restore (compact=true)** | ~2~5K | ✅ 매우 효율적 |
| **session_restore (compact=false)** | ~10~30K | ❌ 사용 금지 권장 |

### 4.3 과다 사용 위험 패턴

1. **leader 의 이중 검증 비용**
   - worker 가 WORKER_RESULT 로 보고한 evidence(`git_diff`, `task_list`, `test_check`)를 leader 가 Bash 로 재호출
   - 환각 방어로는 필수지만 **단순 작업에서는 과잉**
   - 완화책: 1-task 짜리 단순 변경은 solo-track 으로 우회 (서브에이전트 spawn 자체 회피)

2. **agent 프롬프트 크기**
   - `engram-leader.md` 355줄 / `engram-worker.md` 278줄 / `engram-retro.md` 353줄
   - 서브에이전트 spawn 시 메인 컨텍스트와 분리되지만 spawn 비용 자체는 있음

3. **`note_list`/`history_for` 무지성 호출**
   - 누적 시 응답 비대화
   - 완화책: 기본이 summary-only + `include_resolved=false`. **detail 은 `note_get` 으로 한 건씩** 패턴 강제

4. **`/loop 10m` 폴링**
   - 푸시 알림 부재로 stalled 모니터링이 폴링 기반
   - 매 폴링이 `session_restore` 호출 → 가벼우나 누적 부담

### 4.4 토큰 효율 종합

**★★★★☆ (4/5)** — 설계는 토큰을 의식하고 있으나(compact 모드, summary/detail 분리, deferred tool, fractional ord), **다단계 에이전트 파이프라인은 본질적으로 무거움**. 모든 작업을 work-journaling 으로 강제하는 건 비효율. **작업 규모별 트랙 분기가 결정적**.

---

## 5. 알려진 한계 (README 명시)

- 태스크 단위 claim 없음 (이슈 단위만)
- lease 자동 만료 없음 → stalled 시 사용자 force-release 의존
- 푸시 알림 없음 → `/loop` 폴링
- `project_create` MCP 부재 (CLI 만)
- Desktop UI Phase 3 미완성 → CLI/MCP 가 주 인터페이스
- note `custom_type` 미구현

---

## 6. 권장 운영 가이드라인

### 6.1 작업 규모별 트랙 선택

| 작업 규모 | 권장 트랙 | 토큰 비용 |
|---|---|---|
| 1~3 task, 단일 PR, 1세션 완결 | **solo-track** (메인 에이전트 직접) | 5~15K |
| 4~10 task, 다단계 검증 필요 | **work-journaling** (analyzer→leader→worker) | 40~80K |
| 신규 분기 목표·로드맵 | **mission-plan** + work-journaling | 50~150K |
| 코드 변경 없는 조사·문서화 | **solo-track** 또는 doxus 만 사용 | 2~10K |

### 6.2 토큰 절약 베스트 프랙티스

1. `session_restore` 는 **항상 `compact=true`** — detail 은 필요할 때 `note_get`
2. `board_status` 도 `compact=true` 기본 사용
3. 단순 작업은 **work-journaling 우회**, solo-track 으로
4. `note_list` 결과가 크면 **summary 만 읽고 필요한 노트만 `note_get`**
5. retro 는 **스프린트 종료 시점 1회만** — 중간 호출 금지

### 6.3 사용자 개입 패턴

- 막힌 이슈 진단: `stalled_issues(threshold_minutes=30)` → `history_for` 로 마지막 액션 확인
- 강제 회수: `issue_release(id, agent_id="user", force=true)`
- 승인: Desktop UI 또는 CLI `engram issue finish <id>`
- 대화: `note_add(type=comment, author="user")` → 워커가 다음 사이클에 응답

---

## 7. 종합 평가

| 항목 | 점수 | 코멘트 |
|---|---|---|
| 작업 계획·추적 | ★★★★★ | 5계층 + 명확한 상태머신 |
| 히스토리·인계 | ★★★★★ | 7종 노트 + scope broadcast + resolve |
| 사용자 개입성 | ★★★★★ | demo→finished user-only + force-release + 감사 추적 |
| 멀티 에이전트 안전 | ★★★★★ | CAS claim + agent_id 강제 + worker/leader 분리 |
| 토큰 효율 | ★★★★☆ | 설계는 의식적이나 full pipeline 무거움 — 트랙 분기 필수 |
| 문서·온보딩 | ★★★★☆ | CHANGELOG/README 충실, ADR 존재. 로드맵 ADR 부족 |
| 알려진 한계 대응 | ★★★☆☆ | 폴링 의존·lease 만료 없음 → 운영에서 사용자 부담 |

### 결론

**engram 은 사용자가 기대한 "단기기억" 역할에 매우 잘 맞도록 설계되어 있다.** 특히:

1. `demo → finished` user-only 게이트
2. 7종 노트의 인계 시스템
3. leader 의 evidence 재검증을 통한 환각 방어

…는 "안전한 단기기억" 의 정수다.

**다만 토큰 사용은 작업 규모와 무관하게 full pipeline 을 쓰면 과해진다.** 사용자가 의식적으로 **solo-track ↔ work-journaling 을 작업 규모에 따라 분기**하는 운영 원칙을 세우면 토큰 비용을 60~70% 절감 가능. `intake-as-issue` 스킬에 **명확한 임계선(예: 예상 task ≤ 3 → solo-track 강제)** 룰이 더 강하게 박혀 있어야 함.

doxus(장기기억)와의 역할 분리도 자연스럽다 — engram 은 "지금 무엇을 하고 있는가", doxus 는 "과거에 무엇을 결정·발견했는가". **decision/discovery 노트의 요약본은 engram, 정식 문서화는 doxus 로 승격**하는 운영 흐름이 이상적.

---

## 8. 개선 제안 (선택)

1. **`solo-track` 자동 추천 임계선 명시화** — SKILL 에 "예상 task ≤ 3 또는 단일 파일 변경이면 자동 solo-track" 룰 추가
2. **`session_restore` 기본값을 `compact=true`** 로 강제 (현재는 옵션)
3. **lease 자동 만료** (e.g., 2시간) — stalled recovery 자동화
4. **engram → doxus 승격 헬퍼** — `decision` 노트를 doxus ADR 로 변환하는 스크립트

---

## 9. 검증 포인트 (이 평가의 사실관계 확인 방법)

- `engram --help` / `engram-mcp` 도구 목록 → `crates/engram-mcp/src/`
- README 핵심 섹션: `jake-marketplace/plugins/engram-orchestrator/README.md`
- 상태머신: `crates/engram-core/src/models/issue.rs`
- 관련 ADR: ADR-0007 (Agent Demo Gate), ADR-0009 (Audit + CAS), ADR-0010 (CLI/MCP Parity), ADR-0014 (Mission as sprint-agnostic SSOT) → `docs/adr/`
- 워커 컨트랙트: `agents/engram-worker.md`, `skills/work-journaling/SKILL.md`
- 토큰 추정은 라인 수 × ~3 토큰/라인 어림. 실측은 MCP 호출 후 응답 길이 관찰로 가능.
