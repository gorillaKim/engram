# ADR-0007: Agent Demo Gate — 서버 사이드 검증 및 도구 분리

## Status
Amended (2026-05-24: 서버 사이드 스키마 분리 및 검증 강제 도입)

## Context

Engram 의 이슈 상태 흐름에서 `finished` 와 `cancelled` 는 사용자만 결정해야 한다. Agent 가 실수로 이슈를 `finished` 처리하면 사용자의 검토 기회가 사라진다.
기존에는 클라이언트 측/프롬프트 규칙으로만 강제했으나, 에이전트가 schema에 노출된 `finished`/`cancelled`를 보고 오작동하거나 학습 과정에서 에러를 겪는 등의 문제가 지속되어, 서버 인터페이스 레벨에서 완전히 분리하기로 결정했다.

## Decision

인터페이스 및 서버 사이드에서 다음과 같은 규칙을 강제한다:

1. **`issue_update` 스키마 수정**: `status` enum에서 `finished`, `cancelled`를 제외하여 에이전트가 이 API를 통해 직접 완료/취소 상태로 전이할 수 없게 차단한다.
2. **사용자 전용 도구 신설 (`issue_finish` / `issue_cancel`)**: 
   - `issue_finish`: `demo` 상태의 이슈만 `finished` 상태로 전이할 수 있으며, `changed_by != "user"`인 경우 에러를 반환한다.
   - `issue_cancel`: `finished`가 아닌 임의 상태의 이슈를 `cancelled` 상태로 전이할 수 있으며, 취소 사유(`reason`)를 필수로 받고 `changed_by != "user"`인 경우 에러를 반환한다.
3. **감사 이력 연동**: 전이 성공 시 `history` 테이블에 상태 변경 이력 및 `cancel_reason`을 저장한다.

## Consequences

- 에이전트의 불필요한 시도 차단: 스키마 단에서 허용 값이 아니므로 에이전트가 완료/취소를 원천적으로 시도하지 않음.
- 안전성 향상: 비인가 에이전트가 임의로 도구를 호출하더라도 repository 레이어에서 에러(`Error::Validation`)를 내며 차단됨.
- 히스토리 보강: 취소 시 구체적 사유(`cancel_reason`)를 함께 영구적으로 보존할 수 있음.
