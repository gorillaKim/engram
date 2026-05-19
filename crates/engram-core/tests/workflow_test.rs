use engram_core::{
    Db,
    models::{
        sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus},
        epic::CreateEpicInput,
        issue::{CreateIssueInput, UpdateIssueInput, IssueStatus, IssuePriority},
        task::{CreateTaskInput, UpdateTaskInput, TaskStatus},
        note::{CreateNoteInput, NoteType},
        history::EntityType,
        LinkType,
    },
};

async fn setup() -> Db {
    Db::open_in_memory().await.unwrap()
}

async fn seed_sprint_epic(db: &Db) -> (i64, i64) {
    let sprint = db.sprint_create(CreateSprintInput {
        name: "Test Sprint".to_string(),
        goal: None,
        start_date: None,
        end_date: None,
    }).await.unwrap();

    db.sprint_update(sprint.id, UpdateSprintInput {
        status: Some(SprintStatus::Active),
        ..Default::default()
    }, "agent").await.unwrap();

    let epic = db.epic_create(CreateEpicInput {
        project_key:"test-project".to_string(),
        title: "Test Epic".to_string(),
        description: None,
    }).await.unwrap();

    (sprint.id, epic.id)
}

#[tokio::test]
async fn test_full_sprint_workflow() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    // 이슈 생성 (required)
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        sprint_id: Some(sprint_id),
        title:"Test Issue".to_string(),
        description: None,
        goal: Some("인증 흐름 완전 전환".to_string()),
        priority: None,
    }).await.unwrap();

    assert_eq!(issue.status, IssueStatus::Required);

    // 이슈 준비 완료 (required → ready)
    let ready_issue = db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();
    assert_eq!(ready_issue.status, IssueStatus::Ready);

    // 태스크 생성 후 ready 전환 (task_next는 ready 태스크만 반환)
    let t1 = db.task_create(CreateTaskInput {
        issue_id: issue.id,
        title: "Task 1".to_string(),
        description: None,
        goal: None,
        after_task_id: None,
        source: None,
    }).await.unwrap();

    db.task_update(t1.id, UpdateTaskInput {
        status: Some(TaskStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // caveat note 추가
    db.note_add(CreateNoteInput {
        issue_id: issue.id,
        task_id: None,
        note_type: NoteType::Caveat,
        summary: "조심할 점".to_string(),
        detail: None,
        author: None,
        agent_id: None,
        scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();

    // session_restore — active_epics에 이슈가 포함되어야 함
    let snapshot = db.session_restore(Some("test-project")).await.unwrap();
    assert!(!snapshot.active_epics.is_empty(), "active_epics 비어있음");

    let epic_snap = &snapshot.active_epics[0];
    assert!(!epic_snap.active_issues.is_empty(), "active_issues 비어있음");

    let issue_snap = &epic_snap.active_issues[0];
    assert_eq!(issue_snap.active_notes.len(), 1, "caveat note 1건이어야 함");
    assert_eq!(issue_snap.active_notes[0].note_type, NoteType::Caveat);

    // task_next — ready 태스크가 반환되어야 함
    let next = db.task_next(Some("test-project"), None).await.unwrap();
    assert!(next.is_some(), "task_next가 None을 반환");
    assert_eq!(next.unwrap().task_id, t1.id);

    println!("✅ 전체 워크플로우 테스트 통과");
}

#[tokio::test]
async fn test_blocked_by_reverse_query() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    let issue_a = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id), title:"Issue A".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let issue_b = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id), title:"Issue B".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();

    // A blocks B
    db.issue_link(issue_a.id, issue_b.id, engram_core::models::LinkType::Blocks).await.unwrap();

    // B의 blocked_by 역방향 조회 → A가 반환돼야 함
    let blockers = db.issue_blocked_by(issue_b.id).await.unwrap();
    assert_eq!(blockers.len(), 1, "blocker 1건이어야 함");
    assert_eq!(blockers[0].source_id, issue_a.id);

    println!("✅ blocked_by 역방향 쿼리 테스트 통과");
}

#[tokio::test]
async fn test_fractional_ord_insert() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id), title:"Issue".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let t1 = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T1".to_string(),
        description: None, goal: None, after_task_id: None, source: None,
    }).await.unwrap(); // ord = 1.0

    let t2 = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T2".to_string(),
        description: None, goal: None, after_task_id: None, source: None,
    }).await.unwrap(); // ord = 2.0

    // T1과 T2 사이에 삽입
    let t_mid = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T_mid".to_string(),
        description: None, goal: None, after_task_id: Some(t1.id), source: None,
    }).await.unwrap(); // ord = 1.5

    assert!(t_mid.ord > t1.ord && t_mid.ord < t2.ord, "ord 순서 오류");

    let tasks = db.task_list(issue.id, None).await.unwrap();
    assert_eq!(tasks[0].title, "T1");
    assert_eq!(tasks[1].title, "T_mid");
    assert_eq!(tasks[2].title, "T2");

    println!("✅ Fractional index 삽입 테스트 통과");
}

