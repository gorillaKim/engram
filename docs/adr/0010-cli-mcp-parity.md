# ADR-0010: CLI ↔ MCP 도구 패리티 — 명명·플래그·JSON·exit code 컨벤션

## Status
Accepted

## Context
플러그인의 서브에이전트(engram-orchestrator 의 worker/leader/analyzer 등) 가 Agent SDK 의 tool whitelist 로 MCP 도구를 못 받거나 stdio MCP 에 붙지 못하는 환경에서, `engram <area> <verb>` 셸 호출로 동일한 워크플로를 수행해야 한다. 현재 CLI 는 MCP 도구 45 개 중 28 개만 1:1 매핑이 있고 17 개가 미노출이다 (`docs/cli-mcp-parity.md`). 또 `engram retro`/`engram hook` 처럼 CLI 만 가지는 명령과 섞여 있어, 후속 이슈에서 새 명령을 추가할 때마다 작성자가 명명·인자·출력형식·exit code 를 매번 자체 판단하면 일관성이 깨진다. 본 ADR 은 후속 이슈 #12 ~ #17 의 구현/리뷰가 따를 컨벤션을 확정한다.

## Decision

1. **명령 트리 = `engram <area> <verb>` 2단**.
   - `<area>` 는 MCP 도구 이름의 첫 토큰(`epic_`, `issue_`, `task_test_` 등) 을 따른다. snake_case → kebab-case 변환 (예: `task_test_` → `task-test`).
   - `<verb>` 는 MCP 도구 이름의 두 번째 토큰 이후 (예: `issue_set_sprint` → `engram issue set-sprint`).
   - 예외 — 의미가 area 가 아닌 도구:
     - `my_blocked_issues` → `engram blocked list` (신규 area `blocked`).
     - `board_status` → `engram board status` (신규 area `board`).
     - `session_restore` / `session_end` → 기존 `engram session restore|end` 유지.
     - `history_*` → 신규 area `history`.
2. **모든 CLI 명령에 글로벌 `--json` 플래그**. 기본 출력은 사람용 텍스트 (요약/체크리스트 마크업), `--json` 지정 시 MCP `tools/call` 응답과 의미적으로 동치인 raw JSON 을 stdout 으로 emit. `--json` 의 페이로드는 `serde_json::to_string_pretty(...)` 직렬화 결과. CLI 만의 부가 텍스트(이모지, "✅" 등) 는 절대 포함하지 않는다.
3. **인자 명명 규칙**.
   - 위치 인자는 단일 id 에만 사용 (`engram issue get <id>`, `engram note resolve <id>`).
   - 그 외는 `--long` flag (snake_case → kebab-case): `--project-key`, `--agent-id`, `--threshold-minutes`, `--scope-target-id`. 단, 의미상 명확한 별칭은 짧게 허용: `--project` (= project_key), `--epic` (= epic_id), `--sprint` (= sprint_id), `--issue` (= issue_id), `--task` (= task_id).
   - bool 은 `--flag` 단독 (예: `--backlog-only`, `--force`, `--include-resolved`). 기본 false.
   - enum 인자는 MCP 와 동일한 snake_case 문자열 그대로 받는다 (예: `--status working`, `--type caveat`, `--link-type blocks`).
4. **exit code**.
   - `0`: 성공.
   - `2`: 입력 검증 실패 (`Error::Validation`). 사람용 stderr 에 메시지, `--json` 모드면 stdout 에 `{"error":{"code":"validation","message":"..."}}`.
   - `3`: NotFound (`Error::NotFound`). 동일 포맷.
   - `4`: 점유/CAS 거부 (`issue_claim` 실패 등 — `Error::Conflict`).
   - `1`: 그 외 DB/IO 에러.
   - clap 의 파싱 실패는 그대로 `2` (clap 기본) 와 일치.
5. **agent_id 정책**. 사용자가 직접 부르는 CLI 호출은 `--agent-id` 미지정 시 `"user"` 로 fallback. 서브에이전트가 호출할 때는 `--agent-id <self>` 를 항상 명시한다 (#17 문서 갱신).
6. **`--json` 출력의 동치 보장은 #15 통합 테스트에서 검증**. MCP `tools::dispatch(name, args)` 의 `Value` 와 CLI 의 `stdout`(JSON 파싱 후) 가 의미적으로 동일해야 한다.

## Consequences
- 후속 이슈 #12 ~ #17 의 구현/리뷰가 명령형·인자명·exit code·JSON 모드 컨벤션을 본 ADR 에서 인용해 통일된다.
- 사용자용 텍스트 출력과 머신용 JSON 출력이 한 코드 경로에서 분기되므로 모든 CLI handler 에 `OutputFormat::{Human, Json}` 라우팅이 들어간다 (#12 인프라).
- 사용자 친화 명령 (`engram issue ready`, `engram task finish`, `engram retro`, `engram hook install`) 는 MCP 미노출 alias/CLI-only 로 남는다 — 패리티 검증 대상에서 제외된다 (`docs/cli-mcp-parity.md §9`).
- exit code 분류가 추가되어 hook 안에서 `engram` 호출 결과 처리가 정밀해진다 (예: `2` 는 사용자 입력 오류, `4` 는 CAS 충돌이라 재시도 가능 등).

## Trade-offs
- `--json` 을 모든 명령에 일괄 적용하는 비용이 들지만, `OutputFormat` 헬퍼 한 번에 처리 가능. 대안 (CLI 는 기본 JSON, `--pretty` 만 사람용) 은 기존 `engram retro` / `engram hook` 의 사용자 흐름과 충돌해 폐기.
- `area` 명명을 MCP 첫 토큰에서 기계적으로 도출하지 않고 `my_blocked_issues` → `blocked`, `board_status` → `board` 처럼 의미 기반으로 재배치한 케이스가 4 개 있다. 일관성 비용이 있지만, 사용자가 외울 표면이 작아져 받아들임.
- exit code 분류는 hook 통합 시 변별력이 필요하다는 판단. 단순화 (성공 0 / 실패 1) 안은 hook 자동 재시도 정책을 못 짠다는 이유로 폐기.
