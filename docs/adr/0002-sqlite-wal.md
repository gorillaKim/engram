# ADR-0002: SQLite + WAL Mode

## Status
Accepted

## Context
Engram은 로컬 개인 도구다. 별도 서버 프로세스 없이 단일 바이너리로 동작해야 한다. MCP 서버(백그라운드 프로세스)와 CLI, 향후 Tauri 데스크톱 앱이 동시에 같은 DB에 접근할 수 있어야 한다. PostgreSQL 같은 클라이언트-서버 DB는 설치·운영 부담이 있고, 개인 로컬 도구의 규모에 맞지 않는다.

## Decision

SQLite를 스토리지 엔진으로 사용한다. WAL(Write-Ahead Logging) 모드를 활성화해 reader와 writer가 서로를 블로킹하지 않도록 한다. `busy_timeout = 5000ms`로 설정해 write 경합 시 최대 5초 재시도한다. PostgreSQL은 사용하지 않는다.

## Consequences

- 긍정: 서버 프로세스 불필요, 단일 바이너리로 배포 가능하다.
- 긍정: WAL 모드로 reader/writer 동시 접근이 가능하다.
- 긍정: `~/.engram/engram.db` 파일 하나로 전체 상태 관리 및 백업이 간단하다.
- 부정: 대규모 팀 동시 사용에는 적합하지 않다.
- 부정: 네트워크 파일시스템(NFS, SMB)에서 SQLite WAL은 안전하지 않다 — 팀 공유 시나리오는 Phase 4 이후 별도 검토한다.