#[tokio::test]
async fn test_session_restore_filters_by_project() {
    let db = setup().await;

    let sprint = db.sprint_create(CreateSprintInput {
        name: "Filter Sprint".to_string(),
        goal: None,
        start_date: None,
        end_date: None,
    }).await.unwrap();

    db.sprint_update(sprint.id, UpdateSprintInput {
        status: Some(SprintStatus::Active),
        ..Default::default()
    }, "agent").await.unwrap();

    let epic_a = db.epic_create(CreateEpicInput {
        project_key:"proj-a".to_string(),
        title: "Epic A".to_string(),
        description: None,
    }).await.unwrap();

    let epic_b = db.epic_create(CreateEpicInput {
        project_key:"proj-b".to_string(),
        title: "Epic B".to_string(),
        description: None,
    }).await.unwrap();

    // proj-a 이슈 생성 후 Ready 전환
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id: epic_a.id,
        sprint_id: Some(sprint.id),
        title:"Issue A".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // proj-b 이슈 생성 후 Ready 전환
    let issue_b = db.issue_create(CreateIssueInput {
        epic_id: epic_b.id,
        sprint_id: Some(sprint.id),
        title:"Issue B".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_b.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // proj-a 조회 → proj-a 에픽만
    let snap_a = db.session_restore(Some("proj-a")).await.unwrap();
    assert_eq!(snap_a.active_epics.len(), 1, "proj-a: active_epics는 1개여야 함");
    assert_eq!(snap_a.active_epics[0].epic.project_key, "proj-a");

    // proj-b 조회 → proj-b 에픽만
    let snap_b = db.session_restore(Some("proj-b")).await.unwrap();
    assert_eq!(snap_b.active_epics.len(), 1, "proj-b: active_epics는 1개여야 함");
    assert_eq!(snap_b.active_epics[0].epic.project_key, "proj-b");
}

#[tokio::test]
async fn test_task_next_priority_ordering() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    // 이슈 A: Critical
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id,
        sprint_id: Some(sprint_id),
        title:"Critical Issue".to_string(),
        description: None,
        goal: None,
        priority: Some(IssuePriority::Critical),
    }).await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // 이슈 B: High
    let issue_b = db.issue_create(CreateIssueInput {
        epic_id,
        sprint_id: Some(sprint_id),
        title:"High Issue".to_string(),
        description: None,
        goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();
    db.issue_update(issue_b.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // 각 이슈에 태스크 1개씩 생성 후 Ready 전환
    let task_a = db.task_create(CreateTaskInput {
        issue_id: issue_a.id,
        title: "Task Critical".to_string(),
        description: None,
        goal: None,
        after_task_id: None,
        source: None,
    }).await.unwrap();
    db.task_update(task_a.id, UpdateTaskInput {
        status: Some(TaskStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    let task_b = db.task_create(CreateTaskInput {
        issue_id: issue_b.id,
        title: "Task High".to_string(),
        description: None,
        goal: None,
        after_task_id: None,
        source: None,
    }).await.unwrap();
    db.task_update(task_b.id, UpdateTaskInput {
        status: Some(TaskStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // task_next → Critical 이슈의 태스크가 먼저 반환돼야 함
    let next = db.task_next(Some("test-project"), None).await.unwrap();
    assert!(next.is_some(), "task_next가 None을 반환");
    assert_eq!(next.unwrap().task_id, task_a.id, "Critical 이슈의 태스크가 먼저 반환돼야 함");
}

#[tokio::test]
async fn test_cross_project_blocking() {
    let db = setup().await;

    let sprint = db.sprint_create(CreateSprintInput {
        name: "Block Sprint".to_string(),
        goal: None,
        start_date: None,
        end_date: None,
    }).await.unwrap();
    db.sprint_update(sprint.id, UpdateSprintInput {
        status: Some(SprintStatus::Active),
        ..Default::default()
    }, "agent").await.unwrap();

    let epic_a = db.epic_create(CreateEpicInput {
        project_key:"proj-a".to_string(),
        title: "Epic A".to_string(),
        description: None,
    }).await.unwrap();
    let epic_b = db.epic_create(CreateEpicInput {
        project_key:"proj-b".to_string(),
        title: "Epic B".to_string(),
        description: None,
    }).await.unwrap();

    // proj-a 이슈 A (Ready)
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id: epic_a.id,
        sprint_id: Some(sprint.id),
        title:"Issue A (blocker)".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // proj-b 이슈 B (Ready)
    let issue_b = db.issue_create(CreateIssueInput {
        epic_id: epic_b.id,
        sprint_id: Some(sprint.id),
        title:"Issue B (blocked)".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_b.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // A blocks B
    db.issue_link(issue_a.id, issue_b.id, LinkType::Blocks).await.unwrap();

    // proj-b 이슈 B에 태스크 생성 후 Ready 전환
    let task_b = db.task_create(CreateTaskInput {
        issue_id: issue_b.id,
        title: "Task B".to_string(),
        description: None,
        goal: None,
        after_task_id: None,
        source: None,
    }).await.unwrap();
    db.task_update(task_b.id, UpdateTaskInput {
        status: Some(TaskStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // 이슈 B는 blocked → task_next(proj-b) None 반환
    let next_before = db.task_next(Some("proj-b"), None).await.unwrap();
    assert!(next_before.is_none(), "이슈 B가 blocked 상태일 때 task_next는 None이어야 함");

    // 이슈 A를 Finished로 전환 (Required → Ready → Working → Finished 순이나 test에선 직접 DB 우회 불가 — Working 거쳐야 함)
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Finished),
        ..Default::default()
    }, "agent").await.unwrap();

    // 이제 이슈 B의 blocker가 finished → task_next(proj-b) 태스크 반환
    let next_after = db.task_next(Some("proj-b"), None).await.unwrap();
    assert!(next_after.is_some(), "blocker가 finished 된 후 task_next는 태스크를 반환해야 함");
    assert_eq!(next_after.unwrap().task_id, task_b.id, "이슈 B의 태스크가 반환돼야 함");
}

#[tokio::test]
async fn test_scope_expansion_warning() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    // 이슈 생성 및 Ready 전환
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        sprint_id: Some(sprint_id),
        title:"Scope Expansion Issue".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // 1개 planned 태스크
    db.task_create(CreateTaskInput {
        issue_id: issue.id,
        title: "Planned Task".to_string(),
        description: None, goal: None, after_task_id: None,
        source: Some(engram_core::models::task::TaskSource::Planned),
    }).await.unwrap();

    // 3개 agent_discovered 태스크 (75% → 팽창 경고)
    for i in 0..3 {
        db.task_create(CreateTaskInput {
            issue_id: issue.id,
            title: format!("Discovered Task {i}"),
            description: None, goal: None, after_task_id: None,
            source: Some(engram_core::models::task::TaskSource::AgentDiscovered),
        }).await.unwrap();
    }

    let snapshot = db.session_restore(Some("test-project")).await.unwrap();

    let expansion_warning = snapshot.warnings.iter()
        .any(|w| w.contains("스코프 팽창") || w.contains("agent_discovered") || w.contains("팽창"));

    assert!(expansion_warning, "팽창 경고가 warnings에 포함돼야 함. 현재 warnings: {:?}", snapshot.warnings);
}

#[tokio::test]
async fn test_history_records_changed_by_actor() {
    let db = setup().await;

    let sprint = db.sprint_create(CreateSprintInput {
        name: "Actor Sprint".to_string(),
        goal: None,
        start_date: None,
        end_date: None,
    }).await.unwrap();
    db.sprint_update(sprint.id, UpdateSprintInput {
        status: Some(SprintStatus::Active),
        ..Default::default()
    }, "user").await.unwrap();

    let epic = db.epic_create(engram_core::models::epic::CreateEpicInput {
        project_key:"actor-test".to_string(),
        title: "Actor Epic".to_string(),
        description: None,
    }).await.unwrap();

    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id,
        sprint_id: Some(sprint.id),
        title:"Actor Issue".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    // agent 가 working 까지 전환
    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Ready),   ..Default::default() }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Working),  ..Default::default() }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Demo),     ..Default::default() }, "agent").await.unwrap();
    // 사용자가 Finished 로 전환
    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Finished), ..Default::default() }, "user").await.unwrap();

    let history = db.history_list(EntityType::Issue, issue.id).await.unwrap();
    let status_history: Vec<_> = history.iter().filter(|h| h.field == "status").collect();
    assert!(!status_history.is_empty(), "status history가 존재해야 함");

    let last = status_history.last().unwrap();
    assert_eq!(last.changed_by, "user", "finished 전이는 사용자가 한 것으로 기록되어야 함");

    let demo_entry = status_history.iter().rfind(|h| h.new_value.as_deref() == Some("demo")).unwrap();
    assert_eq!(demo_entry.changed_by, "agent", "demo 전이는 agent가 한 것으로 기록되어야 함");
}

