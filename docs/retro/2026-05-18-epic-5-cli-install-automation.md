# Epic #5 회고 — CLI 설치 자동화 (서브에이전트 마찰 제거)

- **일자**: 2026-05-18
- **스프린트**: #2 ("engram 만들기")
- **에픽**: #5 — CLI 설치 자동화 — 서브에이전트 마찰 제거
- **상태**: 4/6 finished (P0~P3 완료, P4~P5 취소)
- **세션 ID**: 870e2815-5d9c-461d-8f92-56780c9e1234 (예시)

## 1. 동기

Epic #4 를 통해 CLI 가 MCP 와 동등한 기능을 갖추었으나, 서브에이전트(worker/leader 등)가 실제로 CLI 를 활용하기에는 설치 장벽이 높았다. Rust 툴체인이 없는 환경에서도 서브에이전트가 `engram` 명령어를 즉시 호출할 수 있도록 **"원클릭/자동 설치"** 인프라를 구축하는 것이 본 에픽의 핵심 목표였다.

## 2. 결과 요약

| 항목 | 상태 / 수치 |
|------|------------|
| 첫 정식 릴리스 | v0.1.0 (tag) ✓ |
| 배포 플랫폼 | macOS (arm64/x64), Linux (x64) |
| 신규 설치 채널 | 2종 (curl-pipe, Homebrew) |
| Placeholder 제거 | `<owner>` → `gorillaKim` 일괄 치환 |
| 자동 설치 Hook | `engram-orchestrator` 연동 설계 완료 |
| 취소 이슈 | 2건 (P4: binstall, P5: Tauri sidecar) |

## 3. 이슈 단위 정리

| # | 제목 | 핵심 산출 |
|---|------|----------|
| #18 | P0 — Release artifact 생성 | `<owner>` → `gorillaKim` 치환, `v0.1.0` 태그 푸시, GitHub Release 아티팩트 생성 |
| #19 | P1 — install.sh (curl-pipe) | 루트 `install.sh`, README 1줄 가이드 (`curl ... | sh`) |
| #20 | P2 — 플러그인 install hook | `docs/draft-postinstall-hook.sh` 작성 및 orchestrator 플러그인 연동 가이드 |
| #21 | P3 — Homebrew tap & Formula | `Formula/engram.rb`, `engram.rb` (tap 저장소용), sha256 검증 완료 |
| #22 | P4 — cargo-binstall (취소) | v0.1.0 범위에서 제외 (crates.io publish 의존성으로 인해 후속 이관) |
| #23 | P5 — Tauri sidecar (취소) | 데스크톱 앱 배포 전략 수정으로 인해 취소 |

## 4. 핵심 결정 및 변경 사항

### 4.1 릴리스 자동화 인프라 확정
- `.github/workflows/release.yml` 을 활성화하여 태그 푸시 시 자동으로 macOS 및 Linux 용 바이너리를 빌드하고 tar.gz 아티팩트를 생성하도록 설정했다.
- GitHub Runner 의 `macos-13` 지원 종료(retirement) 문제를 발견하고 `macos-latest`로 즉시 업데이트하여 CI 안정성을 확보했다.

### 4.2 설치 편의성 극대화 (install.sh)
- `uname` 기반의 자동 아키텍처 감지와 `~/.local/bin` 설치를 지원하는 멱등성(idempotent) 있는 설치 스크립트를 도입했다. 이를 통해 `cargo install` 없이도 5초 내외로 CLI 환경 구축이 가능해졌다.

### 4.3 Homebrew 생태계 편입
- `gorillaKim/homebrew-engram` 탭을 통해 macOS 사용자에게 가장 친숙한 설치 경로를 제공했다. 이는 단순 바이너리 배포를 넘어 프로젝트의 공식적인 신뢰도를 높이는 계기가 되었다.

## 5. 잘 된 점

1. **Placeholder 조기 제거**: 에픽 시작과 동시에 `<owner>`를 실제 식별자로 치환하여, 이후 작성된 모든 스크립트와 문서의 링크가 즉시 동작할 수 있었다.
2. **CI 문제의 빠른 인지 및 해결**: `macos-13` 러너 이슈로 빌드가 실패했을 때, 원인을 정확히 파악하고 `macos-latest`로 전환하여 릴리스 일정을 준수했다.
3. **설치 스크립트의 멱등성**: 이미 설치된 버전을 확인하여 불필요한 다운로드를 방지하고, PATH 미설정 시 안내 메시지를 출력하는 등 사용자 경험(UX)에 집중했다.

## 6. 어려웠던 점 / 개선 여지

### 6.1 crates.io Publish 의존성 (P4 취소 원인)
- `cargo binstall` 지원을 위해 crates.io publish를 시도했으나, 워크스페이스 내 의존성 구조(`engram-core` → `engram-mcp` → `engram-cli`)로 인해 순차적 publish 과정이 복잡하여 이번 에픽에서는 제외되었다. 
- **교훈**: 외부 패키지 매니저 등록은 내부 의존성 정리가 선행되어야 함을 확인했다.

### 6.2 바이너리 공증(Notarization) 부재
- 현재 배포된 macOS 바이너리는 공증되지 않아, 최초 실행 시 사용자가 보안 설정을 수동으로 해제해야 하는 불편함이 남아 있다.
- **개선 방향**: 다음 릴리스 주기에서 Apple Developer 계정을 통한 Notarization 단계를 CI에 통합해야 한다.

## 7. 발견된 후속 작업

1. **macOS Notarization & Code Signing**: macOS 사용자의 실행 차단 문제를 근본적으로 해결.
2. **Windows(msi/exe) 배포 지원**: 현재 Linux/macOS에 국한된 release 매트릭스를 Windows까지 확장.
3. **`engram-orchestrator` 실제 Hook 적용**: 설계된 postinstall 스크립트를 실제 플러그인 배포 판에 병합.
4. **crates.io 공식 게시**: `engram-cli` 패키지 등록 및 `cargo binstall` 공식 지원.

## 8. 산출물 빠른 인덱스

- 설치 스크립트: [`install.sh`](../../install.sh)
- Homebrew Formula: [`Formula/engram.rb`](../../Formula/engram.rb)
- 릴리스 워크플로: [`.github/workflows/release.yml`](../../.github/workflows/release.yml)
- 갱신된 설치 가이드: [`README.md`](../../README.md), [`docs/plugin-setup.md`](../plugin-setup.md)
