# ADR-0006: Streamable HTTP Transport (SSE 폐기)

## Status
Accepted

## Context
Phase 2 초기 구현에서 SSE(Server-Sent Events) 전송을 사용했다. 그러나 MCP 사양(2025-03-26 이후)에서 SSE는 deprecated 되었고 Streamable HTTP가 표준 전송으로 채택되었다. Claude Code, claude.ai 웹 등 주요 클라이언트들이 Streamable HTTP를 요구한다.

## Decision
`sse.rs`를 제거하고 `http.rs`(Streamable HTTP transport)로 대체한다. axum 기반 구현은 유지하되 `/mcp` 단일 엔드포인트로 통합한다. `GET /mcp`는 서버→클라이언트 알림용으로 M3까지 `405 Method Not Allowed`를 반환한다. SO_REUSEADDR을 적용해 포트 즉시 재사용이 가능하게 한다.

## Consequences
- Claude Code의 `"type": "http"` MCP 설정으로 바로 연결 가능
- SSE 관련 복잡한 session 채널 관리 코드 제거
- GET 스트리밍은 M3 Supervisor 구현 시 추가 예정
