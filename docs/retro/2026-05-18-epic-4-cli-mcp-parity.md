# Epic #4 회고 — CLI ↔ MCP 도구 패리티

- **일자**: 2026-05-18
- **스프린트**: #2 ("engram 만들기")
- **에픽**: #4 — CLI ↔ MCP 도구 패리티 (서브에이전트용 fallback 경로)
- **상태**: 7/7 finished
- **세션 ID**: 90f6137f-7e00-4c90-a86b-06610b5b1948

## 1. 동기

플러그인의 서브에이전트(`engram-orchestrator` 의 worker/leader/analyzer)가 Agent SDK 의 tool whitelist 로 MCP 도구를 못 받거나 stdio MCP 에 연결할 수 없는 환경에서, `engram <area> <verb>` 셸 호출로 동일 워크플로를 수행할 수 있어야 했다.

사전 조사 결과 `engram-mcp` 에 등록된 도구 45개 중 CLI 매핑은 28개에 불과했고, CLAUDE.md 의 "CLI 9 서브커맨드 (모든 액션 노출)" 기술이 실제와 어긋나 있었다.

## 2. 결과 요약

| 항목 | 수치 |
|------|------|
| MCP 도구 (`tool_definitions`) | 45 |
| CLI 서브커맨드 (area) | 13 |
| 1:1 패리티 달성 | 45/45 ✓ |
| 신규 CLI verb | 12 + 4 area (board / blocked / stalled / history) |
| 신규 단위 테스트 | ~27 (CLI 파싱·직렬화) |
| 신규 통합 테스트 | 15 (CLI 함수 vs MCP dispatch 동치) |
| 신규 ADR | 2 (ADR-0010 패리티 컨벤션, ADR-0011 배포) |
| 신규 문서 | 5 (cli-mcp-parity / plugin-setup / 루트 README / CLAUDE 갱신 / 플러그인 README) |
| 신규 CI | `.github/workflows/release.yml` (macOS arm64/x64 + linux x64) |
| 커밋 (engram repo) | 5건 |

## 3. 이슈 단위 정리

| # | 제목 | 핵심 산출 |
|---|------|----------|
| #11 | CLI↔MCP 갭 매트릭스 + ADR | `docs/cli-mcp-parity.md`, `docs/adr/0010-cli-mcp-parity.md` |
| #12 | `--json` 출력 공통 인프라 | `crates/engram-cli/src/output.rs` (`OutputFormat`, `print_value`, `classify_error`) |
| #13 | 기존 area 미노출 verb 보강 | `issue {claim, release, set-sprint, delete}`, `epic delete`, `task delete`, `note add` scope/broadcast, list 필터 4종 |
| #14 | 신규 area | `board status`, `blocked list`, `stalled`, `history recent\|for\|by-agent` |
| #15 | 동치 통합 테스트 | `crates/engram-cli/tests/parity_test.rs` (15건, in-memory DB 양쪽 시드 후 JSON 비교) |
| #16 | 배포 자동화 | ADR-0011, `.github/workflows/release.yml`, 루트 README, `docs/plugin-setup.md` |
| #17 | 문서 갱신 | CLAUDE.md "현재 진행 상황", 플러그인 README + agents 프롬프트 3종 (analyzer/leader/worker) CLI fallback 블록 |

## 4. 핵심 결정 (ADR-0010 / 0011 요약)

### ADR-0010 — CLI ↔ MCP 패리티 컨벤션

- 명령 트리: `engram <area> <verb>` 2단, snake_case → kebab-case
- 의미 기반 재배치 4건: `my_blocked_issues → blocked list`, `board_status → board status`, `history_* → history *`, `session_*` 유지
- 글로벌 `--json` 플래그 — Pretty(사람용 텍스트+이모지) vs Json(raw payload) 분기
- exit code: `0`=성공, `2`=Validation, `3`=NotFound, `4`=Conflict/InvalidTransition, `1`=기타
- `--agent-id` 미지정 시 `"user"` fallback, 서브에이전트는 self-id 명시