#[tokio::test]
async fn test_sprint_single_active_invariant() {
    let db = setup().await;

    let s1 = db.sprint_create(CreateSprintInput {
        name: "S1".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    let s2 = db.sprint_create(CreateSprintInput {
        name: "S2".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();

    db.sprint_update(s1.id, UpdateSprintInput {
        status: Some(SprintStatus::Active), ..Default::default()
    }, "user").await.unwrap();
    assert_eq!(db.sprint_get(s1.id).await.unwrap().status, SprintStatus::Active);

    // s2 활성화 → s1 은 자동으로 planning 으로 강등되어야 함
    db.sprint_update(s2.id, UpdateSprintInput {
        status: Some(SprintStatus::Active), ..Default::default()
    }, "user").await.unwrap();

    assert_eq!(db.sprint_get(s1.id).await.unwrap().status, SprintStatus::Planning,
        "이전 활성 스프린트는 planning 으로 강등되어야 함");
    assert_eq!(db.sprint_get(s2.id).await.unwrap().status, SprintStatus::Active);

    let active = db.sprint_list(None).await.unwrap()
        .into_iter().filter(|s| s.status == SprintStatus::Active).count();
    assert_eq!(active, 1, "활성 스프린트는 항상 1개 이하여야 함");
}

#[tokio::test]
async fn test_sprint_delete_empty_ok_and_blocked_when_has_epic() {
    let db = setup().await;

    let empty = db.sprint_create(CreateSprintInput {
        name: "Empty".into(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    db.sprint_delete(empty.id).await.expect("빈 스프린트는 삭제 가능해야 함");
    assert!(db.sprint_get(empty.id).await.is_err(), "삭제된 스프린트 조회는 실패해야 함");

    // 새 설계: 이슈는 issues.sprint_id 로 직접 스프린트 소속.
    // 스프린트 삭제 시 ON DELETE SET NULL 로 이슈가 자동으로 백로그로 이동.
    let (sprint_id, _) = seed_sprint_epic(&db).await;
    db.sprint_delete(sprint_id).await.expect("이슈/에픽이 있어도 스프린트 삭제는 가능해야 함 (이슈는 백로그로 이동)");
    assert!(db.sprint_get(sprint_id).await.is_err(), "삭제된 스프린트 조회는 실패해야 함");
}

#[tokio::test]
async fn test_stalled_issues_detects_working_issue() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    // 이슈 두 건: 하나는 working 으로 전이, 하나는 required 로 그대로 둠
    let working_issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "Working Issue".to_string(),
        description: None, goal: None, priority: None,
    }).await.unwrap();
    db.issue_update(working_issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready), ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(working_issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Working), ..Default::default()
    }, "agent").await.unwrap();

    let _required_issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "Required Issue".to_string(),
        description: None, goal: None, priority: None,
    }).await.unwrap();

    // threshold=0: working 상태인 이슈가 한 건 잡힌다
    let stalled = db.stalled_issues(Some("test-project"), IssueStatus::Working, 0).await.unwrap();
    assert_eq!(stalled.len(), 1, "working 이슈 1건이 잡혀야 함");
    assert_eq!(stalled[0].id, working_issue.id);
    assert_eq!(stalled[0].project_key, "test-project");
    assert_eq!(stalled[0].status, IssueStatus::Working);
    assert!(stalled[0].minutes_in_status >= 0);

    // threshold=10000: 방금 만든 이슈가 10000분 정체일 수 없음 → 빈 목록
    let none = db.stalled_issues(Some("test-project"), IssueStatus::Working, 10_000).await.unwrap();
    assert!(none.is_empty(), "10000분 정체는 새 이슈에서 발견될 수 없음");

    // required 상태도 잡혀야 함 (history 없는 경우 updated_at 폴백)
    let req_stalled = db.stalled_issues(None, IssueStatus::Required, 0).await.unwrap();
    assert_eq!(req_stalled.len(), 1, "required 상태 이슈가 한 건 (project_key 필터 없음)");

    // 다른 프로젝트로 필터하면 빈 목록
    let other_proj = db.stalled_issues(Some("other-project"), IssueStatus::Working, 0).await.unwrap();
    assert!(other_proj.is_empty(), "다른 프로젝트 필터는 빈 결과");
}

