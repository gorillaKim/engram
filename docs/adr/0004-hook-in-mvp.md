# ADR-0004: Claude Code Hook을 MVP에 포함

## Status
Accepted

## Context
Engram의 핵심 가치는 세션 시작 시 자동으로 이전 컨텍스트(진행 중인 이슈, 미완료 태스크, 주의사항)를 복원하는 것이다. Hook 없이는 사용자가 매번 `session_restore`를 직접 요청해야 하는데, 실사용 환경에서 이 단계를 잊어버리면 Engram의 존재 이유가 사라진다.

Hook을 Phase 2 이후로 미루면 MVP 단계에서 실제 사용 가치를 검증할 수 없다.

## Decision

Phase 1 MVP에 Claude Code Hook 연동을 포함한다. `engram hook install` 명령으로 `PreToolUse(Bash)` 훅과 `Stop` 훅을 `.claude/settings.json`에 등록한다. 각 프로젝트의 `CLAUDE.md`에 `project_key`를 설정하면 해당 프로젝트 컨텍스트가 세션 시작 시 자동 주입된다.

## Consequences

- 긍정: 세션 시작 시 별도 요청 없이 자동으로 컨텍스트가 주입된다.
- 긍정: MVP 단계에서 핵심 가치를 실제 사용 환경에서 검증할 수 있다.
- 부정: 프로젝트마다 `CLAUDE.md`에 `project_key` 설정을 수동으로 추가해야 한다.
- 부정: `engram hook install`을 실행하지 않은 환경에서는 수동 호출이 필요하다.
