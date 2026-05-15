# M5 — 폴리시 (필터 / 그래프 시각화 / ADR & 규칙 정리)

> **상위 문서**: [overview.md](./overview.md) · **이전**: [m4-tray-notifications.md](./m4-tray-notifications.md)

**예상 기간**: 3~4일

## 전제

M0~M4 완료. 앱은 일상 사용 가능한 수준이고, 이제 거친 부분을 다듬고 거버넌스 문서를 마무리한다.

## 목표

1. 보드 사용성 향상: project/epic/priority/date-range 필터, 스코프 팽창 경고 시각화
2. BlockingGraph 시각화 (Drawer 내 작은 다이어그램)
3. **서브에이전트 정의**와 **규칙 문서** 정식 작성 → Demo Gate 정책 발효
4. ADR 0006 (Tauri 스택), 0007 (Demo Gate) 작성. CLAUDE.md 갱신

## Scope

### 1. 고급 필터

상단 필터 바 확장:

| 필터 | 동작 |
|---|---|
| Project | `BoardStatus.projects` 에서 자동 추출, 다중 선택 가능 |
| Epic | 선택된 Project 의 에픽만 표시 |
| Priority | critical/high/medium/low 다중 토글 |
| Date range | sprint 시작~종료 기본, custom range 가능 |
| Hide finished | (M2 의 토글 유지) |
| Show cancelled | 기본 off |

상태는 모두 Zustand `boardFilters` 에 묶음. `useBoardStatus` query key 에 포함 → 자동 refetch.

### 2. 스코프 팽창 경고 시각화

`session_restore` 의 `warnings` 가 이미 `agent_discovered > 50%` 이슈를 알려줌. 보드 상단에:

```
⚠ 스코프 팽창 감지 (2건)                            [상세 보기]
  • #39 refresh token CSRF 보호 — discovered 60% (3/5)
  • #41 토큰 갱신 정책 — discovered 75% (3/4)
```

해당 카드는 우상단에 `⚠팽창` 배지 (M0~M4 에서 디자인은 이미 시안에 포함).

### 3. BlockingGraph 시각화

Issue Detail Drawer 안에 작은 다이어그램:

- 노드: 이슈 (현재 이슈는 강조)
- 엣지: blocks 관계
- 사이클 감지 시 빨간 표시

라이브러리: **`@xyflow/react`** (가볍고 React-friendly). 단순 그래프이므로 manual layout 으로 시작:

```
   [#43] ─blocks─► [#36 ← here] ─blocks─► [#28]
                     ▲
                     │
                   [#44]
```

`blocked_issues_graph(project_key)` 결과를 받아 ego-network (1-hop neighbors) 표시.

### 4. 서브에이전트 정의

**`.claude/agents/engram-worker.md`** (신규):

```markdown
---
name: engram-worker
description: |
  Engram 이슈를 처리하는 작업자 서브에이전트. 상태 전이는 working → demo 까지만 수행하며,
  finished/cancelled 처리는 절대 하지 않습니다. demo 진입 직전에 검증 결과를
  note_add(type=context) 로 남겨 사용자가 검토할 수 있게 합니다.
tools: ['mcp__engram__*']
---

# Engram Worker

## 역할

지정된 이슈를 분석·구현·검증하여 사용자가 검토할 수 있는 demo 상태까지 끌어올린다.

## 작업 흐름

1. `session_restore` 로 컨텍스트 파악
2. `task_next` 로 다음 태스크 선택
3. 작업 진행 — 발견된 새 작업은 `task_insert_after(source=agent_discovered)` 로 추가
4. 태스크 완료 시 `task_update(status=finished)`
5. 모든 태스크 완료 → 이슈 상태 `working → demo`
6. demo 직전 `note_add(type=context, summary="검토 가이드: ...", detail=...)`
7. **여기서 정지**. `issue_update(status=finished)` 를 **절대 호출하지 않음**

## 금지 사항

- `issue_update(status=finished)` 호출
- `issue_update(status=cancelled)` 호출 (사용자 결정 사항)

위반 시 사용자가 즉시 칸반에서 되돌릴 수 있고, `history.changed_by='agent'` 로 추적되므로 사후 감사 가능.
```

### 5. 규칙 문서

**`.claude/rules/agent-demo-gate.md`** (신규):

```markdown
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
```

CLAUDE.md 의 규칙 표에 등록:

```markdown
| 작업 | 참조할 규칙 |
|------|------------|
| ...                       | ... |
| Demo→Finished 전이        | `.claude/rules/agent-demo-gate.md` |
```

### 6. ADR 작성

- **`docs/adr/0006-desktop-tauri.md`**:
  - Status: Accepted
  - Decision: Tauri v2 + React + Tailwind + shadcn/ui + dnd-kit 채택
  - Reasons: 단일 바이너리, native shell, 풍부한 plugin 생태계, MIT 라이선스, Mac/Win 동시 지원
  - Trade-offs: Webview 성능 한계, plugin 일부 베타. Electron 보다 메모리 절약
- **`docs/adr/0007-agent-demo-gate.md`**:
  - Status: Accepted
  - Decision: 코드 강제 X, 서브에이전트 프롬프트 + `.claude/rules/agent-demo-gate.md` + `history.changed_by` 감사
  - Reasons: 단일 사용자 환경에서 코드 게이트는 과공학. UI 어포던스로 사용자가 즉시 되돌릴 수 있음
  - Trade-offs: 외부 MCP 클라이언트가 우회 가능. 필요 시 `--require-auth` 옵션 추가 검토

### 7. CLAUDE.md 갱신

- 진행 상황 요약에 Phase 3 (Desktop) 완료 추가
- ADR 표에 0006/0007/0008 추가
- 규칙 표에 `agent-demo-gate.md` 추가

### 8. 도크 뱃지 (선택)

Tauri v2 `set_badge_count` API 로 검토 대기 (demo) 카운트를 도크 아이콘에 표시. Watcher 가 변동 시 갱신.

### 9. 환경설정 윈도우 정식화

M4 의 단순 dialog 를 본격 윈도우로 승격:

- MCP: autostart, port, transport
- 알림: 카테고리별 on/off (required/demo/blocker), 조용한 시간 설정
- UI: 기본 project filter, hide finished 기본값
- DB: 백업 경로, "DB 위치 열기" 버튼

## 변경 파일 목록

```
crates/engram-desktop/ui/src/
  components/FilterBar.tsx                            (M)  Project/Epic/Priority/DateRange/Cancelled
  components/ScopeExpansionBanner.tsx                 (+)
  components/IssueCard.tsx                            (M)  ⚠팽창 배지
  components/BlockingGraph.tsx                        (+)
  routes/IssueDetail.tsx                              (M)  BlockingGraph 통합
  routes/Settings.tsx                                 (+)
  store/ui.ts                                         (M)  filters, notification prefs
crates/engram-desktop/src/watcher.rs                  (M)  dock badge 갱신
crates/engram-desktop/Cargo.toml                      (M)  필요한 plugin
.claude/agents/engram-worker.md                       (+)
.claude/rules/agent-demo-gate.md                      (+)
docs/adr/0006-desktop-tauri.md                        (+)
docs/adr/0007-agent-demo-gate.md                      (+)
CLAUDE.md                                             (M)  ADR/규칙/진행상황
```

## Verification

1. **필터**
   - 프로젝트 2개 + 에픽 3개 + priority high 만 → 카드 노출이 의도대로
   - 필터 조합 변경마다 URL/state 가 보존
2. **스코프 팽창**
   - agent_discovered > 50% 이슈에서 배너 + 카드 배지 표시
3. **블로킹 그래프**
   - A blocks B, B blocks C 셋업 → Drawer 안에 A→B→C 그래프 표시
   - 사이클 (B blocks A) 추가 → 빨간 사이클 표시
4. **서브에이전트**
   - Claude Code 에서 `/engram-worker` 호출 시 description 이 표시되고 finished 호출이 거부됨 (rule + prompt 효과 확인)
   - 부주의로 호출했을 경우 `SELECT ... WHERE changed_by='agent' AND new_value='finished'` 로 탐지 가능
5. **회귀**
   - `cargo test --workspace` 모두 green
   - 기존 CLI/MCP 동작 변경 없음

## 완료 기준

- [ ] 모든 필터 동작
- [ ] 스코프 팽창 시각화
- [ ] BlockingGraph 시각화
- [ ] 서브에이전트 + 규칙 문서 머지, CLAUDE.md 표 갱신
- [ ] ADR 0006/0007 머지
- [ ] (선택) 도크 뱃지
- [ ] (선택) 환경설정 윈도우
