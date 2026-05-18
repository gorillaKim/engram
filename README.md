# Engram

Agent Issue Management System — Sprint / Epic / Issue / Task / Note 를 SQLite 에 저장하고
**MCP 서버 (stdio JSON-RPC) + CLI** 로 노출한다.

- 설계: `doxus://brain/Ideas/agent/Engram - Agent Issue Management System.md`
- 구현 계획: `doxus://brain/Ideas/agent/Engram - Implementation Plan.md`
- 코딩 규칙: [`.claude/rules/`](.claude/rules/)
- 설계 결정: [`docs/adr/`](docs/adr/)
- 에픽 회고: [`docs/retro/`](docs/retro/)
- CLI ↔ MCP 패리티 매트릭스: [`docs/cli-mcp-parity.md`](docs/cli-mcp-parity.md)
- 플러그인 setup 가이드: [`docs/plugin-setup.md`](docs/plugin-setup.md)

## Installation

### 1차 권장 — `cargo install` (Rust toolchain 보유 시)

```bash
# 원격 git 에서 직접 설치
cargo install --git https://github.com/gorillaKim/engram engram-cli

# 또는 로컬 클론 후 설치
git clone https://github.com/gorillaKim/engram
cd engram
cargo install --path crates/engram-cli
```

설치되는 binary 이름은 `engram` (workspace `[[bin]] name = "engram"`).
설치 후 확인:

```bash
engram --version    # → engram <version>
engram --help
```

### 1차 권장 — Homebrew (macOS)

macOS 사용자는 brew를 통해 가장 쉽고 안전하게 설치할 수 있습니다.

```bash
brew tap gorillaKim/engram
brew install engram
```

### 2차 권장 — `install.sh` (curl-pipe)

한 줄 명령으로 OS/Arch 감지 및 최신 바이너리 설치가 가능합니다.

```bash
curl -fsSL https://raw.githubusercontent.com/gorillaKim/engram/main/install.sh | sh
```

### 3차 — prebuilt binary (GitHub Releases)

태그 push 시 `.github/workflows/release.yml` 가 자동 빌드. 지원 타겟:

- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-unknown-linux-gnu` (linux x64)

설치 예시 (macOS Apple Silicon):

```bash
VER=0.1.0   # 원하는 release version
curl -L "https://github.com/gorillaKim/engram/releases/download/v${VER}/engram-${VER}-aarch64-apple-darwin.tar.gz" \
  | tar xz
mv engram /usr/local/bin/   # 또는 ~/.local/bin/ 등 PATH 에 있는 곳
engram --version
```

자세한 배포 결정은 [ADR-0011: CLI 배포 경로](docs/adr/0011-cli-distribution.md) 참조.

## Quick Start

```bash
# 스프린트 생성 + 활성화
engram sprint create --name "Sprint #1" --goal "engram 완성하기"
engram sprint update 1 --status active

# 에픽 + 이슈 + 태스크
engram epic create --project myproj --title "MVP"
engram issue create --epic 1 --sprint 1 --title "첫 이슈"
engram task create --issue 1 --title "구현"

# 세션 복원 (서브에이전트 진입 시 호출)
engram session restore --project myproj --json

# 칸반 보드 / 블로킹 / 정체 / 변경 이력
engram board status --project myproj
engram blocked list --project myproj
engram stalled --threshold-minutes 10
engram history recent --since-minutes 60
```

모든 명령은 `--json` 글로벌 플래그로 머신 파싱 가능한 출력을 낸다 (서브에이전트용).
명령형/인자/exit code 컨벤션은 [ADR-0010](docs/adr/0010-cli-mcp-parity.md) 참조.

## MCP 서버 모드

CLI 와 동일한 도메인을 stdio JSON-RPC 로도 노출한다. Claude Code 등 MCP 클라이언트가 직접 붙는 경로:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | engram-mcp
```

(서브에이전트가 MCP 서버에 못 붙는 환경에서는 위 CLI 가 fallback. CLI ↔ MCP 동일성은 ADR-0010 + 매핑 매트릭스 보장.)

## Claude Code Hook 통합

```bash
engram hook install       # ~/.claude/settings.json 에 hook 등록
engram hook uninstall
```

각 프로젝트의 `CLAUDE.md` 에 `project_key: <name>` 을 명시하면 hook 이 session_restore 컨텍스트를 자동 출력.

## Workspace 구조

```
engram/
├── crates/
│   ├── engram-core/   ← 도메인 모델 + sqlx SQLite repository (의존성 0)
│   ├── engram-mcp/    ← JSON-RPC stdio MCP 서버
│   ├── engram-cli/    ← clap 기반 CLI + Hook 통합
│   └── engram-desktop/← Tauri v2 데스크톱 (Phase 3)
├── migrations/        ← engram-core/migrations/NNNN_*.sql
├── docs/adr/          ← Architecture Decision Records
└── .claude/rules/     ← 작업 시 참조할 코딩 규칙
```

상세는 `CLAUDE.md` 참조.

## Development

```bash
cargo build              # 전체 빌드
cargo test --workspace   # 전체 테스트
cargo run -p engram-cli -- sprint list      # CLI 실행
echo '<json>' | cargo run -p engram-mcp     # MCP stdio 수동 시험
```

마이그레이션은 `Db::open` 안에서 자동 적용. 별도 CLI 호출 불필요.

## License

(TBD)
