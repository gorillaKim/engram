# Rule: 새 MCP 도구 추가 절차

## 변경되는 파일 (반드시 3곳)

1. `crates/engram-mcp/src/tools/<area>.rs`
   - `tool_definitions() -> Vec<Value>` 에 새 도구의 JSON 정의 push
   - `pub async fn <name>(db: Arc<Db>, args: &Value) -> engram_core::Result<Value>` 핸들러 추가
2. `crates/engram-mcp/src/tools/mod.rs`
   - `dispatch()` 의 `match name` 에 `"<tool_name>" => <area>::<name>(db, args).await,` 분기 추가
3. (선택) `crates/engram-cli/src/commands/<area>.rs` — 같은 동작을 CLI 로도 노출할 때

`all_tool_definitions()` 는 각 `tool_definitions()` 를 `.concat()` 하므로, 새 area 모듈을 추가했다면 거기에도 등록한다.

## JSON 스키마 컨벤션

- **필드명은 camelCase**: `inputSchema` (snake_case 아님). MCP 사양 준수.
- 도구 정의 형식 (json! 매크로):

  ```rust
  json!({
      "name": "epic_create",
      "description": "...",
      "inputSchema": {
          "type": "object",
          "required": ["sprint_id", "project_key", "title"],
          "properties": {
              "sprint_id":   { "type": "integer" },
              "project_key": { "type": "string" },
              "title":       { "type": "string" },
              "description": { "type": "string" }
          }
      }
  })
  ```

- 도구 이름은 `<area>_<verb>` snake_case (예: `issue_link`, `task_next`).
- **agent_id 필수성 (ADR-0010 보강)**: 상태/소유권/노트를 변경하는 모든 MCP 도구는 `inputSchema`의 `required` 배열에 `"agent_id"`를 필수 포함해야 합니다. (조회 도구는 optional 유지)

## description 작성 규칙

도구 description **문장이 Agent의 호출 의사결정을 좌우**한다.

- 한국어 1~3문장, 명령형.
- 언제 호출할지를 명확히: "세션 시작 시 반드시 호출", "in_progress 태스크가 있을 때 호출".
- 강제 언어 OK: "반드시", "항상", "필수".
- 입력 의미 / 출력 형태를 한 줄로 함께 적는 게 좋다.

예시:
```
세션 시작 시 반드시 호출하세요. 현재 활성 스프린트의 에픽/이슈 진행 현황,
미완료 태스크, 주의사항(caveat) 목록, 다음 처리할 태스크를 반환합니다.
project_key를 지정하면 해당 프로젝트 컨텍스트만 반환합니다.
```

## 핸들러 시그니처

- `engram_core::Result<Value>` 반환. 도메인 객체는 `serde_json::to_value(&x).unwrap()` 으로 직렬화.
- 입력 파싱은 `args["field"].as_str() / as_i64() / as_bool()` 직접 접근. struct 디시리얼라이즈는 입력 형태가 안정될 때 도입.
- 없거나 잘못된 필수 필드는 `Error::Validation(...)` 으로 명시 반환 (panic / unwrap 금지).
- 도메인 에러는 `engram_core::Error` 그대로 전파 — `server.rs` 의 `handle_tools_call` 이 JSON-RPC error code `-32000` 로 매핑한다.

## 응답 직렬화

- `handle_tools_call` 은 결과를 `[{ "type": "text", "text": <pretty json> }]` 로 래핑한다. 핸들러는 그냥 `Value` 만 돌려주면 된다.
- 큰 응답 (예: `session_restore`)은 sub-struct 를 만들어 `derive(Serialize)` — `repository/session.rs::SessionSnapshot` 패턴 참조.

## 통합 테스트 동반 추가

새 도구를 추가하면 `crates/engram-core/tests/workflow_test.rs` (없으면 신규 생성) 에 해당 도구를 사용하는 워크플로 시나리오를 한 가지 이상 추가한다 — `.claude/rules/testing-strategy.md` 참조.

## 변경 후 확인

- `cargo build -p engram-mcp` 통과
- `tools/list` 응답에 새 도구가 포함되는지 수동 확인:
  ```bash
  echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run -p engram-mcp
  ```
