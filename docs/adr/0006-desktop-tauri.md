# ADR-0006: Desktop — Tauri v2 스택 채택

## Status
Accepted

## Context

Engram 은 로컬 SQLite DB 를 사용하는 단일 사용자 에이전트 이슈 관리 도구다. Phase 3 에서 CLI/MCP 에 더해 네이티브 데스크톱 앱을 추가하기로 했다. 선택지는 크게 Electron, Tauri v2, 순수 웹앱이었다.

## Decision

**Tauri v2** + React 18 + TypeScript + Tailwind CSS + shadcn/ui 컴포넌트 + dnd-kit DnD 를 채택한다.

UI 는 Vite 멀티엔트리 빌드(`main` + `tray`) 로 메인 윈도우와 트레이 팝오버를 분리한다.

## Consequences

- 단일 바이너리 배포 가능 (`cargo tauri build` → `.app` / `.exe`)
- native shell, file system, notification, tray 등 풍부한 plugin 생태계 활용
- MIT 라이선스, macOS/Windows 동시 지원
- Electron 대비 메모리 사용량 대폭 감소 (50–80 MB vs 150–300 MB)
- Webview 성능에 한계가 있으며, macOS WebKit 과 Windows WebView2 간 렌더링 차이 존재
- tauri-plugin-notification 등 일부 plugin 이 베타 상태 — API 변경 가능성

## Trade-offs

Electron 은 Chromium 번들로 렌더링 일관성이 높지만 메모리·디스크 비용이 크다.
순수 웹앱은 macOS tray 나 native notification 접근이 불가하다.
Tauri v2 는 이 중간 지점으로 단일 사용자 로컬 도구에 적합하다.
