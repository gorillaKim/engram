# ADR-0001: Single Central DB

## Status
Accepted

## Context
Engram은 여러 프로젝트(xpert-da-web, xpert-na-web 등)를 하나의 에이전트가 관리하는 구조다. DB를 프로젝트별로 분리하면 두 가지 문제가 생긴다. 첫째, 프로젝트 A의 이슈가 프로젝트 B의 이슈를 블로킹하는 cross-project 관계를 `issue_links` 테이블로 표현할 수 없다. 둘째, MCP 서버를 프로젝트마다 별도로 띄워야 하므로 Claude Code 설정이 복잡해진다.

단일 DB라면 이 두 문제를 모두 피할 수 있고, `session_restore` 같은 전체 스프린트 현황 조회도 단일 쿼리로 처리된다.

## Decision

단일 중앙 DB `~/.engram/engram.db`를 사용한다. 프로젝트 구분은 `epics.project_key` 컬럼(TEXT)으로 처리하며, 조회 시 `project_key` 파라미터로 필터링한다. MCP 서버는 하나만 실행한다.

## Consequences

- 긍정: cross-project 블로킹 관계를 `issue_links` 테이블에 직접 저장할 수 있다.
- 긍정: MCP 서버 하나로 모든 프로젝트를 통합 관리한다.
- 긍정: 전체 스프린트 현황을 단일 쿼리로 조회할 수 있다.
- 부정: 팀 공유 시 DB 동기화 방법이 없다 (Phase 4 이후 검토 대상).
- 부정: DB 파일을 잘못 삭제하면 모든 프로젝트 데이터가 손실된다.
