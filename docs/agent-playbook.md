# Engram Agent Playbook (단독 에이전트 사용 가이드)

이 문서는 별도의 오케스트레이터 플러그인 없이, **단독 AI 에이전트가 engram MCP 도구를 활용하여 이슈 및 태스크를 관리하고 작업을 진행하는 표준 워크플로**를 설명합니다.

---

## 1. 세션 시작 (Session Start)

에이전트가 새로운 환경에 투입되어 작업을 시작할 때는 항상 가장 먼저 `session_restore` 도구를 호출하여 이전 작업 상태와 현재 프로젝트의 상황을 복원하고 파악해야 합니다.

* **권장 호출**:
  ```json
  session_restore(project_key="engram", compact=true)
  ```

### 응답 4대 핵심 필드 해석법
`session_restore`는 단순 복원을 넘어 에이전트가 즉시 취해야 할 행동을 제시합니다.
1. `next_action`: 에이전트가 즉시 수행해야 할 최우선 행동 권장 사항(예: "이슈 점유", "태스크 4번 진행").
2. `active_caveats`: 프로젝트나 스프린트 범위에 등록된 주의 사항(caveat) 목록. **작업 전 반드시 숙지해야 하는 함정이나 규칙입니다.**
3. `stalled_working`: `working` 상태에 오랫동안 정체되어 있는 이슈 목록.
4. `pending_drafts`: 임시 저장된 상태의 이슈 또는 작성 중인 드래프트 목록.

#### ✅ 좋은 예 (Good)
> `session_restore`를 호출한 직후 `active_caveats`를 꼼꼼히 확인하고, 지시된 `next_action`을 최우선으로 실행 계획에 반영함.

#### ❌ 안티패턴 (Bad)
> `session_restore`를 건너뛰고 곧바로 `issue_list`를 조회하여 임의의 이슈를 claim하고 작업을 시작해, 주의 사항(`active_caveats`)에 기록된 중요한 제약 조건을 놓침.

---

## 2. 이슈 점유 (Issue Claim)

에이전트가 작업할 이슈를 결정했다면, 다른 에이전트와의 중복 작업을 방지하기 위해 반드시 점유(`issue_claim`)를 획득해야 합니다.

* **권장 호출**:
  ```json
  issue_claim(id=193, agent_id="your-agent-id")
  ```

### CAS (Compare-and-Swap) 실패 및 좀비 Lease 회수
- **CAS 실패 (Conflict)**: 이미 다른 에이전트가 점유 중이거나 상태가 전이된 경우 `-32000: Conflict` 에러가 발생합니다. 이 경우 임의로 작업을 재시도하지 말고, 잠시 대기(backoff retry) 후 다시 조회하거나 다른 이슈를 픽업해야 합니다.
- **강제 점유 해제 (`force=true`)**: 담당 에이전트가 크래시 등의 이유로 동작하지 않아 정체 상태(stalled)가 지속될 때만 `force=true` 옵션을 사용하여 점유를 강제 회수합니다. **정당한 이유 없이 남의 점유를 빼앗아서는 안 됩니다.**

#### ✅ 좋은 예 (Good)
```json
// 일반적인 점유 시도
issue_claim(id=193, agent_id="gemini-cli")
```

#### ❌ 안티패턴 (Bad)
> 다른 에이전트가 작업 중인 이슈에 대해 무조건 `force=true`로 점유를 가로채어 병목이나 충돌을 유발함.

---

## 3. 작업 진행 (Task Workflow)

이슈가 점유(`working`)되면, 이슈 하위에 쪼개진 태스크들을 순차적으로 실행합니다.

* **권장 흐름**:
  1. `task_next(project_key="engram")`를 호출하여 현재 점유한 이슈에서 진행해야 할 최우선순위 태스크 정보를 획득합니다.
  2. 코드를 수정하고 작업을 완료합니다.
  3. 완료된 태스크의 상태를 업데이트합니다:
     ```json
     task_update(id=42, status="finished", agent_id="your-agent-id")
     ```
  4. 다음 태스크를 다시 `task_next`로 조회하여 반복합니다.

#### ✅ 좋은 예 (Good)
> 하나의 태스크 단위로 작업을 쪼개서 수행하고, 완료할 때마다 `task_update`를 호출하여 실시간 진척 상황을 DB에 반영함.

#### ❌ 안티패턴 (Bad)
> 이슈 하위의 태스크 10개를 모두 끝마칠 때까지 단 한 번도 `task_update`를 호출하지 않고, 마지막에 모든 태스크를 일괄 완료 처리하여 협업 중인 다른 에이전트/사용자가 진행 상황을 모르게 함.

---

## 4. 노트 작성 (Notes System)

작업 도중 발생하는 주의사항, 결정 사항, 장애 분석, 인수인계 정보 등은 노트를 생성(`note_add`)하여 저장소의 단일 진실 원천(SSOT)으로 기록합니다.

