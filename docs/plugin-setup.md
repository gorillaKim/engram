# Engram 플러그인 (`engram-orchestrator`) Setup 가이드

이 문서는 `engram-orchestrator` 플러그인의 서브에이전트(worker / leader / analyzer / orchestrator) 가 Engram CLI/MCP 에 접근하기 위해 사용자가 1회 수행해야 할 setup 절차를 정리한다.

본 repo (`engram`) 는 CLI 바이너리 (`engram`) + MCP 서버 (`engram-mcp`) 만 호스팅한다. 플러그인 패키지 자체는 별도 저장소에 있다.

## 0. 호환성 매트릭스

| 항목                | 권장        |
|--------------------|-------------|
| Rust toolchain     | stable ≥ 1.75 (1차 설치 경로용)  |
| engram CLI version | **>= 0.1.0** (현 workspace 버전) |
| OS                 | macOS arm64 / macOS x64 / linux x64 (prebuilt 매트릭스) |

서브에이전트는 setup 시 `engram --version` 으로 호환 가능 버전인지 확인하고, 미달 시 사용자에게 재설치 안내.

## 1. CLI 설치

### 1.1 1차 권장 — `cargo install`

```bash
cargo install --git https://github.com/<owner>/engram engram-cli
# 또는 로컬 클론 후
cargo install --path crates/engram-cli
```

확인:

```bash
engram --version    # → engram 0.1.0
which engram        # $PATH 에서 위치 확인
```

### 1.2 2차 — prebuilt binary (Rust toolchain 없는 환경)

`.github/workflows/release.yml` 가 태그 push 시 자동 빌드. 사용자 OS 에 맞는 asset 다운로드:

```bash
VER=0.1.0   # 원하는 release version
# macOS Apple Silicon
curl -L "https://github.com/<owner>/engram/releases/download/v${VER}/engram-${VER}-aarch64-apple-darwin.tar.gz" | tar xz
# macOS Intel
curl -L "https://github.com/<owner>/engram/releases/download/v${VER}/engram-${VER}-x86_64-apple-darwin.tar.gz"  | tar xz
# linux x64
curl -L "https://github.com/<owner>/engram/releases/download/v${VER}/engram-${VER}-x86_64-unknown-linux-gnu.tar.gz" | tar xz

# $PATH 안의 디렉토리에 배치
mv engram /usr/local/bin/      # 또는 ~/.local/bin/
engram --version
```

자세한 결정 근거: [ADR-0011](adr/0011-cli-distribution.md).

## 2. Claude Code Hook 등록 (선택)

세션 시작 시 자동으로 `engram session restore` 컨텍스트를 inject 하려면:

```bash
engram hook install
```

`~/.claude/settings.json` 에 hook 이 등록된다. 제거:

```bash
engram hook uninstall
```

각 프로젝트의 `CLAUDE.md` 에 다음을 추가:

```
## Engram
project_key: <your-project-name>
```

## 3. 서브에이전트의 호출 패턴

플러그인 서브에이전트는 다음 두 경로 중 가용한 쪽을 사용:

1. **MCP 도구 호출** (Agent SDK 가 tool whitelist 로 노출) — 1차 권장.
2. **CLI shell 호출** — MCP 가 막힌 환경의 fallback.

CLI fallback 패턴 (ADR-0010 + 매트릭스):

```bash
# 세션 시작
engram session restore --project myproj --json

# 이슈 잡기 (CAS 안전)
engram issue claim 12 --agent-id "$AGENT_ID" --json

# 작업 진행 후 demo 로 해제
engram issue release 12 --agent-id "$AGENT_ID" --transition-to demo --json

# 정체 이슈 감시 (leader)
engram stalled --threshold-minutes 10 --json

# 변경 이력 조회 (감사)
engram history recent --since-minutes 30 --json
engram history by-agent --agent-id "$AGENT_ID" --limit 20 --json

# 노트 (issue/task/broadcast)
engram note add --issue 12 --type context --summary "..." --agent-id "$AGENT_ID" --json
engram note add --scope epic --scope-target-id 4 --type decision --summary "..." --agent-id "$AGENT_ID" --json
```

전체 verb 매트릭스: [`docs/cli-mcp-parity.md`](cli-mcp-parity.md).

## 4. exit code 처리

ADR-0010 §4 에 따라:

| exit | 의미                                       | 서브에이전트 권장 동작                |
|------|--------------------------------------------|--------------------------------------|
| 0    | 성공                                       | 다음 단계 진행                       |
| 1    | DB/Migration/기타 anyhow                  | stop & report                        |
| 2    | Validation (입력 오류)                     | 인자 수정 후 재시도                  |
| 3    | NotFound                                  | 대상 ID 재조회 후 결정                |
| 4    | InvalidTransition (CAS 거부 포함)         | 다른 이슈로 전환 또는 backoff 후 재시도 |

`--json` 모드에서는 stderr 에 `{"error":{"code":"...","message":"..."}}` 가 함께 emit.

## 5. 트러블슈팅

- `engram: command not found` → `which engram` 으로 위치 확인. cargo install 시 `~/.cargo/bin` 이 `$PATH` 에 있는지 확인. prebuilt 설치 시 mv 경로 확인.
- `engram --version` 이 예상과 다름 → 다른 버전이 PATH 에서 먼저 잡힘. `which -a engram` 으로 중복 확인.
- DB 위치: `~/.engram/engram.db` (ADR-0001). 손상 시 `mv ~/.engram/engram.db ~/.engram/engram.db.bak` 후 재시작.
- Hook 등록 후 동작 안 함 → `~/.claude/settings.json` 의 `hooks.PreToolUse[].command` 가 `engram snapshot-text` 인지 확인. `engram hook uninstall && engram hook install` 로 재등록.