// =====================================================
// 삭제 cascade 동작 검증 (issue_delete / epic_delete)
// =====================================================

#[tokio::test]
async fn test_issue_delete_cascades_tasks_notes_links() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    // 두 이슈, 각각 태스크/노트/링크 보유
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "A".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();
    let issue_b = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "B".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let t1 = db.task_create(CreateTaskInput {
        issue_id: issue_a.id, title: "t1".into(),
        description: None, goal: None, after_task_id: None, source: None,
    }).await.unwrap();
    db.task_test_add(t1.id, "테스트1".into()).await.unwrap();

    db.note_add(CreateNoteInput {
        issue_id: issue_a.id, task_id: None,
        note_type: NoteType::Caveat,
        summary: "A 의 caveat".into(), detail: None,
        author: None,
        agent_id: None,
        scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();

    db.issue_link(issue_a.id, issue_b.id, LinkType::Blocks).await.unwrap();

    // 사전: 모든 자식 존재 확인
    assert_eq!(db.task_list(issue_a.id, None).await.unwrap().len(), 1, "이슈 A 에 태스크 1건");
    assert_eq!(db.note_list(Some(issue_a.id), None, None, false).await.unwrap().len(), 1, "이슈 A 에 노트 1건");
    assert_eq!(db.issue_links_for(issue_a.id).await.unwrap().len(), 1, "이슈 A 에 링크 1건");

    // 이슈 A 삭제
    db.issue_delete(issue_a.id, "agent").await.unwrap();

    // 이슈 자체가 없음
    assert!(db.issue_get(issue_a.id).await.is_err(), "삭제된 이슈는 조회 실패");

    // 자식 데이터 cascade 확인
    assert!(db.task_list(issue_a.id, None).await.unwrap().is_empty(), "태스크가 모두 삭제됨");
    assert!(db.note_list(Some(issue_a.id), None, None, false).await.unwrap().is_empty(), "노트가 모두 삭제됨");
    assert!(db.issue_links_for(issue_b.id).await.unwrap().is_empty(), "이슈 B 측에서 본 링크도 cascade 됨");

    // 이슈 B 는 살아 있어야 한다
    assert!(db.issue_get(issue_b.id).await.is_ok(), "관계 없는 이슈 B 는 살아 있음");
}

#[tokio::test]
async fn test_epic_delete_cascades_all_issues_and_descendants() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    // 같은 에픽에 이슈 2개 + 다른 에픽 1개
    let other_epic = db.epic_create(CreateEpicInput {
        project_key: "test-project".into(),
        title: "Other".into(), description: None,
    }).await.unwrap();

    let i1 = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "i1".into(), description: None, goal: None, priority: None,
    }).await.unwrap();
    let i2 = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "i2".into(), description: None, goal: None, priority: None,
    }).await.unwrap();
    let other_issue = db.issue_create(CreateIssueInput {
        epic_id: other_epic.id, sprint_id: Some(sprint_id),
        title: "other".into(), description: None, goal: None, priority: None,
    }).await.unwrap();

    // i1 에 task / note 부착
    let t = db.task_create(CreateTaskInput {
        issue_id: i1.id, title: "t".into(),
        description: None, goal: None, after_task_id: None, source: None,
    }).await.unwrap();
    db.note_add(CreateNoteInput {
        issue_id: i1.id, task_id: Some(t.id),
        note_type: NoteType::Decision,
        summary: "d".into(), detail: None, author: None, agent_id: None,
        scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();

    // 에픽 삭제 — 하위 이슈/태스크/노트 모두 함께 사라져야 한다
    db.epic_delete(epic_id, "agent").await.unwrap();

    assert!(db.epic_get(epic_id).await.is_err(), "에픽이 삭제됨");
    assert!(db.issue_get(i1.id).await.is_err(), "하위 이슈 i1 cascade");
    assert!(db.issue_get(i2.id).await.is_err(), "하위 이슈 i2 cascade");
    assert!(db.task_list(i1.id, None).await.unwrap().is_empty(), "i1 의 태스크 cascade");
    assert!(db.note_list(Some(i1.id), None, None, false).await.unwrap().is_empty(), "i1 의 노트 cascade");

    // 다른 에픽/이슈는 살아 있어야 한다
    assert!(db.epic_get(other_epic.id).await.is_ok(), "다른 에픽은 살아 있음");
    assert!(db.issue_get(other_issue.id).await.is_ok(), "다른 에픽의 이슈는 살아 있음");
}

// =====================================================
// 멀티 에이전트 claim / release (CAS) 검증
// =====================================================

#[tokio::test]
async fn test_issue_claim_blocks_concurrent_claim() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "claim race".into(), description: None, goal: None, priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // Agent A 가 먼저 잡는다 → 성공
    let claimed = db.issue_claim(issue.id, "agent-a").await.unwrap();
    assert_eq!(claimed.status, IssueStatus::Working);
    assert_eq!(claimed.assigned_agent.as_deref(), Some("agent-a"));

    // Agent B 의 claim 은 실패해야 한다 (lease 가 살아있음)
    let conflict = db.issue_claim(issue.id, "agent-b").await;
    assert!(conflict.is_err(), "Agent B 는 이미 잡혀있어 claim 실패해야 함: {:?}", conflict);

    // 동일 agent-a 의 재호출은 idempotent (이미 자기가 잡은 상태)
    let same_a = db.issue_claim(issue.id, "agent-a").await;
    assert!(same_a.is_ok(), "같은 에이전트의 재 claim 은 OK");
}

