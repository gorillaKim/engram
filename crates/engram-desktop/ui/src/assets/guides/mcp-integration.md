# 🔌 MCP 및 AI 에이전트 연동 가이드

Engram은 개발 생산성을 극대화하기 위해 AI 에이전트와 직접 소통할 수 있는 **Model Context Protocol (MCP)** 서버를 기본 내장하고 있습니다. 
이 가이드에서는 에이전트 환경(Claude Desktop 등)에 Engram MCP를 등록하는 방법과 CLI 훅(Hook) 연동 방법을 다룹니다.

---

## 🌐 Model Context Protocol (MCP) 이란?

MCP는 거대 언어 모델(LLM) 에이전트가 로컬 개발 도구, 데이터베이스, 파일 시스템 등과 안전하게 데이터를 주고받을 수 있도록 규격화된 표준 개방형 프로토콜입니다. 
Engram은 45개 이상의 풍부한 MCP 도구(Tools)를 제공하여 에이전트가 이슈 조회, 태스크 체크, 세션 상태 갱신 등을 완벽히 자율적으로 제어할 수 있도록 돕습니다.

---

## 🛠️ Claude Desktop에 Engram MCP 서버 등록하기

Claude Desktop 앱에 Engram을 연동하려면, Claude 설정 파일(`claude_desktop_config.json`)을 열어 아래와 같이 `engram` MCP 서버 설정을 추가해 주어야 합니다.

### 설정 파일 위치
* **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
* **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

### JSON 설정 예시

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-mcp",
      "args": []
    }
  }
}
```

> [!TIP]
> 만약 `engram-mcp`가 전역 path에 등록되어 있지 않다면, 빌드된 절대 경로를 적어주거나 `cargo run`을 통하도록 지정할 수 있습니다.
> ```json
> "command": "cargo",
> "args": ["run", "--release", "--manifest-path", "/Users/사용자/gorillaProject/engram/crates/engram-mcp/Cargo.toml"]
> ```

설정을 완료한 뒤 Claude Desktop을 재시작하면, 채팅 창 우측 하단에 플러그 플러그인 아이콘이 활성화되며 `issue_create`, `task_update` 등의 Engram 전용 도구들을 에이전트가 인지하고 사용할 수 있게 됩니다.

---

## 🪝 CLI 훅(Hook) 연동 및 설정

Engram CLI는 Git 커밋 및 툴 체인 호출 시 세션을 자동으로 연동해 주는 **Codex Hook** 기술을 지원합니다. 
이 훅을 프로젝트에 설치하면 사용자가 터미널에서 작업 세션을 열거나 닫을 때, 혹은 에이전트가 작업을 수행하기 전후로 데이터의 스냅샷을 자동 동기화할 수 있어 협업 데이터의 충돌을 예방합니다.

### 훅 설치 명령어 (프로젝트 루트 디렉토리에서 실행)
```bash
# 훅 설치
engram hook install

# 훅 상태 및 세션 검사
engram hook post-session-check

# 훅 비활성화
engram hook uninstall
```

설치가 정상 완료되면 `.git/hooks/` 폴더 내에 Engram 연동을 위한 트리거 쉘 스크립트들이 배치되어 백그라운드에서 유기적으로 데이터 싱크 작업을 지원하게 됩니다.