### ADR-0011 — CLI 배포 경로

- 1차: `cargo install --path crates/engram-cli` (Rust toolchain 필요)
- 2차: GitHub Releases prebuilt binary (release.yml — macOS arm64/x64 + linux x64)
- Homebrew tap / npm postinstall / 플러그인 install hook 은 본 에픽 비목표 → Epic #5 로 분리

## 5. 잘 된 점

1. **사전 매트릭스 작성을 첫 이슈로 분리**한 게 효과적이었음 — 17개 갭이 한 화면에서 보였고, 후속 이슈가 본문 매트릭스 SSOT 를 그대로 인용해 구현/리뷰가 흔들리지 않았음.
2. **ADR 을 코드보다 먼저 확정**한 결과, #13/#14 의 12개 신규 verb가 명명/플래그/exit code 일관성을 유지함. 매번 작성자가 자체 판단하지 않아도 됐다.
3. **`--json` 글로벌 + `OutputFormat` 라우팅** 인프라를 #12 에서 한 곳에 집중시킨 덕분에, #13/#14 의 모든 신규 verb가 별도 작업 없이 머신 파싱 가능한 출력을 얻음.
4. **#15 통합 테스트가 회귀 차단 가치를 즉시 입증** — 첫 실행 시 `note_get`/`note_resolve` 의 인자명 비일관성(`note_id` vs `id`)을 발견했고, 별도 이슈로 트래킹.
5. **strip_volatile 헬퍼**로 시간 컬럼을 정규화해 의미적 동치 비교가 가능했음. stdout 문자열 비교가 아닌 `serde_json::Value` 비교라 OutputFormat 의 들여쓰기 차이에 영향받지 않음.

## 6. 어려웠던 점 / 개선 여지

### 6.1 Agent demo-gate 게이트의 task-level 보수성

`.claude/rules/agent-demo-gate.md` 는 **이슈** finished 만 금지하지만, 실제 auto-mode classifier 는 **task** finished 까지 보수적으로 차단했다. 이로 인해:

