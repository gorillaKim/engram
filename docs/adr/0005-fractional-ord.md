# ADR-0005: tasks.ord REAL 타입 Fractional Index

## Status
Accepted

## Context
이슈 내 태스크 순서를 사용자가 자유롭게 재배치해야 한다. 정수 인덱스(`1, 2, 3, ...`)를 사용하면 중간 삽입 시 후속 태스크 전체를 재정규화해야 하므로 O(n) 쓰기가 발생한다. 또한 SQL에서 `order`는 예약어이므로 컬럼명으로 직접 사용할 수 없다.

## Decision

`tasks.ord REAL` 컬럼을 fractional index로 사용한다. 삽입 규칙은 다음과 같다: 끝에 추가할 때는 `last_ord + 1.0`, 맨 앞에 추가할 때는 `first_ord - 1.0`, 두 태스크 사이에 삽입할 때는 `(prev_ord + next_ord) / 2.0`. 컬럼명은 SQL 예약어 충돌을 피하기 위해 `ord`로 정한다. 조회 시 항상 `ORDER BY ord ASC`를 사용한다.

## Consequences

- 긍정: 중간 삽입이 O(1)이다 — 다른 행의 `ord`를 변경하지 않는다.
- 긍정: 대부분의 사용 패턴에서 재정규화가 불필요하다.
- 부정: IEEE 754 double 정밀도 한계로 같은 두 값 사이에 약 52회 연속 중간 삽입 후 재정규화가 필요하다. 재정규화 로직은 `.claude/rules/fractional-index.md`에 명시되어 있다.
- 부정: `ord` 컬럼명이 `order`보다 직관적이지 않아 코드 가독성이 약간 낮다.
