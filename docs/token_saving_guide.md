# 대역폭 및 토큰 절약 가이드 (Token Saving Guide)

이 문서는 에이전트(LLM) 환경에서 대역폭을 보존하고 컨텍스트 토큰을 절약하기 위한 Engram API의 최적화 기능 사용법을 안내합니다.

Engram은 SQLite 기반의 단일 로컬 데이터베이스를 사용하지만, MCP(Model Context Protocol) 또는 CLI를 통해 대량의 이슈 설명(Description)이나 상세 내역(Detail)이 LLM의 컨텍스트 윈도우에 그대로 로드되면 비용 증가 및 성능 저하가 발생할 수 있습니다. 이를 방지하기 위해 다음과 같은 **Compact 모드**를 지원합니다.

---

## 1. MCP 도구 목록 컴팩트 조회 (`tools/list` compact)

LLM 에이전트가 처음에 사용 가능한 도구 목록을 조회할 때, 모든 도구의 `inputSchema`와 전체 `description`을 그대로 가져오면 매우 큰 컨텍스트를 소모하게 됩니다.

- **동작 방식**: `tools/list` 요청 시 `params.compact = true`를 넘겨주면, 도구 스키마의 `properties`와 `required` 필드가 제거되고, `description`은 마침표(`.`) 기준 첫 문장만 요약하여 반환됩니다.
- **JSON-RPC 예시**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {
      "compact": true
    }
  }
  ```

> [!TIP]
> 에이전트 부트스트랩 또는 툴 탐색 단계에서 툴 이름만 가볍게 파악하고자 할 때는 반드시 `compact: true` 옵션을 사용해 보세요.

---

## 2. 노트 본문 생략 목록 조회 (`note_list` include_detail)

노트는 회의록, 아키텍처 결정(ADR), 장애 원인(RCA) 등 장문의 상세 텍스트(`detail`)를 담고 있을 가능성이 높습니다.

- **동작 방식**: `note_list` API는 기본적으로 `include_detail = false` 상태로 동작합니다. 이 경우 요약(`summary`)과 메타데이터만 반환하여 텍스트 데이터 전송을 최소화합니다.
- **사용 방법**:
  - **MCP (`note_list`)**: `"include_detail": true`를 명시적으로 전달해야만 상세 본문(`detail`)이 로드됩니다.
  - **CLI (`engram note list`)**: 기본 목록에는 `detail`이 포함되지 않으며, 상세 정보 조회가 필요한 경우 `--include-detail` 플래그를 추가해야 합니다.
    ```bash
    # 기본 (compact) 조회
    engram note list --issue 42 --json
    
    # 상세 본문 포함 조회
    engram note list --issue 42 --include-detail --json
    ```

---

## 3. 단일 조회 컴팩트 모드 (`issue_get` / `note_get` compact)

단일 이슈나 노트를 상세 조회(`get`)할 때도 메타데이터(상태, 담당자, 관계)만 확인하고 싶다면 컴팩트 모드를 유용하게 활용할 수 있습니다.

### Issue Get (`issue_get`)
- **동작 방식**: `compact: true`인 경우, 장문의 기획서나 마일스톤이 기록되는 `description` 및 `goal` 필드를 `NULL`로 채워 반환합니다.
- **사용 방법**:
  - **MCP (`issue_get`)**:
    ```json
    {
      "id": 42,
      "compact": true
    }
    ```
  - **CLI (`engram issue get`)**:
    ```bash
    engram issue get 42 --compact --json
    ```

### Note Get (`note_get`)
- **동작 방식**: `compact: true`인 경우, `detail` 필드를 `NULL`로 채워 반환합니다.
- **사용 방법**:
  - **MCP (`note_get`)**:
    ```json
    {
      "id": 12,
      "compact": true
    }
    ```
  - **CLI (`engram note get`)**:
    ```bash
    engram note get 12 --compact --json
    ```

---

## 4. 모범 사례 (Best Practices)

> [!IMPORTANT]
> - **조회는 가볍게, 필요할 때만 상세하게**: 처음에 목록이나 대시보드를 뿌릴 때는 항상 `compact` 상태의 API나 기본 `note_list`를 이용하세요. 특정 이슈/노트의 상세 내용이 업무 진행에 필수적인 시점에만 `compact = false` 혹은 `include_detail = true` 옵션을 켜서 개별 조회하시기 바랍니다.
> - **에이전트 점유(Claim) 시**: 에이전트가 작업을 시작하기 위해 이슈를 점유(`claim`)하거나 상태를 변경할 때는 내부적으로 데이터의 정합성을 검증하기 위해 풀 레코드(`compact = false`)를 조회하므로 개발 시 유의해 주세요.
