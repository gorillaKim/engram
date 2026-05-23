# CLI ↔ MCP 도구 패리티 매트릭스

이 문서는 `engram-mcp` 가 노출하는 모든 MCP 도구와 `engram-cli` 서브커맨드의 **현재 매핑 상태 / 갭 / 목표 명령형** 을 기록하는 단일 진실 원천(SSOT)이다.
후속 이슈(#12 ~ #17) 는 이 문서를 기준으로 구현/검증을 진행한다.

명명/플래그/JSON/exit code 컨벤션은 [ADR-0010](./adr/0010-cli-mcp-parity.md) 에 별도로 결정.

## 조사 기준 시점

- 코드: `crates/engram-mcp/src/tools/*.rs` (45 tool_definitions), `crates/engram-cli/src/commands/*.rs`
- 추가로 `crates/engram-mcp/src/tools/mod.rs::dispatch` 의 분기 47개 중 **2개는 tool_definitions 에 등록되지 않음** — 이는 본 작업 외 별도 발견사항(아래 §5 참조).

## 1) sprint area (5/5)

| MCP 도구          | 현 CLI                          | 갭 | 목표 명령형                              |
|-------------------|---------------------------------|----|------------------------------------------|
| `sprint_create`   | `engram sprint create`          | -  | `engram sprint create --name X [--goal G] [--start D] [--end D]` |
| `sprint_list`     | `engram sprint list`            | -  | `engram sprint list [--json]`            |
| `sprint_current`  | `engram sprint current`         | -  | `engram sprint current [--json]`         |
| `sprint_update`   | `engram sprint update`          | -  | `engram sprint update <id> [--name X] [--status S] [--goal G]` |
| `sprint_delete`   | `engram sprint delete`          | -  | `engram sprint delete <id>`              |

→ 패리티 완성. JSON 표준 출력 확인만 필요 (#12).

## 2) epic area (4/5 — `epic_delete` 누락)

| MCP 도구       | 현 CLI                | 갭                   | 목표 명령형                                    |
|----------------|-----------------------|----------------------|-----------------------------------------------|
| `epic_create`  | `engram epic create`  | `mission_id` required 추가 (M6) | `engram epic create --mission M --project P --title T [--description D]` |
| `epic_get`     | `engram epic get`     | -                    | `engram epic get <id>`                         |
| `epic_list`    | `engram epic list`    | `include_completed` 인자 추가 (M6) | `engram epic list [--project P] [--status S] [--include-completed]` |
| `epic_update`  | `engram epic update`  | description 인자 존재 — OK | `engram epic update <id> [--status S] [--title T] [--description D]` |
| `epic_delete`  | (없음)                | **CLI 미노출**       | `engram epic delete <id>`                      |

→ 갭 1건 (#13 처리). M6: `epic_create`에 `--mission` required 추가, `epic_list`에 `--include-completed` 추가.

## 3) issue area (7/13 — claim/release/delete/set-sprint/blocked/stalled 누락)

| MCP 도구             | 현 CLI                       | 갭                  | 목표 명령형                                                     |
|----------------------|------------------------------|---------------------|-----------------------------------------------------------------|
| `issue_create`       | `engram issue create`        | -                   | `engram issue create --epic E [--sprint S] --title T`           |
| `issue_get`          | `engram issue get`           | -                   | `engram issue get <id>`                                         |
| `issue_list`         | `engram issue list`          | `--status`, `--sprint`, `--backlog-only` 인자 미지원; `--mission M` 추가 (M6) | `engram issue list [--project P] [--epic E] [--sprint S] [--status S] [--backlog-only] [--mission M]` |
| `issue_update`       | `engram issue update`, `engram issue ready` | `ready` 는 편의용 (유지) | `engram issue update <id> [--status S] [--priority P] [--title T]` |
| `issue_link`         | `engram issue link`          | -                   | `engram issue link --source S --target T [--type blocks]`       |
| `issue_unlink`       | `engram issue unlink`        | -                   | `engram issue unlink --link-id L`                               |
| `issue_delete`       | (없음)                       | **CLI 미노출**      | `engram issue delete <id>`                                      |
| `issue_claim`        | `engram issue claim`         | -                   | `engram [--agent-id A] issue claim <id> [--agent-id A]`         |
| `issue_release`      | `engram issue release`       | -                   | `engram [--agent-id A] issue release <id> [--agent-id A] --transition-to T [--force]` |
| `issue_set_sprint`   | (없음)                       | **CLI 미노출**      | `engram issue set-sprint <id> --sprint S`                       |
| `my_blocked_issues`  | (없음)                       | **CLI 미노출**      | `engram blocked list --project P`  *(신규 area, §7)*             |
| `stalled_issues`     | (없음)                       | **CLI 미노출**      | `engram issue stalled --threshold-minutes 10 [--project P] [--status working]` |

→ 갭 6건 (#13 + #14 분할 처리). `my_blocked_issues` 는 issue 가 아닌 신규 `blocked` area 로 분리 권장 (§7).

## 4) task area (6/6 + `task_finish` 가 CLI 만의 별도 verb)

| MCP 도구             | 현 CLI                          | 갭                         | 목표 명령형                                  |
|----------------------|---------------------------------|----------------------------|---------------------------------------------|
| `task_create`        | `engram task create`            | -                          | `engram task create --issue I --title T [--goal G]` |
| `task_list`          | `engram task list`              | `--status` 인자 미지원     | `engram task list --issue I [--status S]`   |
| `task_update`        | `engram task update`, `engram task finish` (편의) | `finish` 는 user-only 경로로 유지 | `engram task update <id> [--status S] [--title T]` |
| `task_insert_after`  | `engram task insert-after`      | -                          | `engram task insert-after --issue I --after A --title T` |
| `task_next`          | `engram task next`              | -                          | `engram task next [--project P]`            |
| `task_delete`        | (없음)                          | **CLI 미노출**             | `engram task delete <id>`                   |

→ 갭 1건 (#13).

## 5) task_test area (0/7 — area 자체 없음)

| MCP 도구               | 현 CLI | 갭                | 목표 명령형                                          |
|------------------------|--------|-------------------|-----------------------------------------------------|
| `task_test_add`        | (없음) | **CLI 미노출**    | `engram task-test add --task T --label L`           |
| `task_test_add_bulk`   | (없음) | **CLI 미노출**    | `engram task-test add-bulk --task T --labels "A,B,C"` (또는 `--from-file`) |
| `task_test_list`       | (없음) | **CLI 미노출**    | `engram task-test list --task T`                    |
| `task_test_check`      | (없음) | **CLI 미노출**    | `engram task-test check <id>`                       |
| `task_test_check-bulk` | (없음) | **CLI 미노출**    | `engram task-test check-bulk --ids "1,2,3"`         |
| `task_test_uncheck`    | (없음) | **CLI 미노출**    | `engram task-test uncheck <id>`                     |
| `task_test_remove`     | (없음) | **CLI 미노출**    | `engram task-test remove <id>`                      |

→ 갭 7건 (#14: 신규 area).

## 6) note area (4/4)

| MCP 도구       | 현 CLI                  | 갭                                | 목표 명령형                                                                  |
|----------------|-------------------------|-----------------------------------|------------------------------------------------------------------------------|
| `note_add`     | `engram note add`       | scope/broadcast 인자 미지원        | `engram note add --type T --summary S [--detail D] [--scope project --project P] [--scope epic --target ID] [--scope sprint --target ID] [--issue I] [--task T] [--agent-id A]` |
| `note_list`    | `engram note list`      | `--type` 필터, `--include-resolved`, broadcast scope 조회 미지원 | `engram note list [--issue I] [--task T] [--type T] [--include-resolved]`     |
| `note_get`     | `engram note get`       | -                                 | `engram note get <id>`                                                       |
| `note_resolve` | `engram note resolve`   | -                                 | `engram note resolve <id> [--agent-id A]`                                    |

→ 갭 2건 (#13: note_add scope 확장, note_list 필터 인자).

## 7) session area (2/3 — `board_status` 누락)

| MCP 도구          | 현 CLI                   | 갭                                 | 목표 명령형                                  |
|-------------------|--------------------------|------------------------------------|---------------------------------------------|
| `session_restore` | `engram session restore` | `compact` 파라미터 추가됨 (#176)   | `engram session restore [--project P] [--compact]` |
| `session_end`     | `engram session end`     | -                                  | `engram session end [--project P]`          |
| `board_status`    | (없음)                   | **CLI 미노출** — `engram board status` 신규 area 로 노출 권장 | `engram board status [--project P]`         |

→ 갭 1건 (#14: 신규 `board` area).

## 8) history area (0/3 — area 자체 없음)

| MCP 도구          | 현 CLI | 갭                | 목표 명령형                                                 |
|-------------------|--------|-------------------|-------------------------------------------------------------|
| `history_for`     | (없음) | **CLI 미노출**    | `engram history for --entity-type E --entity-id I`          |
| `history_by_agent`| (없음) | **CLI 미노출**    | `engram history by-agent --agent-id A [--limit N]`          |
| `history_recent`  | (없음) | **CLI 미노출**    | `engram history recent [--since-minutes M] [--limit N]`     |

→ 갭 3건 (#14: 신규 `history` area).

## 9) mission area (0/7 — M6 신규, area 자체 없음)

| MCP 도구              | 현 CLI | 갭                | 목표 명령형                                                          |
|-----------------------|--------|-------------------|---------------------------------------------------------------------|
| `mission_create`      | (없음) | **CLI 미노출**    | `engram mission create --title T [--description D] [--sprint S]`    |
| `mission_get`         | (없음) | **CLI 미노출**    | `engram mission get <id>`                                            |
| `mission_list`        | (없음) | **CLI 미노출**    | `engram mission list [--sprint S] [--status S]`                     |
| `mission_update`      | (없음) | **CLI 미노출**    | `engram mission update <id> [--title T] [--description D] [--status S]` |
| `mission_delete`      | (없음) | **CLI 미노출**    | `engram mission delete <id>` *(하위 epic 있으면 거부)*              |
| `mission_get_tree`    | (없음) | **CLI 미노출**    | `engram mission tree <id> [--include-completed]` *(active only 기본)* |
| `mission_set_sprint`  | (없음) | **CLI 미노출**    | `engram mission set-sprint <id> --sprint S`                          |

→ 갭 7건 (M6 신규 area). `mission_update(status=completed|cancelled)` 는 사용자 전용 — `agent-demo-gate.md` §Mission 참조.

## 10) CLI-only 명령 (MCP 미노출 — 유지)

| CLI 명령                              | 비고                                                            |
|---------------------------------------|----------------------------------------------------------------|
| `engram retro [--sprint S]`           | 회고 리포트. retro_report 는 repository 노출만, MCP 미노출 — 유지 |
| `engram hook install / uninstall / post-session-check` | Claude Code 통합 전용. MCP 노출 부적합                |
| `engram snapshot-text [--project P]`  | Hook 내부에서 호출되는 텍스트 출력. MCP 미노출 — 유지            |
| `engram issue ready <id>`             | `issue_update(status=ready)` 의 편의 alias — 유지              |
| `engram task finish <id>`             | `task_update(status=finished)` 의 편의 alias — 유지            |

## 11) 발견사항 / 후속 조치

1. **CLI 의 `issue list` 가 `--status`, `--sprint`, `--backlog-only` 미지원** — IssueFilter 의 일부 필드가 CLI 로 빠짐.
2. **CLI 의 `note add` 가 broadcast scope (project/sprint/epic) 미지원** — Phase 2 의 broadcast 노트가 CLI 로 못 만들어짐.
3. **`engram epic list --status` 인자 미지원** — `epic_list` MCP 는 status 필터 받음.

## 12) 총계

| 항목                                | 수치  |
|------------------------------------|-------|
| MCP tool_definitions               | **52** (기존 45 + M6 mission 7) |
| MCP dispatch 분기                  | 45+ (tool_definitions 와 동기화됨) |
| 1:1 CLI 매핑 존재                  | 28    |
| **CLI 미노출 (구현 필요)**         | **24** (기존 17 + mission 7) |
| 추가 인자 보강 필요 CLI            | 6 (기존 4 + epic_create mission_id, epic_list include_completed) |

후속 이슈 분담:
- **#12** (CLI JSON 인프라): 28 + 17 = 45 모든 명령에 `--json` 적용.
- **#13** (기존 area 보강): epic_delete, issue_delete/claim/release/set_sprint, task_delete, note 인자 보강, issue/epic/task `--status` 필터 — **11 verb / 4 인자 보강**.
- **#14** (신규 area): `task-test` (7), `history` (3), `board` (1), `blocked` (1) — **12 verb**.
- **#15** (동치 통합 테스트): CLI 호출 vs `mcp dispatch` 결과 동치 비교 — 45 도구 전부.
- **#16** (배포): cargo install / prebuilt binary 경로 ADR 의 명령형 확정 후 동봉.
- **#17** (문서): 본 매트릭스 + ADR-0010 + 서브에이전트 프롬프트.
- **M6** (mission area): `mission` 신규 area 7 verb CLI 구현 + `epic_create --mission` required + `epic_list --include-completed` + `issue_list --mission`.
