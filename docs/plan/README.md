# Engram Desktop & Tray Widget — 구현 계획

Engram 데스크톱 앱(Tauri v2) + 메뉴바 트레이 위젯 + 임베디드 Streamable HTTP MCP 서버 구현 계획을 마일스톤별로 분리해 보관한다.

## 문서 구조

| 문서 | 내용 |
|---|---|
| [overview.md](./overview.md) | 전체 비전 · 아키텍처 · 디자인 시안 · Tauri command 명세 · 검증 시나리오 |
| [m0-foundations.md](./m0-foundations.md) | **선행 정비** (2~3일). `engram-mcp` lib 분리, SSE→HTTP 전환, `changed_by` 파라미터, pool 사이즈 |
| [m1-scaffold-board.md](./m1-scaffold-board.md) | 스캐폴딩 + 보드 읽기 (1주). Tauri 부트, 칸반 5컬럼 read-only |
| [m2-dnd-drawer.md](./m2-dnd-drawer.md) | DnD + Drawer + Finished 필터 (1주). dnd-kit, Issue Detail, hide-finished 토글 |
| [m3-mcp-supervisor.md](./m3-mcp-supervisor.md) | 임베디드 MCP Supervisor (1주). 시작/정지/재시작, 로그/호출 이력, McpManager UI |
| [m4-tray-notifications.md](./m4-tray-notifications.md) | 메뉴바 트레이 + 알림 (3~4일). 진행률 바, NSNotificationCenter 푸시 |
| [m5-polish.md](./m5-polish.md) | 폴리시 (3~4일). 고급 필터, BlockingGraph 시각화, ADR/규칙 정리 |

## 마일스톤 순서

```
M0 ──► M1 ──► M2 ──► M3 ──► M4 ──► M5
선행    보드    DnD    MCP     트레이   폴리시
정비    읽기    Drawer Super.   알림    
```

M0 는 다른 모든 마일스톤이 의존하므로 **별도 PR** 로 먼저 머지한다.

## 총 예상

**4.5~5주** (1인 풀타임 기준)

## 참고

- 합의된 결정 사항: 사용자가 `AskUserQuestion` 으로 확정한 UI 스택(Tailwind+shadcn/ui+dnd-kit), Demo Gate 정책(서브에이전트 프롬프트+rules 만), 위젯 형태(macOS 메뉴바 트레이).
- MCP 전송 방식: **Streamable HTTP** (SSE 는 MCP 사양상 deprecated).
- Opus 아키텍트 리뷰 반영 사항: `changed_by` 파라미터 강제, `SqlitePool` max_connections 명시, graceful shutdown 신호 라우팅, M0 선행 분리.
