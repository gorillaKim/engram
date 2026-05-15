# M2 — DnD 상태 전이 + Drawer + Finished 필터

> **상위 문서**: [overview.md](./overview.md) · **이전**: [m1-scaffold-board.md](./m1-scaffold-board.md) · **다음**: [m3-mcp-supervisor.md](./m3-mcp-supervisor.md)

**예상 기간**: 1주

## 전제

[M1](./m1-scaffold-board.md) 완료 — 읽기 전용 칸반 5컬럼이 정상 표시되고 single-instance 플러그인 작동.

## 목표

칸반 카드를 드래그-앤-드롭으로 상태 전이시키고, 카드 클릭으로 Issue Detail Drawer 가 열려 태스크 체크리스트와 노트를 보여주며, Demo 컬럼에서 사용자가 직접 `Finished` 처리할 수 있다. Finished 누적 가독성 보호용 **hide-finished 토글** 도입.

## Scope

### 1. 쓰기 Tauri Commands 추가

**`crates/engram-desktop/src/commands.rs`** — `changed_by="user"` 명시:

```rust
#[tauri::command]
pub async fn issue_set_status(
    db: State<'_, Arc<Db>>,
    id: i64,
    status: String,
) -> Result<Issue, String> {
    let parsed = parse_issue_status(&status).map_err(|e| e.to_string())?;
    db.issue_update(id, UpdateIssueInput {
        status: Some(parsed),
        ..Default::default()
    }, "user").await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn issue_set_priority(/* ... changed_by="user" ... */) -> Result<Issue, String>;

#[tauri::command]
pub async fn task_set_status(/* ... changed_by="user" ... */) -> Result<Task, String>;

#[tauri::command]
pub async fn note_add(/* ... */) -> Result<Note, String>;
#[tauri::command]
pub async fn note_get(/* ... */) -> Result<Note, String>;
#[tauri::command]
pub async fn note_list(/* ... */) -> Result<Vec<NoteSummary>, String>;
#[tauri::command]
pub async fn note_resolve(/* ... changed_by="user" ... */) -> Result<(), String>;

#[tauri::command]
pub async fn task_list(/* ... */) -> Result<Vec<Task>, String>;

#[tauri::command]
pub async fn blocked_issues_graph(/* ... */) -> Result<BlockingGraph, String>;
```

`InvalidTransition` 에러는 그대로 문자열화되어 프론트로 전파 → toast.

### 2. dnd-kit 통합

```bash
pnpm add @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities
```

**`useIssueDnd.ts`**:
```ts
export function useIssueDnd() {
  const qc = useQueryClient();
  const setStatus = useMutation({
    mutationFn: ({ id, status }: { id: number; status: string }) =>
      invoke<Issue>('issue_set_status', { id, status }),
    onMutate: async ({ id, status }) => {
      await qc.cancelQueries({ queryKey: ['boardStatus'] });
      const prev = qc.getQueryData(['boardStatus']);
      // optimistic update: 해당 이슈를 새 컬럼으로
      qc.setQueryData(['boardStatus'], (old: any) => /* ... */);
      return { prev };
    },
    onError: (err, _vars, ctx) => {
      // 롤백 + toast
      qc.setQueryData(['boardStatus'], ctx?.prev);
      toast.error(`상태 변경 실패: ${err}`);
    },
    onSettled: () => qc.invalidateQueries({ queryKey: ['boardStatus'] }),
  });
  return setStatus;
}
```

`KanbanBoard.tsx` 가 `DndContext` 로 감싸고, 각 컬럼이 `useDroppable`, 각 카드가 `useDraggable` 또는 `useSortable`.

### 3. Issue Detail Drawer

`shadcn` drawer 컴포넌트 활용. Trigger 는 `IssueCard` 의 onClick (단, drag 중일 때는 무시 — dnd-kit 의 `activationConstraint: { distance: 5 }` 로 분리).

**`IssueDetail.tsx`** 표시 항목:
- 헤더: `#{id}` + 제목 + 상태 dropdown + priority dropdown + epic chip
- Body:
  - **목표** (`issue.goal`) — 강조 박스
  - **설명** (`issue.description`)
  - **태스크 리스트** (`task_list` 호출): checkbox → `task_set_status('finished' | 'ready')`. fractional ord 순서대로
  - **노트 리스트** (`note_list`):
    - 타입별 아이콘 (caveat=⚠ / decision=★ / discovery=💡 / blocker_detail=🚫 / context=✎ / reference=📎)
    - 클릭 시 `note_get` 으로 detail 펼치기
  - **블로커 표시** — `blocked_issues_graph` 호출해 해당 이슈를 막는 blocker 가 있으면 highlight
