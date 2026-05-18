# ADR-0011: CLI 배포 경로 — cargo install 정식 + GitHub Releases prebuilt

## Status
Accepted

## Context
ADR-0010 의 패리티 완성으로 `engram` CLI 는 MCP 도구 45 개 전부를 동등 호출 가능한 단일 바이너리가 됐다. 이제 `engram-orchestrator` 플러그인의 서브에이전트(worker/leader/analyzer 등) 가 Agent SDK tool whitelist 로 MCP 도구를 못 받거나 stdio MCP 서버에 못 붙는 환경에서도 `engram <area> <verb> --json ...` 로 동일한 워크플로를 수행할 수 있어야 한다. 그러려면 사용자/플러그인 설치 시 `engram` 이 `$PATH` 에서 잡혀야 한다. 후보 경로:

1. `cargo install --path crates/engram-cli` (또는 `cargo install --git ...`) — 사용자가 Rust toolchain 보유 시 가장 단순.
2. GitHub Releases 의 prebuilt binary (macOS arm64/x64, linux x64) — Rust toolchain 없이도 한 줄 다운로드.
3. 플러그인 install hook 에서 위 둘 중 하나를 자동 실행 — UX 친화적이지만 권한·실패 처리 복잡.
4. npm postinstall — 플러그인이 npm 으로 배포될 때 자연스럽지만 본 repo 는 npm package 가 아니라 비대칭.

Homebrew tap 은 향후 별도 고려 (본 ADR 범위 외, #16 description 도 비목표로 명시).

## Decision

1. **1차 (즉시 지원): `cargo install`**.
   - 정식 명령: `cargo install --git https://github.com/<owner>/engram engram-cli` (workspace 안의 `engram-cli` 크레이트 명).
   - 로컬 개발: `cargo install --path crates/engram-cli`.
   - 설치되는 바이너리 이름은 `engram` (Cargo.toml 의 `[[bin]] name = "engram"`).
   - 모든 README/플러그인 setup 문서가 이 경로를 1차 권장.
2. **2차 (Release 워크플로 도입): GitHub Releases prebuilt binary**.
   - `.github/workflows/release.yml` 가 태그 (`v*`) push 시 다음 타겟을 동시 빌드해 Release asset 으로 업로드:
     - `aarch64-apple-darwin` (macOS Apple Silicon)
     - `x86_64-apple-darwin` (macOS Intel)
     - `x86_64-unknown-linux-gnu` (linux x64)
   - asset 파일명 규칙: `engram-<version>-<target>.tar.gz` (내부 binary 이름 `engram`).
   - 사용자는 `curl -L https://github.com/<owner>/engram/releases/download/v<ver>/engram-<ver>-aarch64-apple-darwin.tar.gz | tar xz` 로 단일 바이너리 추출 후 `$PATH` 에 배치.
3. **플러그인 install hook 자동 설치는 본 ADR 범위 외**.
   - 이유: 사용자 환경마다 `$PATH` 권한·디렉토리 정책이 달라 자동 설치 실패 처리가 무거워진다. 플러그인 README 가 1차/2차 경로 두 가지를 안내하고 사용자 선택에 맡긴다.
4. **npm postinstall 는 채택하지 않음**.
   - 본 repo 는 Rust workspace 이며 npm package 가 아니다. 플러그인(`engram-orchestrator`) 이 npm 으로 배포될 때 그쪽 postinstall 이 cargo install 또는 prebuilt fetch 를 호출하는 건 플러그인 측 책임 — 본 repo 의 결정 영역 아님.
5. **버전 호환성 보장**.
   - `engram --version` 은 `Cargo.toml` 의 workspace version 을 그대로 출력 (clap `#[command(version)]` 매크로가 `CARGO_PKG_VERSION` 사용). 검증: `engram --version` → `engram 0.1.0` (현재 workspace version).
   - 플러그인 setup 문서가 호환 가능한 최소 버전을 명시 (예: `engram >= 0.1.0`). 서브에이전트는 setup 시 `engram --version` 으로 version 비교 후 미달이면 안내.

## Consequences
- 사용자는 cargo toolchain 보유 여부에 따라 1차(cargo install) 또는 2차(prebuilt) 중 선택. 두 경로 모두 README 의 "Installation" 섹션에서 안내된다.
- Release 워크플로가 도입되면 매 태그 push 마다 3 타겟이 빌드·업로드된다. CI 시간 비용은 워크플로 캐시(`Swatinem/rust-cache`) 로 완화.
- 플러그인 측은 사용자에게 "engram 을 먼저 설치하세요 (cargo install 또는 prebuilt)" 만 안내하면 되어 install hook 디버깅 부담이 사라진다.
- `--version` 의 workspace inherit 가 보장되므로 미래 버전업 시 호환 검증이 단순 string 비교로 가능.

## Trade-offs
- npm postinstall 자동화를 안 함 → 플러그인 사용자가 수동으로 한 단계 더 거쳐야 한다. 대신 권한·네트워크 오류 표면이 사용자에게 보여 디버깅이 명확.
- prebuilt binary 매트릭스에 Windows 는 포함 안 함 (현재 사용자 베이스가 macOS/Linux 중심이라는 판단). 필요 시 별도 ADR 로 추가.
- Homebrew tap 은 향후 별도 ADR. tap 도입 전까지는 macOS 사용자도 cargo install 또는 Release tar 사용.