#[tokio::test]
async fn test_issue_release_to_ready_clears_assigned_agent() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "release".into(), description: None, goal: None, priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready), ..Default::default()
    }, "agent").await.unwrap();

    let claimed = db.issue_claim(issue.id, "agent-a").await.unwrap();
    assert_eq!(claimed.assigned_agent.as_deref(), Some("agent-a"));

    // ready 로 release → assigned_agent 비워지고 다른 에이전트가 claim 가능
    let released = db.issue_release(issue.id, IssueStatus::Ready, "agent-a", false).await.unwrap();
    assert_eq!(released.status, IssueStatus::Ready);
    assert_eq!(released.assigned_agent, None);

    let reclaim = db.issue_claim(issue.id, "agent-b").await.unwrap();
    assert_eq!(reclaim.assigned_agent.as_deref(), Some("agent-b"));
}

#[tokio::test]
async fn test_issue_status_change_clears_assignment_when_leaving_working() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "demo flow".into(), description: None, goal: None, priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready), ..Default::default()
    }, "agent").await.unwrap();
    db.issue_claim(issue.id, "agent-a").await.unwrap();

    // working → demo 로 일반 update 호출 시에도 assigned_agent 가 정리되어야 한다
    let demoed = db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo), ..Default::default()
    }, "agent-a").await.unwrap();
    assert_eq!(demoed.status, IssueStatus::Demo);
    assert_eq!(demoed.assigned_agent, None, "working 벗어나면 assigned_agent 가 비워져야 함");
}