- 푸터 액션:
  - `[Working 으로 되돌리기]` (demo 상태에서 demo→working)
  - `[완료로 표시 (Finished)]` (demo 상태에서만 활성)
  - `[취소 (Cancelled)]`

### 4. Finished 필터 토글

> Opus 권고 D — Finished 가 누적되면 가독성 떨어짐. M2 에서 1줄 토글로 처리.

**`store/ui.ts`**:
```ts
type UIState = {
  hideFinished: boolean;
  toggleHideFinished: () => void;
  // ...
};
```

**`KanbanBoard.tsx`** 의 컬럼 렌더링:
```tsx
{!hideFinished && <KanbanColumn status="finished" ... />}
```

상단 필터 바에 "✓ 완료 숨기기" 체크박스.

### 5. 잘못된 전이 토스트

shadcn 의 `toast` 또는 `sonner` 사용. `useIssueDnd` 의 `onError` 에서 메시지 노출:

```
working → required 는 허용되지 않습니다.
       이슈를 취소하려면 cancelled 컬럼에 놓아주세요.
```

`engram-core::Error::InvalidTransition(s)` 문자열을 그대로 보여줘도 충분. (i18n 은 M5)

## 변경 파일 목록

```
crates/engram-desktop/src/commands.rs                 (M)  쓰기 명령 추가
crates/engram-desktop/src/main.rs                     (M)  invoke_handler 에 신규 명령 등록
crates/engram-desktop/ui/src/                         (M/+)
  components/IssueCard.tsx                            (M)  useDraggable, onClick → drawer open
  components/KanbanColumn.tsx                         (M)  useDroppable
  components/KanbanBoard.tsx                          (M)  DndContext, hideFinished 적용
  hooks/useIssueDnd.ts                                (+)  optimistic mutation
  routes/IssueDetail.tsx                              (+)  drawer 내용
  components/TaskChecklist.tsx                        (+)  태스크 체크박스
  components/NoteList.tsx                             (+)  note summary + on-click detail
  store/ui.ts                                         (M)  hideFinished, selectedIssueId
  ipc/invoke.ts                                       (M)  쓰기 명령 래퍼
```

## Verification

1. **단위**
   ```bash
   cargo test -p engram-desktop                        # commands.rs 검증
   pnpm --filter engram-desktop-ui test                # useIssueDnd / IssueCard
   ```
2. **수동 시나리오**
   - Working 카드 → Demo 컬럼으로 드래그 → DB `issues.status='demo'`, `history.changed_by='user'` 확인 (`sqlite3 ~/.engram/engram.db "SELECT * FROM history ORDER BY id DESC LIMIT 3"`)
   - Demo 카드 클릭 → Drawer 열림 → `[완료로 표시]` 클릭 → `finished` 전환
   - Finished 카드를 Required 로 드래그 → toast `InvalidTransition` + 카드 복귀
   - "완료 숨기기" 토글 → Finished 컬럼 자체가 사라짐
3. **회귀**
   - 기존 36 + M0 신규 1 + M1/M2 신규 = 모두 green

## Out of Scope

- MCP Supervisor (→ M3)
- 트레이 / 알림 (→ M4)
- 고급 필터 / BlockingGraph 시각화 (→ M5)

## 완료 기준

- [x] 5컬럼 사이 모든 유효 전이가 DnD 로 가능 (dnd-kit DndContext + useDroppable/useDraggable)
- [x] 무효 전이는 toast + 자동 롤백 (useIssueDnd onError → sonner toast + QueryClient rollback)
- [x] Issue Detail Drawer 가 태스크/노트 표시 (IssueDetail.tsx + TaskChecklist + NoteList)
- [x] Demo → Finished 처리 시 `history.changed_by='user'` (do_issue_set_status → changed_by="user")
- [x] Hide-finished 토글 동작 (UIStore.hideFinished + KanbanBoard 필터)
- [x] 모든 테스트 green (`cargo test --workspace`: 47 passed)
