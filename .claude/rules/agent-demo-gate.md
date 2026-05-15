# Rule: Agent Demo Gate

## 원칙

Engram 이슈 상태 흐름에서 `demo → finished` 와 `* → cancelled` 는 **사용자 전용** 이다.
Agent (직접 호출 또는 engram-worker 서브에이전트) 는 다음을 준수한다:

1. **`issue_update(status=finished)` 호출 금지**
2. **`issue_update(status=cancelled)` 호출 금지**
3. demo 진입 직전 반드시 `note_add(type=context, summary, detail)` 으로 검토 가이드 작성
4. demo 진입 후에는 사용자의 칸반 조작을 기다린다 (`task_next` 가 다른 이슈를 반환할 수 있음)

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
