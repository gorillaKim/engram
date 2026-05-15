# ADR-0008: 임베디드 MCP 서버 (같은 프로세스 tokio task)

## Status
Accepted

## Context
데스크톱 앱(engram-desktop)이 MCP 서버를 호스팅해야 한다. 두 가지 방안이 있다: ① 별도 프로세스(`engram-mcp` 바이너리)를 자식 프로세스로 실행, ② 같은 프로세스 내 tokio task로 HTTP 서버 기동.

## Decision
데스크톱 앱이 `engram_mcp::http::run_http_with_hook`을 직접 호출해 동일 프로세스 내 tokio task로 MCP HTTP 서버를 호스팅한다. `engram-mcp`를 lib + bin 듀얼 크레이트로 전환해 라이브러리 형태로 임베드한다.

## Consequences
- 단일 `Db` pool 공유 (WAL + max_connections=5 로 안전)
- graceful shutdown: oneshot 채널 하나로 HTTP 서버 중단 가능
- 같은 tokio runtime 사용 → context switch 오버헤드 없음
- 도구 호출이 CPU를 오래 점유하면 UI가 stall될 수 있음 → M3에서 30초 timeout 추가 예정
- 별도 프로세스 방안 대비: IPC/소켓 관리 불필요, 포트 충돌 감지 용이

## Trade-offs
별도 프로세스 방식은 crash isolation 이점이 있으나, SQLite WAL 환경에서 두 프로세스가 같은 DB를 동시에 쓰면 busy_timeout에 의존해야 해 불확실성이 높다. 임베디드 방식은 단일 writer를 보장하므로 더 안전하다.