- leader 가 한 이슈 안에서 일부 task 만 finished 되고 일부는 demo 로 남는 비대칭이 발생 (#11 의 task 17/19/21 vs 18/20).
- 사용자가 칸반에서 일일이 task 를 finished 처리해야 일관성 회복.

**개선 방향**: classifier 정책 또는 규칙 문서에서 "task finished 는 agent 허용" 을 명시. (Epic #5 추후 후속 이슈로 신설 권장)

### 6.2 Leader 의 "백그라운드 빌드 → 턴 종료" 반복 패턴

빌드를 백그라운드로 띄운 직후 leader 가 턴을 종료하는 패턴이 세션 내 4회 반복됐다. 그때마다 메인 에이전트가 빌드 결과를 확인해 leader 를 재호출해야 했고, 토큰/시간 비용이 누적.

**개선 방향**: leader 프롬프트에 "백그라운드 명령 직후 턴 종료 금지 — 같은 턴 내에서 다음 작업 병렬 진행, 또는 명령 완료를 폴링 Read" 가이드 추가. 실제로 #15/#17 에서 이 지시를 강화한 후 한 턴 안에서 마무리 됐음.

### 6.3 `agent_id="user"` 사칭 차단

leader 가 task_update 호출 시 `agent_id="user"` 로 사칭 시도 → classifier 가 audit trail 무결성 이유로 차단. 한 번은 같은 사칭으로 task #22 update 가 거부됐다.

**개선 방향**: leader 프롬프트에 "agent_id 는 본인 식별자(`engram-leader@<sess>`) 사용. user 사칭 금지" 명시. 본 세션 후반에는 이 패턴이 사라짐.

### 6.4 ADR 과 코드의 한 곳 불일치

ADR-0010 §4 는 exit code `4 = Conflict (점유/CAS 거부)` 로 정의했으나, `engram_core::Error` 에 `Conflict` variant 가 없어 `issue_claim` 의 CAS 거부가 `Error::Validation` → exit code 2 로 떨어진다.

**개선 방향**: 별도 후속 이슈로 `Error::Conflict(String)` variant 추가 + `issue_claim`/`issue_release` 의 Validation 메시지 일부 재분류.

### 6.5 `<owner>` placeholder

README / release.yml / plugin-setup.md 의 GitHub org 가 `<owner>` placeholder 로 남아 있어, 실제 2차 prebuilt 경로가 활성화되지 않은 상태로 demo 가 되었다.

**개선 방향**: Epic #5 의 P0 (#18) 에서 일괄 sed 치환 + 첫 v0.1.0 태그 푸시 → release artifact 생성.

## 7. 발견된 후속 작업 (별도 이슈 신설 권장)

1. `engram_core::Error::Conflict` variant 도입 + CAS 거부 재분류 (ADR-0010 §4 정합)
2. MCP `epic_list_backlog`, `epic_set_sprint` 의 `tool_definitions` 등록 (dispatch 분기만 있고 정의 없음 — 클라이언트에 보이지 않음)
3. MCP `note_get` / `note_resolve` 의 `note_id` → `id` 인자명 통일 (다른 도구와 일관)
4. Hook installer `PreToolUse:Bash` → `SessionStart` 매처 이전 (CLAUDE.md "알려진 한계" 항목)
5. License 결정 (`README.md` 의 "License: (TBD)")
6. ADR-0006/0007 번호 중복 정리 (`docs/adr/README.md`)
7. macOS notarization / code signing (Epic #5 P5 후속)
8. Windows release artifact (`release.yml` 매트릭스 미포함)
9. `note add --issue` default 0 → broadcast scope 미입력 시 NotFound UX 개선
10. **CLI 설치 마찰 제거 (Epic #5 등록 완료)** — P0 release artifact 생성 → P1 install.sh → P2 플러그인 install hook → P3 Homebrew tap → P4 cargo-binstall → P5 Tauri sidecar

## 8. 산출물 빠른 인덱스

- 매핑 SSOT: [`docs/cli-mcp-parity.md`](../cli-mcp-parity.md)
- ADR: [`docs/adr/0010-cli-mcp-parity.md`](../adr/0010-cli-mcp-parity.md), [`docs/adr/0011-cli-distribution.md`](../adr/0011-cli-distribution.md)
- 플러그인 setup: [`docs/plugin-setup.md`](../plugin-setup.md)
- 통합 테스트: [`crates/engram-cli/tests/parity_test.rs`](../../crates/engram-cli/tests/parity_test.rs)
- 출력 인프라: [`crates/engram-cli/src/output.rs`](../../crates/engram-cli/src/output.rs)
- CI: [`.github/workflows/release.yml`](../../.github/workflows/release.yml)
- 루트 README: [`README.md`](../../README.md)

## 9. 커밋 히스토리 (engram repo)

| 커밋 | 이슈 | 요약 |
|------|------|------|
| `7c9317c` | #11 | docs: 패리티 매트릭스 + ADR-0010 |
| `540cc87` | #12 | feat(cli): `--json` 출력 공통 인프라 |
| `4bb8212` | #13/#14 | feat(cli): MCP 패리티 verb + 신규 area |
| `e0384ec` | #16 | feat: 배포 자동화 (ADR-0011 + release.yml) |
| `f8f7763` | #15/#17 | test: 패리티 통합 테스트 + CLAUDE.md 갱신 |

플러그인 README + agents 프롬프트 3종 갱신은 `jake-marketplace` 별도 저장소.

## 10. 한 줄 결론

> 매트릭스 → ADR → 인프라 → 구현 → 검증 → 배포 → 문서 의 6단계 분리가 잘 작동했다.
> 다음 에픽(설치 자동화)에서도 같은 골격 유지.