#[tokio::test]
async fn test_issue_delete_records_history() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "to delete".into(), description: None, goal: None, priority: None,
    }).await.unwrap();

    db.issue_delete(issue.id, "user").await.unwrap();

    // history 에 deletion 이벤트가 user 액터로 남아야 한다 (공개 API 로 조회)
    let entries = db.history_list(EntityType::Issue, issue.id).await.unwrap();
    let has_delete = entries.iter().any(|h| h.field == "deleted" && h.changed_by == "user");
    assert!(has_delete, "issue_delete 가 changed_by='user' 로 history 에 기록되어야 함, entries={:?}", entries);
}

// =====================================================
// 2차 라운드 — note.agent_id 인스턴스 식별
// =====================================================

#[tokio::test]
async fn test_note_add_persists_agent_id() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "agent_id note".into(),
        description: None, goal: None, priority: None,
    }).await.unwrap();

    // agent_id 를 명시한 노트
    let n1 = db.note_add(CreateNoteInput {
        issue_id: issue.id, task_id: None,
        note_type: NoteType::Decision,
        summary: "결정 1".into(), detail: None,
        author: Some("agent".into()),
        agent_id: Some("claude-opus@sess-A".into()),
        scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();
    assert_eq!(n1.agent_id.as_deref(), Some("claude-opus@sess-A"));

    // agent_id 생략한 노트 (호환성 — 기존 동작 유지)
    let n2 = db.note_add(CreateNoteInput {
        issue_id: issue.id, task_id: None,
        note_type: NoteType::Comment,
        summary: "코멘트".into(), detail: None,
        author: Some("user".into()),
        agent_id: None,
        scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();
    assert_eq!(n2.agent_id, None, "agent_id 미지정은 NULL 유지");

    // note_list 응답에도 agent_id 가 노출되어야 한다
    let notes = db.note_list(Some(issue.id), None, None, false).await.unwrap();
    let opus_notes: Vec<_> = notes.iter()
        .filter(|n| n.agent_id.as_deref() == Some("claude-opus@sess-A"))
        .collect();
    assert_eq!(opus_notes.len(), 1, "claude-opus 가 남긴 노트 1건 조회 가능");
}

#[tokio::test]
async fn test_issue_release_force_overrides_ownership() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "zombie lease".into(),
        description: None, goal: None, priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready), ..Default::default()
    }, "agent").await.unwrap();
    db.issue_claim(issue.id, "agent-zombie").await.unwrap();

    // 다른 호출자가 force=false 로 release 시도 → 거부
    let denied = db.issue_release(issue.id, IssueStatus::Ready, "user", false).await;
    assert!(denied.is_err(), "force=false 면 ownership 검증 — user 는 agent-zombie 의 lease 해제 불가");

    // force=true 면 회수 성공
    let recovered = db.issue_release(issue.id, IssueStatus::Ready, "user", true).await.unwrap();
    assert_eq!(recovered.status, IssueStatus::Ready);
    assert_eq!(recovered.assigned_agent, None, "강제 회수 후 assigned_agent 정리");

    // history 에 user 가 force 회수한 기록이 남는지
    let entries = db.history_list(EntityType::Issue, issue.id).await.unwrap();
    let user_release = entries.iter().any(|h| h.field == "status" && h.new_value.as_deref() == Some("ready") && h.changed_by == "user");
    assert!(user_release, "force release 가 changed_by='user' 로 history 에 남아야 함, entries={:?}", entries);
}