* **노트 타입(`note_type`)별 사용 시점**:
  - `caveat`: 작업 시 겪을 수 있는 함정이나 중요 정책 (sprint/project 범위로 브로드캐스트되어 `session_restore` 시 노출됨).
  - `decision`: 아키텍처나 비즈니스 로직 등 중요 의사결정 사항 기록.
  - `discovery`: 분석 중 발견한 새로운 사실이나 구조.
  - `blocker_detail`: 블로커(Blocker) 발생 시의 기술적 상세 내용 기술.
  - `context`: 데모 진입 전 필수 작성하는 인수인계 정보.
  - `reference`: 참고할 만한 관련 링크나 파일 경로 등.

* **scope별 필수 필드**:
  - `scope="issue"` ➔ `issue_id` 필수
  - `scope="task"` ➔ `task_id` 필수
  - `scope="sprint"` ➔ `scope_target_id` (sprint_id) + `scope="sprint"` 필수
  - `scope="epic"` ➔ `scope_target_id` (epic_id) + `scope="epic"` 필수
  - `scope="project"` ➔ `project_key` 필수

#### ✅ 좋은 예 (Good)
```json
note_add(
  scope="issue",
  issue_id=193,
  note_type="decision",
  summary="플레이북 포맷 통일 결정",
  detail="단독 에이전트의 오사용 방지를 위해 Good/Bad 예시를 무조건 1개씩 포함하기로 하였습니다.",
  agent_id="gemini-cli"
)
```

---

## 5. 데모 진입 및 Demo Gate (Demo & Review)

> [!CAUTION]
> **에이전트는 이슈 상태를 직접 `finished` 또는 `cancelled`로 변경할 수 없습니다!**
> 모든 작업이 완료되면 에이전트는 이슈 상태를 오직 **`demo` (검토 대기)** 상태까지만 업데이트해야 합니다. 최종 종결 처리는 사용자의 몫입니다.

* **데모 릴리즈 시퀀스**:
  1. **인수인계 노트 작성**: `note_add(note_type="context")`를 통해 구현 내용 및 사용자가 직접 검수할 수 있는 상세 가이드(검수 커맨드, 확인 경로 등)를 반드시 기록합니다.
  2. **이슈 릴리즈**: `issue_release`를 통해 점유를 해제하고 `demo` 상태로 전이합니다.
     ```json
     issue_release(id=193, agent_id="your-agent-id", transition_to="demo")
     ```

* **사용자(User) 전용 도구**:
  - `issue_finish(id, agent_id="user")` ➔ 오직 `agent_id`가 `user`인 경우에만 성공하며 이슈를 `finished`로 전이합니다.
  - `issue_cancel(id, reason, agent_id="user")` ➔ 이슈를 `cancelled`로 전이합니다.

#### ✅ 좋은 예 (Good)
> 모든 구현 완료 후 검수 가이드가 담긴 `context` 노트를 추가하고, `issue_release(transition_to="demo")`를 호출하여 사용자에게 바통을 넘김.

#### ❌ 안티패턴 (Bad)
> `issue_update(status="finished")`를 에이전트가 직접 호출하여 에러를 마주하거나, 검수 안내 노트 없이 무작정 `demo`로만 상태를 던져 사용자가 검수 방법을 파악할 수 없게 만듦.

---

## 6. 블로커 대응 (Handling Blockers)

작업 도중 타 이슈나 선행 태스크의 미비로 인해 더 이상 진행할 수 없는 블로커(Blocker) 상황이 발생할 경우, 시스템적으로 의존성 관계를 맺어 관리합니다.

* **블로커 대응 시퀀스**:
  1. `my_blocked_issues(project_key="engram")`를 호출하여 현재 프로젝트 내에서 자신의 작업 동선을 가로막고 있는 순환 의존성이나 리프 블로커 체인을 파악합니다.
  2. 해당 이슈에 대해 `note_add(note_type="blocker_detail")`로 블로킹 상황을 상세히 설명합니다.
  3. 이슈 간의 차단 링크를 생성합니다.
     ```json
     issue_link(source_id=193, target_id=194, link_type="blocks", agent_id="your-agent-id")
     // 193번 이슈가 완료되기 전에는 194번이 진행될 수 없음을 선언 (193 blocks 194)
     ```

---

## 7. 세션 종료 (Session End)

현재 환경에서의 작업을 마무리하고 세션을 종료할 때는 `session_end`를 호출하여 임시 생성된 환경을 정리합니다.

* **권장 호출**:
  ```json
  session_end(project_key="engram")
  ```

- 이 도구를 호출하면 세션 내에서 생성된 잔여 락(lock) 및 점유(lease)가 안전하게 클리어되거나 정리 대상(stalled 대기 등)으로 분류됩니다.

---
구체적인 API 도구별 입력 스키마 사양은 저장소의 `.claude/rules/mcp-tool-shape.md` 및 `docs/cli-mcp-parity.md`를 함께 참조해 주십시오.
