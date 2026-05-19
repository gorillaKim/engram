# Engram CLI 설치 가이드

Engram CLI (`engram`) 는 단일 바이너리로 MCP 도구 45개와 동일한 기능을 셸에서 직접 호출할 수 있다.

## 호환성

| 항목 | 지원 |
|------|------|
| macOS Apple Silicon (arm64) | ✅ |
| macOS Intel (x86_64) | ✅ |
| Linux x64 | ✅ |
| Windows | ❌ (미지원) |
| 최소 engram 버전 | 0.1.0 |
| Rust toolchain (cargo install 경로만) | stable ≥ 1.75 |

---

## 설치 방법

### 방법 1 — Homebrew (macOS 권장)

```bash
brew tap gorillaKim/engram
brew install engram
```

### 방법 2 — curl 원라이너

OS와 아키텍처를 자동 감지해 `~/.local/bin/engram`에 설치한다.

```bash
curl -fsSL https://raw.githubusercontent.com/gorillaKim/engram/main/install.sh | sh
```

PATH에 `~/.local/bin`이 없으면 스크립트가 경고를 출력한다. 그때는 셸 설정에 추가:

```bash
# ~/.zshrc 또는 ~/.bashrc
export PATH="$PATH:$HOME/.local/bin"
```

### 방법 3 — cargo install (Rust toolchain 보유 시)

```bash
# 원격 저장소에서 직접
cargo install --git https://github.com/gorillaKim/engram engram-cli

# 로컬 클론 후
git clone https://github.com/gorillaKim/engram && cd engram
cargo install --path crates/engram-cli
```

설치 위치: `~/.cargo/bin/engram` — cargo 설치 시 자동으로 PATH에 추가된다.

### 방법 4 — prebuilt binary 수동 설치

[GitHub Releases](https://github.com/gorillaKim/engram/releases)에서 OS에 맞는 파일을 받는다.

```bash
VER=0.1.0

# macOS Apple Silicon
curl -L "https://github.com/gorillaKim/engram/releases/download/v${VER}/engram-${VER}-aarch64-apple-darwin.tar.gz" | tar xz

# macOS Intel
curl -L "https://github.com/gorillaKim/engram/releases/download/v${VER}/engram-${VER}-x86_64-apple-darwin.tar.gz" | tar xz

# Linux x64
curl -L "https://github.com/gorillaKim/engram/releases/download/v${VER}/engram-${VER}-x86_64-unknown-linux-gnu.tar.gz" | tar xz

# PATH에 있는 디렉토리로 이동
mv engram ~/.local/bin/
```

---

## 설치 확인

```bash
engram --version    # → engram 0.1.0
engram --help
```

---

## Claude Code Hook 등록 (선택)

세션 시작 시 `engram session restore` 컨텍스트를 자동으로 inject 하려면:

```bash
engram hook install
```

`~/.claude/settings.json`에 hook이 등록된다. 각 프로젝트의 `CLAUDE.md`에 다음을 추가:

```
## Engram
project_key: <your-project-name>
```

제거:

```bash
engram hook uninstall
```

---

## 트러블슈팅

**`engram: command not found`**
- cargo install 경로: `~/.cargo/bin`이 PATH에 있는지 확인
- prebuilt/install.sh 경로: `~/.local/bin`이 PATH에 있는지 확인
- `which engram`으로 설치 위치 확인

**버전이 예상과 다름**
- `which -a engram`으로 PATH 상 중복 바이너리 확인
- 오래된 버전을 먼저 제거 후 재설치

**DB 초기화가 필요할 때**
- DB 위치: `~/.engram/engram.db`
- 백업 후 삭제: `mv ~/.engram/engram.db ~/.engram/engram.db.bak`
- 다음 실행 시 자동으로 새 DB 생성

**Hook이 동작하지 않을 때**
```bash
engram hook uninstall && engram hook install
```

---

## 관련 문서

- [ADR-0011: CLI 배포 경로 결정](adr/0011-cli-distribution.md)
- [CLI ↔ MCP 패리티 매트릭스](cli-mcp-parity.md)
- [플러그인 Setup 가이드](plugin-setup.md)
