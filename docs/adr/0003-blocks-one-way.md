# ADR-0003: blocks 단방향 저장

## Status
Accepted

## Context
`issue_links` 테이블에 블로킹 관계를 저장할 때 양방향(`A blocks B` + `B blocked_by A`)으로 두 행을 삽입하면 데이터 불일치 위험이 생긴다. 한쪽 행만 삭제되거나, 삽입 트랜잭션이 실패하면 무결성이 깨진다. 또한 "A가 B를 블로킹한다"는 사실 하나를 두 행으로 표현하는 것은 중복이다.

## Decision

`issue_links`에는 `link_type = 'blocks'`만 단방향으로 저장한다. `source_id`가 블로커, `target_id`가 블로킹 대상이다. 역방향 조회(`B가 무엇에 의해 블로킹되는가`)는 `WHERE target_id = ? AND link_type = 'blocks'` 쿼리로 도출한다.

## Consequences

- 긍정: 관계 하나 = 행 하나이므로 데이터 일관성이 보장된다.
- 긍정: 삽입·삭제 로직이 단순하다 — 트랜잭션에서 한 행만 다룬다.
- 긍정: `CHECK(link_type IN ('blocks'))` 제약으로 잘못된 타입 삽입을 DB 레벨에서 차단할 수 있다.
- 부정: 역방향 조회 시 `source_id`/`target_id` 방향을 명시적으로 구분해야 한다 — 쿼리 작성 시 주의가 필요하다.