// =====================================================
// 2차 라운드 — broadcast scope notes
// =====================================================

#[tokio::test]
async fn test_broadcast_caveat_appears_in_session_restore() {
    use engram_core::models::NoteScope;
    let db = setup().await;
    let (_sprint_id, _epic_id) = seed_sprint_epic(&db).await;

    // project scope caveat 등록 (issue_id 무관)
    db.note_add(CreateNoteInput {
        issue_id: 0,
        task_id: None,
        note_type: NoteType::Caveat,
        summary: "lint 통과 필수".into(),
        detail: None,
        author: Some("user".into()),
        agent_id: None,
        scope: Some(NoteScope::Project),
        scope_target_id: None,
        project_key: Some("test-project".into()),
    }).await.unwrap();

    // session_restore 호출 → active_caveats 에 노출
    let snap = db.session_restore(Some("test-project")).await.unwrap();
    assert_eq!(snap.active_caveats.len(), 1, "project caveat 1건 노출");
    assert_eq!(snap.active_caveats[0].summary, "lint 통과 필수");
    assert_eq!(snap.active_caveats[0].project_key.as_deref(), Some("test-project"));

    // 다른 project 필터 → 빈 결과
    let other = db.session_restore(Some("other-project")).await.unwrap();
    assert!(other.active_caveats.is_empty(), "다른 프로젝트 필터는 broadcast caveat 미노출");
}

#[tokio::test]
async fn test_sprint_scope_note_filters_by_active_sprint() {
    use engram_core::models::NoteScope;
    let db = setup().await;
    let (sprint_id, _epic_id) = seed_sprint_epic(&db).await;

    // 활성 sprint 에 broadcast caveat
    db.note_add(CreateNoteInput {
        issue_id: 0, task_id: None,
        note_type: NoteType::Caveat,
        summary: "sprint freeze: deploy 후 non-critical merge 금지".into(),
        detail: None, author: Some("user".into()),
        agent_id: None,
        scope: Some(NoteScope::Sprint),
        scope_target_id: Some(sprint_id),
        project_key: None,
    }).await.unwrap();

    // session_restore → sprint scope caveat 노출
    let snap = db.session_restore(Some("test-project")).await.unwrap();
    let has_sprint_caveat = snap.active_caveats.iter().any(|n|
        n.summary.contains("sprint freeze") && n.scope_target_id == Some(sprint_id)
    );
    assert!(has_sprint_caveat, "활성 sprint 의 broadcast caveat 노출, active_caveats={:?}", snap.active_caveats);
}

#[tokio::test]
async fn test_history_by_agent_returns_recent_changes() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "audit subject".into(), description: None, goal: None, priority: None,
    }).await.unwrap();

    // 두 에이전트가 각자 변경
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready), ..Default::default()
    }, "agent-a").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        priority: Some(IssuePriority::High), ..Default::default()
    }, "agent-b").await.unwrap();

    // history_by_agent("agent-a") → A 의 기록만
    let a_changes = db.history_by_agent("agent-a", 50).await.unwrap();
    assert!(a_changes.iter().all(|h| h.changed_by == "agent-a"), "A 로만 필터링되어야 함");
    assert!(a_changes.iter().any(|h| h.field == "status"), "A 가 status 변경 기록 보유");

    // history_by_agent("agent-b") → B 의 기록만
    let b_changes = db.history_by_agent("agent-b", 50).await.unwrap();
    assert!(b_changes.iter().all(|h| h.changed_by == "agent-b"));
    assert!(b_changes.iter().any(|h| h.field == "priority"), "B 가 priority 변경 기록 보유");

    // history_recent — 두 변경 모두 잡힘
    let recent = db.history_recent(50, Some(60)).await.unwrap();
    let agents: std::collections::HashSet<_> = recent.iter().map(|h| h.changed_by.clone()).collect();
    assert!(agents.contains("agent-a"));
    assert!(agents.contains("agent-b"));
}

#[tokio::test]
async fn test_broadcast_note_input_validation() {
    use engram_core::models::NoteScope;
    let db = setup().await;

    // project scope 인데 project_key 누락 → Validation error
    let missing_pk = db.note_add(CreateNoteInput {
        issue_id: 0, task_id: None,
        note_type: NoteType::Caveat,
        summary: "x".into(), detail: None,
        author: None, agent_id: None,
        scope: Some(NoteScope::Project),
        scope_target_id: None,
        project_key: None,
    }).await;
    assert!(missing_pk.is_err(), "project scope + project_key 누락은 거부되어야 함");

    // sprint scope 인데 scope_target_id 누락 → Validation error
    let missing_target = db.note_add(CreateNoteInput {
        issue_id: 0, task_id: None,
        note_type: NoteType::Caveat,
        summary: "x".into(), detail: None,
        author: None, agent_id: None,
        scope: Some(NoteScope::Sprint),
        scope_target_id: None,
        project_key: None,
    }).await;
    assert!(missing_target.is_err(), "sprint scope + scope_target_id 누락은 거부되어야 함");
}

