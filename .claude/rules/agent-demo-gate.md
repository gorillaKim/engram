# Rule: Agent Demo Gate

> [!WARNING]
> **에이전트는 이슈 상태를 절대 `finished`나 `cancelled`로 직접 변경할 수 없습니다!**
> 모든 태스크와 목표를 달성하더라도 상태를 반드시 **`demo` (검토)**까지만 변경하고 사용자의 승인을 기다려야 합니다.

## 원칙

Engram 이슈 상태 흐름에서 `demo → finished` 와 `* → cancelled` 는 **사용자 전용** 이다.
Agent (직접 호출 또는 engram-worker 서브에이전트) 는 다음을 준수한다:

1. **`issue_update.status` schema에서 `finished`와 `cancelled`는 제외**되었습니다. 에이전트는 이 상태로 직접 전이할 수 없습니다.
2. **`issue_finish` 및 `issue_cancel` 호출 금지**: 이 두 도구는 사용자 전용(User-only) 게이트로 작동하며, `agent_id != "user"`인 경우 에러를 반환합니다.
3. 작업 완료 시에는 반드시 `issue_release(transition_to=demo)`를 호출하고 승인을 기다립니다.
4. demo 진입 직전 반드시 `note_add(type=context, summary, detail)` 으로 검토 가이드 작성
5. demo 진입 후에는 사용자의 칸반 조작을 기다린다 (`task_next` 가 다른 이슈를 반환할 수 있음)

## 위반 시 사후 감사

`history.changed_by` 필드로 agent/user 구분 가능. 다음 쿼리로 위반 탐지:

```sql
SELECT entity_id, new_value, created_at
FROM history
WHERE entity_type = 'issue'
  AND field = 'status'
  AND new_value IN ('finished', 'cancelled')
  AND changed_by = 'agent';
```

## 데스크톱 UI 어포던스

칸반의 demo 컬럼은 amber 배경 + "검토 대기" 배지로 사용자가 놓치지 않게 한다.
`Finished` 버튼은 demo 상태에서만 활성화된다.