// ─── Issue #34: blocked 이슈 상태 전이 제한 통합 테스트 ───────────────────────

/// A blocks B 설정 후 B→working 시도 시 InvalidTransition, A→finished 후 B→working 성공
#[tokio::test]
async fn test_blocked_issue_cannot_transition_to_working() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    // 이슈 A (blocker) — ready 상태
    let a = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "Blocker A".into(), description: None, goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();
    db.issue_update(a.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    // 이슈 B (blocked) — ready 상태
    let b = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "Blocked B".into(), description: None, goal: None,
        priority: Some(IssuePriority::Medium),
    }).await.unwrap();
    db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    // A blocks B 링크 설정
    db.issue_link(a.id, b.id, LinkType::Blocks).await.unwrap();

    // B→working 시도: A가 아직 active(ready)이므로 InvalidTransition 에러
    let err = db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Working), ..Default::default() }, "agent").await;
    assert!(err.is_err(), "블로커가 있는 이슈는 working 으로 전환 불가해야 함");
    match err.unwrap_err() {
        engram_core::Error::InvalidTransition(msg) => {
            assert!(msg.contains(format!("#{}", a.id).as_str()), "에러 메시지에 블로커 ID 가 포함되어야 함");
        }
        other => panic!("InvalidTransition 에러여야 하는데 {:?} 발생", other),
    }

    // issue_claim 도 동일하게 차단
    let claim_err = db.issue_claim(b.id, "test-agent").await;
    assert!(claim_err.is_err(), "블로커가 있는 이슈는 claim 도 불가해야 함");

    // A→finished (사용자 전용이지만 테스트에서는 직접 DB 수준 검증을 위해 우회)
    // finished 는 agent-demo-gate 규칙상 사용자 전용이나, 도메인 레이어에 직접 강제는 없음.
    // 여기서는 demo 상태로 전환 후 finished 로 전환 (can_transition_to 통과).
    db.issue_claim(a.id, "agent-a").await.unwrap(); // A→working
    db.issue_release(a.id, IssueStatus::Demo, "agent-a", false).await.unwrap(); // A→demo
    // A가 demo 상태가 되면 B 의 블로커가 해소됨
    let ok = db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Working), ..Default::default() }, "agent").await;
    assert!(ok.is_ok(), "블로커가 demo 상태가 되면 B→working 이 가능해야 함");
}

/// blocked 이슈는 cancelled 로는 언제든 전환 가능
#[tokio::test]
async fn test_blocked_issue_can_cancel() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    let a = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "Blocker A".into(), description: None, goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();
    db.issue_update(a.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    let b = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "Blocked B".into(), description: None, goal: None,
        priority: Some(IssuePriority::Medium),
    }).await.unwrap();
    db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    db.issue_link(a.id, b.id, LinkType::Blocks).await.unwrap();

    // B→cancelled 는 블로커와 무관하게 허용
    let result = db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Cancelled), ..Default::default() }, "agent").await;
    assert!(result.is_ok(), "blocked 이슈도 cancelled 로는 전환 가능해야 함");
    assert_eq!(result.unwrap().status, IssueStatus::Cancelled);
}

/// blocked 이슈는 required ↔ ready 전환이 블로커와 무관하게 가능
#[tokio::test]
async fn test_blocked_issue_can_move_required_ready() {
    let db = setup().await;
    let (sprint_id, epic_id) = seed_sprint_epic(&db).await;

    let a = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "Blocker A".into(), description: None, goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();
    db.issue_update(a.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    // B는 required(기본값) 상태로 생성
    let b = db.issue_create(CreateIssueInput {
        epic_id, sprint_id: Some(sprint_id),
        title: "Blocked B".into(), description: None, goal: None,
        priority: Some(IssuePriority::Medium),
    }).await.unwrap();
    assert_eq!(b.status, IssueStatus::Required);

    db.issue_link(a.id, b.id, LinkType::Blocks).await.unwrap();

    // required → ready: 허용
    let to_ready = db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await;
    assert!(to_ready.is_ok(), "blocked 이슈도 required→ready 는 가능해야 함");

    // ready → required: 허용
    let to_required = db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Required), ..Default::default() }, "agent").await;
    assert!(to_required.is_ok(), "blocked 이슈도 ready→required 는 가능해야 함");
    assert_eq!(to_required.unwrap().status, IssueStatus::Required);
}
