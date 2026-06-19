use engram_core::{
    Db,
    models::{
        sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus},
        epic::CreateEpicInput,
        issue::{CreateIssueInput, UpdateIssueInput, IssueStatus, IssuePriority, IssueFilter},
        task::{CreateTaskInput, UpdateTaskInput, TaskStatus},
        note::{CreateNoteInput, NoteType, NoteScope},
        history::EntityType,
        mission::{CreateMissionInput, MissionStatus, MissionFilter},
        LinkType,
    },
};

async fn setup() -> Db {
    Db::open_in_memory().await.unwrap()
}

async fn seed_sprint_epic(db: &Db) -> (i64, i64, i64) {
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

    let mission = db.mission_create(CreateMissionInput {
        title: "Test Mission".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    let epic = db.epic_create(CreateEpicInput {
        project_key:"test-project".to_string(),
        mission_id: Some(mission.id),
        sprint_id: Some(sprint.id),
            title: "Test Epic".to_string(),
        description: None,
    }).await.unwrap();

    (sprint.id, epic.id, mission.id)
}

#[tokio::test]
async fn test_full_sprint_workflow() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 이슈 생성 (required)
    let issue = db.issue_create(CreateIssueInput { epic_id,
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
    let snapshot = db.session_restore(Some("test-project"), false, 120, None).await.unwrap();
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue_a = db.issue_create(CreateIssueInput { epic_id, title:"Issue A".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let issue_b = db.issue_create(CreateIssueInput { epic_id, title:"Issue B".to_string(), description: None, goal: None, priority: None,
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput { epic_id, title:"Issue".to_string(), description: None, goal: None, priority: None,
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

    let mission_a = db.mission_create(CreateMissionInput {
        title: "Mission A".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    let mission_b = db.mission_create(CreateMissionInput {
        title: "Mission B".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    let epic_a = db.epic_create(CreateEpicInput {
        project_key:"proj-a".to_string(),
        mission_id: Some(mission_a.id),
        sprint_id: Some(sprint.id),
            title: "Epic A".to_string(),
        description: None,
    }).await.unwrap();

    let epic_b = db.epic_create(CreateEpicInput {
        project_key:"proj-b".to_string(),
        mission_id: Some(mission_b.id),
        sprint_id: Some(sprint.id),
            title: "Epic B".to_string(),
        description: None,
    }).await.unwrap();

    // proj-a 이슈 생성 후 Ready 전환
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id: epic_a.id,
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
    let snap_a = db.session_restore(Some("proj-a"), false, 120, None).await.unwrap();
    assert_eq!(snap_a.active_epics.len(), 1, "proj-a: active_epics는 1개여야 함");
    assert_eq!(snap_a.active_epics[0].epic.project_key.as_deref(), Some("proj-a"));

    // proj-b 조회 → proj-b 에픽만
    let snap_b = db.session_restore(Some("proj-b"), false, 120, None).await.unwrap();
    assert_eq!(snap_b.active_epics.len(), 1, "proj-b: active_epics는 1개여야 함");
    assert_eq!(snap_b.active_epics[0].epic.project_key.as_deref(), Some("proj-b"));
}

#[tokio::test]
async fn test_task_next_priority_ordering() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 이슈 A: Critical
    let issue_a = db.issue_create(CreateIssueInput { epic_id,
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
    let issue_b = db.issue_create(CreateIssueInput { epic_id,
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

    let mission_a = db.mission_create(CreateMissionInput {
        title: "Mission A".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    let mission_b = db.mission_create(CreateMissionInput {
        title: "Mission B".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    let epic_a = db.epic_create(CreateEpicInput {
        project_key:"proj-a".to_string(),
        mission_id: Some(mission_a.id),
        sprint_id: Some(sprint.id),
            title: "Epic A".to_string(),
        description: None,
    }).await.unwrap();
    let epic_b = db.epic_create(CreateEpicInput {
        project_key:"proj-b".to_string(),
        mission_id: Some(mission_b.id),
        sprint_id: Some(sprint.id),
            title: "Epic B".to_string(),
        description: None,
    }).await.unwrap();

    // proj-a 이슈 A (Ready)
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id: epic_a.id,
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

    // 이슈 A를 Finished로 전환 (Required → Ready → Working → Demo → Finished)
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_finish(issue_a.id, "user").await.unwrap();

    // 이제 이슈 B의 blocker가 finished → task_next(proj-b) 태스크 반환
    let next_after = db.task_next(Some("proj-b"), None).await.unwrap();
    assert!(next_after.is_some(), "blocker가 finished 된 후 task_next는 태스크를 반환해야 함");
    assert_eq!(next_after.unwrap().task_id, task_b.id, "이슈 B의 태스크가 반환돼야 함");
}

#[tokio::test]
async fn test_scope_expansion_warning() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 이슈 생성 및 Ready 전환
    let issue = db.issue_create(CreateIssueInput { epic_id,
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

    let snapshot = db.session_restore(Some("test-project"), false, 120, None).await.unwrap();

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

    let mission = db.mission_create(CreateMissionInput {
        title: "Actor Mission".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    let epic = db.epic_create(engram_core::models::epic::CreateEpicInput {
        project_key:"actor-test".to_string(),
        mission_id: Some(mission.id),
        sprint_id: Some(sprint.id),
            title: "Actor Epic".to_string(),
        description: None,
    }).await.unwrap();

    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id,
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
    db.issue_finish(issue.id, "user").await.unwrap();

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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    db.sprint_delete(sprint_id).await.expect("이슈/에픽이 있어도 스프린트 삭제는 가능해야 함 (이슈는 백로그로 이동)");
    assert!(db.sprint_get(sprint_id).await.is_err(), "삭제된 스프린트 조회는 실패해야 함");
}

#[tokio::test]
async fn test_stalled_issues_detects_working_issue() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 이슈 두 건: 하나는 working 으로 전이, 하나는 required 로 그대로 둠
    let working_issue = db.issue_create(CreateIssueInput { epic_id, title: "Working Issue".to_string(),
        description: None, goal: None, priority: None,
    }).await.unwrap();
    db.issue_update(working_issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready), ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(working_issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Working), ..Default::default()
    }, "agent").await.unwrap();

    let _required_issue = db.issue_create(CreateIssueInput { epic_id, title: "Required Issue".to_string(),
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 두 이슈, 각각 태스크/노트/링크 보유
    let issue_a = db.issue_create(CreateIssueInput { epic_id, title: "A".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();
    let issue_b = db.issue_create(CreateIssueInput { epic_id, title: "B".to_string(), description: None, goal: None, priority: None,
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
    assert_eq!(db.note_list(Some(issue_a.id), None, None, false, true, None, None, None, None, None).await.unwrap().items.len(), 1, "이슈 A 에 노트 1건");
    assert_eq!(db.issue_links_for(issue_a.id).await.unwrap().len(), 1, "이슈 A 에 링크 1건");

    // 이슈 A 삭제
    db.issue_delete(issue_a.id, "agent").await.unwrap();

    // 이슈 자체가 없음
    assert!(db.issue_get(issue_a.id, false).await.is_err(), "삭제된 이슈는 조회 실패");

    // 자식 데이터 cascade 확인
    assert!(db.task_list(issue_a.id, None).await.unwrap().is_empty(), "태스크가 모두 삭제됨");
    assert!(db.note_list(Some(issue_a.id), None, None, false, true, None, None, None, None, None).await.unwrap().items.is_empty(), "노트가 모두 삭제됨");
    assert!(db.issue_links_for(issue_b.id).await.unwrap().is_empty(), "이슈 B 측에서 본 링크도 cascade 됨");

    // 이슈 B 는 살아 있어야 한다
    assert!(db.issue_get(issue_b.id, false).await.is_ok(), "관계 없는 이슈 B 는 살아 있음");
}

#[tokio::test]
async fn test_epic_delete_cascades_all_issues_and_descendants() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 같은 에픽에 이슈 2개 + 다른 에픽 1개
    let other_epic = db.epic_create(CreateEpicInput {
        project_key: "test-project".into(),
        mission_id: Some(mission_id),
        sprint_id: Some(sprint_id),
            title: "Other".into(), description: None,
    }).await.unwrap();

    let i1 = db.issue_create(CreateIssueInput { epic_id, title: "i1".into(), description: None, goal: None, priority: None,
    }).await.unwrap();
    let i2 = db.issue_create(CreateIssueInput { epic_id, title: "i2".into(), description: None, goal: None, priority: None,
    }).await.unwrap();
    let other_issue = db.issue_create(CreateIssueInput {
        epic_id: other_epic.id, title: "other".into(), description: None, goal: None, priority: None,
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
    assert!(db.issue_get(i1.id, false).await.is_err(), "하위 이슈 i1 cascade");
    assert!(db.issue_get(i2.id, false).await.is_err(), "하위 이슈 i2 cascade");
    assert!(db.task_list(i1.id, None).await.unwrap().is_empty(), "i1 의 태스크 cascade");
    assert!(db.note_list(Some(i1.id), None, None, false, true, None, None, None, None, None).await.unwrap().items.is_empty(), "i1 의 노트 cascade");

    // 다른 에픽/이슈는 살아 있어야 한다
    assert!(db.epic_get(other_epic.id).await.is_ok(), "다른 에픽은 살아 있음");
    assert!(db.issue_get(other_issue.id, false).await.is_ok(), "다른 에픽의 이슈는 살아 있음");
}

// =====================================================
// 멀티 에이전트 claim / release (CAS) 검증
// =====================================================

#[tokio::test]
async fn test_issue_claim_blocks_concurrent_claim() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput { epic_id, title: "claim race".into(), description: None, goal: None, priority: None,
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput { epic_id, title: "release".into(), description: None, goal: None, priority: None,
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput { epic_id, title: "demo flow".into(), description: None, goal: None, priority: None,
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput { epic_id, title: "to delete".into(), description: None, goal: None, priority: None,
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput { epic_id, title: "agent_id note".into(),
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
    let notes = db.note_list(Some(issue.id), None, None, false, true, None, None, None, None, None).await.unwrap().items;
    let opus_notes: Vec<_> = notes.iter()
        .filter(|n| n.agent_id.as_deref() == Some("claude-opus@sess-A"))
        .collect();
    assert_eq!(opus_notes.len(), 1, "claude-opus 가 남긴 노트 1건 조회 가능");
}

#[tokio::test]
async fn test_issue_release_force_overrides_ownership() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput { epic_id, title: "zombie lease".into(),
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

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
    let snap = db.session_restore(Some("test-project"), false, 120, None).await.unwrap();
    assert_eq!(snap.active_caveats.len(), 1, "project caveat 1건 노출");
    assert_eq!(snap.active_caveats[0].summary, "lint 통과 필수");
    assert_eq!(snap.active_caveats[0].project_key.as_deref(), Some("test-project"));

    // 다른 project 필터 → 빈 결과
    let other = db.session_restore(Some("other-project"), false, 120, None).await.unwrap();
    assert!(other.active_caveats.is_empty(), "다른 프로젝트 필터는 broadcast caveat 미노출");
}

#[tokio::test]
async fn test_sprint_scope_note_filters_by_active_sprint() {
    use engram_core::models::NoteScope;
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

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
    let snap = db.session_restore(Some("test-project"), false, 120, None).await.unwrap();
    let has_sprint_caveat = snap.active_caveats.iter().any(|n|
        n.summary.contains("sprint freeze") && n.scope_target_id == Some(sprint_id)
    );
    assert!(has_sprint_caveat, "활성 sprint 의 broadcast caveat 노출, active_caveats={:?}", snap.active_caveats);
}

#[tokio::test]
async fn test_history_by_agent_returns_recent_changes() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput { epic_id, title: "audit subject".into(), description: None, goal: None, priority: None,
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

/// A blocks B 설정 후 B→working 시도 시 Conflict, A→finished 후 B→working 성공
#[tokio::test]
async fn test_blocked_issue_cannot_transition_to_working() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 이슈 A (blocker) — ready 상태
    let a = db.issue_create(CreateIssueInput { epic_id, title: "Blocker A".into(), description: None, goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();
    db.issue_update(a.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    // 이슈 B (blocked) — ready 상태
    let b = db.issue_create(CreateIssueInput { epic_id, title: "Blocked B".into(), description: None, goal: None,
        priority: Some(IssuePriority::Medium),
    }).await.unwrap();
    db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    // A blocks B 링크 설정
    db.issue_link(a.id, b.id, LinkType::Blocks).await.unwrap();

    // B→working 시도: A가 아직 active(ready)이므로 Conflict 에러
    let err = db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Working), ..Default::default() }, "agent").await;
    assert!(err.is_err(), "블로커가 있는 이슈는 working 으로 전환 불가해야 함");
    match err.unwrap_err() {
        engram_core::Error::Conflict(msg) => {
            assert!(msg.contains(format!("#{}", a.id).as_str()), "에러 메시지에 블로커 ID 가 포함되어야 함");
        }
        other => panic!("Conflict 에러여야 하는데 {:?} 발생", other),
    }

    // issue_claim 도 동일하게 차단
    let claim_err = db.issue_claim(b.id, "test-agent").await;
    assert!(claim_err.is_err(), "블로커가 있는 이슈는 claim 도 불가해야 함");

    // A→finished: repository layer 에서 changed_by="user" 일 때만 허용.
    // 여기서는 demo 상태로 전환 후 블로커 해소 여부만 검증하므로 demo 까지만 이동.
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
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let a = db.issue_create(CreateIssueInput { epic_id, title: "Blocker A".into(), description: None, goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();
    db.issue_update(a.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    let b = db.issue_create(CreateIssueInput { epic_id, title: "Blocked B".into(), description: None, goal: None,
        priority: Some(IssuePriority::Medium),
    }).await.unwrap();
    db.issue_update(b.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    db.issue_link(a.id, b.id, LinkType::Blocks).await.unwrap();

    // B→cancelled 는 블로커와 무관하게 허용 (사용자 권한으로)
    let result = db.issue_cancel(b.id, "cancelled by user", "user").await;
    assert!(result.is_ok(), "blocked 이슈도 cancelled 로는 전환 가능해야 함");
    assert_eq!(result.unwrap().status, IssueStatus::Cancelled);
}

/// blocked 이슈는 required ↔ ready 전환이 블로커와 무관하게 가능
#[tokio::test]
async fn test_blocked_issue_can_move_required_ready() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let a = db.issue_create(CreateIssueInput { epic_id, title: "Blocker A".into(), description: None, goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();
    db.issue_update(a.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    // B는 required(기본값) 상태로 생성
    let b = db.issue_create(CreateIssueInput { epic_id, title: "Blocked B".into(), description: None, goal: None,
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

// ─── Issue #95: planning_review_queue 한국어 description 패닉 회귀 테스트 ───

/// 한국어(멀티바이트 UTF-8) description이 100바이트 초과일 때 planning_review_queue가
/// 패닉 없이 정상 응답을 반환해야 한다. (&d[..100] 바이트 슬라이싱 버그 재발 방지)
#[tokio::test]
async fn test_planning_review_queue_multibyte_description_no_panic() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 한국어 글자 1자 = UTF-8 3바이트.
    // 40자 → 120바이트: d.len()>100 이지만 chars().count()==40 이므로 excerpt 불필요
    let short_korean = "가".repeat(40);
    assert!(short_korean.chars().count() <= 100);
    assert!(short_korean.len() > 100, "바이트 길이는 100 초과여야 함");

    let issue1 = db.issue_create(CreateIssueInput { epic_id,
        title: "멀티바이트 테스트 이슈".into(),
        description: Some(short_korean),
        goal: None,
        priority: Some(IssuePriority::Medium),
    }).await.unwrap();

    assert!(issue1.mission_id.is_some(), "이슈1의 mission_id 가 Some 이어야 함");
    assert_eq!(issue1.sprint_id, Some(sprint_id), "이슈1의 sprint_id 가 Some(sprint_id) 이어야 함");

    // 110자 한국어 → chars().count()>100 → excerpt 생성 경로
    let long_korean = "가".repeat(110);
    let issue2 = db.issue_create(CreateIssueInput { epic_id,
        title: "긴 한국어 설명 이슈".into(),
        description: Some(long_korean),
        goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();

    assert!(issue2.mission_id.is_some(), "이슈2의 mission_id 가 Some 이어야 함");
    assert_eq!(issue2.sprint_id, Some(sprint_id), "이슈2의 sprint_id 가 Some(sprint_id) 이어야 함");


    // 패닉 없이 성공해야 함 (버그 수정 전에는 여기서 panic → MCP 소켓 크래시)
    let snapshot = db.planning_review_queue("test-project", Some(sprint_id), None).await;
    assert!(snapshot.is_ok(), "한국어 description이 있어도 패닉 없이 성공해야 함");

    let snapshot = snapshot.unwrap();
    let long_item = snapshot.issues.iter()
        .find(|i| i.title == "긴 한국어 설명 이슈")
        .expect("긴 이슈가 결과에 있어야 함");

    let excerpt = long_item.description_excerpt.as_deref().unwrap_or("");
    assert!(excerpt.ends_with("..."), "100자 초과 description은 '...'로 끝나야 함");
    let body = excerpt.trim_end_matches("...");
    assert_eq!(body.chars().count(), 100, "excerpt 본문은 정확히 100자여야 함");
}

// =====================================================
// mission_id 필드 포함 검증 (이슈 #156)
// =====================================================

#[tokio::test]
async fn test_epic_model_includes_mission_id() {
    let db = setup().await;
    let epic = db.epic_create(CreateEpicInput {
        project_key: "test".to_string(),
        mission_id: None,
        sprint_id: None,
            title: "Epic 1".to_string(),
        description: None,
    }).await.unwrap();
    let _ = epic.mission_id;
    assert!(epic.mission_id.is_none(), "mission_id 필드가 Epic에 포함되어야 함");
}

#[tokio::test]
async fn test_issue_model_includes_mission_id() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput { epic_id,
        title: "Issue with mission_id".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    // mission_id 필드가 Issue 구조체에 존재하고 직렬화/역직렬화 가능해야 함
    let _ = issue.mission_id; // 필드 접근으로 컴파일 타임 검증
    // DB에서 다시 읽어도 mission_id 필드가 포함되어야 함
    let fetched = db.issue_get(issue.id, false).await.unwrap();
    let _ = fetched.mission_id;
    let fetched_compact = db.issue_get(issue.id, true).await.unwrap();
    let _ = fetched_compact.mission_id;
}

// =====================================================
// M6 미션 레이어 통합 테스트 (이슈 #139)
// =====================================================

/// 미션 → 에픽(mission_id 명시) → 이슈(자동 상속) → progress_rate 실시간 계산
#[tokio::test]
async fn test_mission_inheritance_workflow() {
    let db = setup().await;

    // 미션 생성 (백로그, sprint_id=None)
    let m = db.mission_create(CreateMissionInput {
        title: "M6 릴리즈".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();
    assert_eq!(m.status, MissionStatus::Active, "신규 미션은 active 상태여야 함");

    // 에픽 생성 — mission_id 명시
    let epic = db.epic_create(CreateEpicInput {
        project_key: "engram".to_string(),
        mission_id: Some(m.id),
        sprint_id: None,
            title: "Core Engine".to_string(),
        description: None,
    }).await.unwrap();
    assert_eq!(epic.mission_id, Some(m.id), "epic.mission_id = mission.id");

    // 이슈 생성 — mission_id=None, 부모 epic에서 자동 상속
    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id,
        title: "DB 마이그레이션".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    assert_eq!(issue.mission_id, Some(m.id), "issue.mission_id = epic.mission_id (자동 상속)");

    // 이슈가 1개이고 아직 required 상태 → progress_rate = 0.0
    let progress_before = db.mission_progress_query(m.id).await.unwrap();
    assert_eq!(progress_before.issues_count, 1, "이슈 1건이어야 함");
    assert_eq!(progress_before.finished_issues, 0, "아직 완료 없음");
    assert_eq!(progress_before.epics_count, 1, "에픽 1건이어야 함");
    assert!((progress_before.progress_rate - 0.0).abs() < 0.001, "progress_rate = 0.0 (0%)");

    // 이슈 완료 처리 (required → ready → working → demo → finished)
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "test").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "test").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo),
        ..Default::default()
    }, "test").await.unwrap();
    db.issue_finish(issue.id, "user").await.unwrap();

    // progress_rate 검증: finished=1, total=1 → 1.0 (100%)
    let progress = db.mission_progress_query(m.id).await.unwrap();
    assert_eq!(progress.issues_count, 1, "이슈 1건");
    assert_eq!(progress.finished_issues, 1, "완료 이슈 1건");
    assert!((progress.progress_rate - 1.0).abs() < 0.001, "progress_rate = 1.0 (100%)");
    assert_eq!(progress.epics_count, 1, "에픽 1건");
}

/// epic.mission_id가 NULL이면 issue.mission_id도 NULL (자동 상속 = NULL 전파)
#[tokio::test]
async fn test_mission_issue_null_mission_id_when_epic_has_none() {
    let db = setup().await;

    let epic = db.epic_create(CreateEpicInput {
        project_key: "test".to_string(),
        mission_id: None,
        sprint_id: None,
            title: "Epic without mission".to_string(),
        description: None,
    }).await.unwrap();
    assert!(epic.mission_id.is_none(), "epic.mission_id가 None이어야 함");

    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id,
        title: "Issue".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    assert!(issue.mission_id.is_none(), "epic.mission_id=NULL이면 issue.mission_id도 NULL");

    // DB 재조회해도 동일
    let fetched = db.issue_get(issue.id, false).await.unwrap();
    assert!(fetched.mission_id.is_none(), "DB 재조회 후에도 mission_id=NULL 유지");
}

/// 여러 epic + 여러 issue, progress 카운터 정확성 검증
#[tokio::test]
async fn test_mission_progress_with_multiple_epics() {
    let db = setup().await;

    let m = db.mission_create(CreateMissionInput {
        title: "Multi Epic Mission".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    // epic 2개, 각각 이슈 2개 (1 finished + 1 required)
    for i in 0..2_i32 {
        let epic = db.epic_create(CreateEpicInput {
            project_key: "test".to_string(),
            mission_id: Some(m.id),
            sprint_id: None,
            title: format!("Epic {i}"),
            description: None,
        }).await.unwrap();

        // finished 이슈
        let done = db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            title: format!("Done issue {i}"),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();
        db.issue_update(done.id, UpdateIssueInput {
            status: Some(IssueStatus::Ready),
            ..Default::default()
        }, "test").await.unwrap();
        db.issue_update(done.id, UpdateIssueInput {
            status: Some(IssueStatus::Working),
            ..Default::default()
        }, "test").await.unwrap();
        db.issue_update(done.id, UpdateIssueInput {
            status: Some(IssueStatus::Demo),
            ..Default::default()
        }, "test").await.unwrap();
        db.issue_finish(done.id, "user").await.unwrap();

        // required 이슈 (미완료)
        db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            title: format!("Todo issue {i}"),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();
    }

    let p = db.mission_progress_query(m.id).await.unwrap();
    assert_eq!(p.epics_count, 2, "에픽 2건이어야 함");
    assert_eq!(p.issues_count, 4, "총 이슈 4건이어야 함");
    assert_eq!(p.finished_issues, 2, "완료 이슈 2건이어야 함");
    assert_eq!(p.todo_issues, 2, "미완료(required) 이슈 2건이어야 함");
    assert!((p.progress_rate - 0.5).abs() < 0.001, "progress_rate = 0.5 (50%), got {}", p.progress_rate);
}

#[tokio::test]
async fn test_session_restore_includes_active_missions() {
    let db = setup().await;

    // 1. 스프린트 생성 + 활성화
    let sprint = db.sprint_create(CreateSprintInput {
        name: "Mission Sprint".to_string(),
        goal: None,
        start_date: None,
        end_date: None,
    }).await.unwrap();
    db.sprint_update(sprint.id, UpdateSprintInput {
        status: Some(SprintStatus::Active),
        ..Default::default()
    }, "agent").await.unwrap();

    // 2. 미션 생성
    let mission = db.mission_create(CreateMissionInput {
        title: "Test Mission".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    // 3. 에픽 생성 (mission_id 연결) — CreateEpicInput에 sprint_id 없음, 이슈에서 직접 지정
    let epic = db.epic_create(CreateEpicInput {
        project_key: "test-proj".to_string(),
        sprint_id: Some(sprint.id),
            title: "Mission Epic".to_string(),
        description: None,
        mission_id: Some(mission.id),
    }).await.unwrap();

    // 4. 이슈 2건 생성 (둘 다 sprint에 속하게)
    let issue1 = db.issue_create(CreateIssueInput {
        epic_id: epic.id,
        title: "Issue 1 (finished)".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    let issue2 = db.issue_create(CreateIssueInput {
        epic_id: epic.id,
        title: "Issue 2 (ready)".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    // 5. issue1 → finished 전이
    db.issue_update(issue1.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue1.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue1.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_finish(issue1.id, "user").await.unwrap();

    // issue2는 ready 상태로 유지
    db.issue_update(issue2.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // 6. session_restore 호출 — project_key 필터 없이 전체
    let snapshot = db.session_restore(None, false, 120, None).await.unwrap();

    assert_eq!(snapshot.active_missions.len(), 1, "active 미션 1건이어야 함");
    let m_summary = &snapshot.active_missions[0];
    assert_eq!(m_summary.id, mission.id, "미션 id 일치");
    assert_eq!(m_summary.title, "Test Mission", "미션 제목 일치");
    assert_eq!(m_summary.epic_count, 1, "에픽 1건");
    // finished 1건 / total 2건 = 0.5
    assert!(
        (m_summary.progress_rate - 0.5).abs() < 0.001,
        "progress_rate = 0.5 expected, got {}",
        m_summary.progress_rate
    );

    // 7. project_key 필터 — 일치하는 프로젝트
    let snap_filtered = db.session_restore(Some("test-proj"), false, 120, None).await.unwrap();
    assert_eq!(snap_filtered.active_missions.len(), 1, "test-proj 필터 시 미션 1건");

    // 8. project_key 필터 — 다른 프로젝트는 미션 포함 안 됨
    let snap_other = db.session_restore(Some("other-proj"), false, 120, None).await.unwrap();
    assert_eq!(snap_other.active_missions.len(), 0, "other-proj 필터 시 미션 0건");

    // 9. 미션 완료 처리 시 active_missions에서 제외
    db.mission_update(mission.id, engram_core::models::mission::UpdateMissionInput {
        status: Some(MissionStatus::Completed),
        ..Default::default()
    }, "agent").await.unwrap();
    let snap_after = db.session_restore(None, false, 120, None).await.unwrap();
    assert_eq!(snap_after.active_missions.len(), 0, "completed 미션은 active_missions에 포함 안 됨");
}

// =====================================================
// Issue #171: Demo gate 코드 강제 테스트 (agent_demo_gate 규칙 적용 확인)
// =====================================================

/// agent 는 finished 전이를 시도해도 Validation 에러로 차단된다.
#[tokio::test]
async fn test_demo_gate_blocks_agent_finish() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "demo gate finish test".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    // ready → working 전이 (agent 허용)
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo),
        ..Default::default()
    }, "agent").await.unwrap();

    // agent 가 finished 로 직접 전이 시도 → Validation 에러
    let result = db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Finished),
        ..Default::default()
    }, "agent").await;
    assert!(result.is_err(), "agent 는 finished 전이 불가해야 함");
    match result.unwrap_err() {
        engram_core::Error::Validation(msg) => {
            assert!(msg.contains("finished") || msg.contains("agent_demo_gate"),
                "에러 메시지에 finished/agent_demo_gate 가 언급되어야 함: {msg}");
        }
        other => panic!("Validation 에러여야 하는데 {:?} 발생", other),
    }
}

/// user 는 finished 전이를 할 수 있다.
#[tokio::test]
async fn test_demo_gate_allows_user_finish() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "user finish test".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo),
        ..Default::default()
    }, "agent").await.unwrap();

    // user 가 finished 로 전이 → 성공
    let result = db.issue_finish(issue.id, "user").await;
    assert!(result.is_ok(), "user 는 finished 전이 가능해야 함: {:?}", result.err());
    assert_eq!(result.unwrap().status, IssueStatus::Finished);
}

/// agent 는 demo 상태로 전이할 수 있다 (gate 는 finished/cancelled 만 차단).
#[tokio::test]
async fn test_demo_gate_allows_agent_demo() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "agent demo allowed test".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "agent").await.unwrap();

    // agent 가 demo 로 전이 → 성공 (gate 는 이를 허용해야 함)
    let result = db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo),
        ..Default::default()
    }, "agent").await;
    assert!(result.is_ok(), "agent 는 demo 전이 가능해야 함 (gate 는 finished/cancelled 만 차단)");
    assert_eq!(result.unwrap().status, IssueStatus::Demo);
}

/// agent 는 cancelled 전이도 차단된다.
#[tokio::test]
async fn test_demo_gate_blocks_agent_cancel() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "agent cancel blocked test".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // agent 가 cancelled 로 직접 전이 시도 → Validation 에러
    let result = db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Cancelled),
        ..Default::default()
    }, "agent").await;
    assert!(result.is_err(), "agent 는 cancelled 전이도 불가해야 함");
    match result.unwrap_err() {
        engram_core::Error::Validation(msg) => {
            assert!(msg.contains("cancelled") || msg.contains("agent_demo_gate"),
                "에러 메시지에 cancelled/agent_demo_gate 가 언급되어야 함: {msg}");
        }
        other => panic!("Validation 에러여야 하는데 {:?} 발생", other),
    }
}

// =====================================================
// Issue #175: Error::Conflict 분기 / CAS race / release ownership
// =====================================================

/// 두 에이전트가 동일 이슈를 claim 할 때 두 번째 에이전트는 Error::Conflict 를 받는다.
#[tokio::test]
async fn test_issue_claim_cas_race_returns_conflict() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "cas race test".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // agent_a 가 먼저 claim → 성공
    db.issue_claim(issue.id, "agent_a").await.unwrap();

    // agent_b 가 동일 이슈 claim 시도 → Error::Conflict (exit 4)
    let err = db.issue_claim(issue.id, "agent_b").await.unwrap_err();
    match err {
        engram_core::Error::Conflict(_) => {} // 예상된 결과
        other => panic!("Conflict 에러여야 하는데 {:?} 발생 — exit 4 보장 필요", other),
    }
}

/// 동일 에이전트가 이미 claim 한 이슈를 재호출하면 idempotent (성공).
#[tokio::test]
async fn test_issue_claim_idempotent_same_agent() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "idempotent claim test".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // 첫 claim
    let first = db.issue_claim(issue.id, "agent_a").await.unwrap();
    assert_eq!(first.assigned_agent.as_deref(), Some("agent_a"));

    // 동일 agent_a 재호출 → 성공 (idempotent)
    let second = db.issue_claim(issue.id, "agent_a").await.unwrap();
    assert_eq!(second.assigned_agent.as_deref(), Some("agent_a"),
        "동일 에이전트의 재호출은 성공해야 함 (idempotent)");
    assert_eq!(second.status, IssueStatus::Working);
}

/// agent_a 가 claim 한 이슈를 agent_b 가 release 시도하면 Error::Conflict 로 거부된다.
#[tokio::test]
async fn test_issue_release_wrong_agent_returns_conflict() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "release ownership test".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // agent_a 가 claim
    db.issue_claim(issue.id, "agent_a").await.unwrap();

    // agent_b 가 force=false 로 release 시도 → Error::Conflict
    let err = db.issue_release(issue.id, IssueStatus::Ready, "agent_b", false).await.unwrap_err();
    match err {
        engram_core::Error::Conflict(msg) => {
            assert!(msg.contains("agent_a") || msg.contains("agent_b"),
                "에러 메시지에 소유자/요청자 정보가 있어야 함: {msg}");
        }
        other => panic!("Conflict 에러여야 하는데 {:?} 발생", other),
    }
}

/// 동일한 입력으로 mission_create를 여러 번 호출할 때 각각 고유한 ID를 가진 Mission이
/// 즉시 반환되고 NotFound가 발생하지 않는지 검증합니다. (WAL 가시성 지연 회피 검증)
#[tokio::test]
async fn test_mission_create_returns_inserted_row() {
    let db = setup().await;

    // 3회 생성 시도
    let m1 = db.mission_create(CreateMissionInput {
        title: "Test Mission 1".to_string(),
        description: None,
        jira_key: Some("TM-1".to_string()),
    }).await.unwrap();

    let m2 = db.mission_create(CreateMissionInput {
        title: "Test Mission 2".to_string(),
        description: None,
        jira_key: Some("TM-2".to_string()),
    }).await.unwrap();

    let m3 = db.mission_create(CreateMissionInput {
        title: "Test Mission 3".to_string(),
        description: None,
        jira_key: Some("TM-3".to_string()),
    }).await.unwrap();

    // ID가 모두 유효하고 고유한지 검증
    assert!(m1.id > 0);
    assert!(m2.id > 0);
    assert!(m3.id > 0);
    assert_ne!(m1.id, m2.id);
    assert_ne!(m2.id, m3.id);
    assert_ne!(m1.id, m3.id);

    // 즉시 조회가 정상적으로 수행되는지 검증 (WAL 가시성 지연 회피 확인)
    let get_m1 = db.mission_get(m1.id).await.unwrap();
    assert_eq!(get_m1.title, "Test Mission 1");
    let get_m2 = db.mission_get(m2.id).await.unwrap();
    assert_eq!(get_m2.title, "Test Mission 2");
    let get_m3 = db.mission_get(m3.id).await.unwrap();
    assert_eq!(get_m3.title, "Test Mission 3");
}

#[tokio::test]
async fn test_issue_finish_success() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "finish test".into(),
        description: None,
        goal: None,
        priority: None,
        }).await.unwrap();

    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Working), ..Default::default() }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Demo), ..Default::default() }, "agent").await.unwrap();

    let res = db.issue_finish(issue.id, "user").await;
    assert!(res.is_ok());
    let updated = res.unwrap();
    assert_eq!(updated.status, IssueStatus::Finished);
}

#[tokio::test]
async fn test_issue_cancel_success() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "cancel test".into(),
        description: None,
        goal: None,
        priority: None,
        }).await.unwrap();

    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();
    db.issue_claim(issue.id, "agent-1").await.unwrap();

    let res = db.issue_cancel(issue.id, "No longer needed", "user").await;
    assert!(res.is_ok());
    let updated = res.unwrap();
    assert_eq!(updated.status, IssueStatus::Cancelled);
    assert!(updated.assigned_agent.is_none(), "취소 시 assigned_agent 는 NULL 로 정리되어야 함");
}

#[tokio::test]
async fn test_issue_finish_rejects_non_user() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "finish reject test".into(),
        description: None,
        goal: None,
        priority: None,
        }).await.unwrap();

    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Working), ..Default::default() }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Demo), ..Default::default() }, "agent").await.unwrap();

    let res = db.issue_finish(issue.id, "agent-1").await;
    assert!(res.is_err(), "non-user 는 finish 호출 불가");
}

#[tokio::test]
async fn test_issue_cancel_rejects_non_user() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "cancel reject test".into(),
        description: None,
        goal: None,
        priority: None,
        }).await.unwrap();

    let res = db.issue_cancel(issue.id, "test", "agent-1").await;
    assert!(res.is_err(), "non-user 는 cancel 호출 불가");
}

#[tokio::test]
async fn test_issue_finish_rejects_non_demo() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "finish non-demo test".into(),
        description: None,
        goal: None,
        priority: None,
        }).await.unwrap();

    db.issue_update(issue.id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();

    let res = db.issue_finish(issue.id, "user").await;
    assert!(res.is_err(), "demo 상태가 아닌 경우 finish 전이 불가");
}

#[tokio::test]
async fn test_issue_sprint_id_follows_mission() {
    let db = setup().await;

    // 1. 스프린트 S1, S2 생성
    let s1 = db.sprint_create(CreateSprintInput {
        name: "Sprint 1".to_string(),
        goal: None,
        start_date: None,
        end_date: None,
    }).await.unwrap();

    let s2 = db.sprint_create(CreateSprintInput {
        name: "Sprint 2".to_string(),
        goal: None,
        start_date: None,
        end_date: None,
    }).await.unwrap();

    // 2. 미션 M1 생성 (sprint_id: Some(s1.id))
    let mission = db.mission_create(CreateMissionInput {
        title: "Mission 1".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    // 3. 에픽 E1 생성 (mission_id: Some(mission.id))
    let epic = db.epic_create(CreateEpicInput {
        project_key: "proj".to_string(),
        mission_id: Some(mission.id),
        sprint_id: None,
            title: "Epic 1".to_string(),
        description: None,
    }).await.unwrap();

    // 4. 이슈 생성 (sprint_id 명시하지 않고 mission_id 에 의해 상속 및 derived 처리되도록 유도)
    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id,
        // sprint_id 직접 지정 안 함
        title: "Issue 1".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    // ADR-0014: Epic 이 sprint SSOT — Epic.sprint_id 가 None 이면 이슈도 None
    let fetched1 = db.issue_get(issue.id, false).await.unwrap();
    assert_eq!(fetched1.sprint_id, None, "epic.sprint_id 미지정 → 이슈 sprint_id 도 None");

    // Epic 을 S1 으로 이동 → 이슈도 자동 따라옴
    db.epic_set_sprint(epic.id, Some(s1.id), "user").await.unwrap();
    let fetched2 = db.issue_get(issue.id, false).await.unwrap();
    assert_eq!(fetched2.sprint_id, Some(s1.id), "epic.sprint_id 변경 후 이슈 sprint_id 도 동기화");

    // Epic 을 S2 로 이동
    db.epic_set_sprint(epic.id, Some(s2.id), "user").await.unwrap();
    let fetched3 = db.issue_get(issue.id, false).await.unwrap();
    assert_eq!(fetched3.sprint_id, Some(s2.id));

    // Epic 을 백로그로 — 이슈도 백로그
    db.epic_set_sprint(epic.id, None, "user").await.unwrap();
    let fetched4 = db.issue_get(issue.id, false).await.unwrap();
    assert_eq!(fetched4.sprint_id, None);
    let _ = (s1.id, s2.id, mission.id);
}

#[tokio::test]
async fn test_mission_spans_multiple_sprints() {
    // ADR-0014: 한 Mission 산하 Epic 들이 서로 다른 sprint 에 속할 수 있다.
    let db = setup().await;

    let s1 = db.sprint_create(CreateSprintInput {
        name: "S1".to_string(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    let s2 = db.sprint_create(CreateSprintInput {
        name: "S2".to_string(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    db.sprint_update(s1.id, UpdateSprintInput {
        status: Some(SprintStatus::Active), ..Default::default()
    }, "user").await.unwrap();

    let mission = db.mission_create(CreateMissionInput {
        title: "Long Initiative".to_string(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    let epic_a = db.epic_create(CreateEpicInput {
        project_key: "p".to_string(),
        mission_id: Some(mission.id),
        sprint_id: Some(s1.id),
        title: "Phase A".to_string(),
        description: None,
    }).await.unwrap();
    let epic_b = db.epic_create(CreateEpicInput {
        project_key: "p".to_string(),
        mission_id: Some(mission.id),
        sprint_id: Some(s2.id),
        title: "Phase B".to_string(),
        description: None,
    }).await.unwrap();

    let tree = db.mission_get_tree(mission.id).await.unwrap();
    assert_eq!(tree.epics.len(), 2, "한 미션에 두 에픽");
    let mut sprints: Vec<i64> = tree.epics.iter().filter_map(|e| e.epic.sprint_id).collect();
    sprints.sort();
    assert_eq!(sprints, vec![s1.id, s2.id], "각 에픽이 서로 다른 sprint 에 속함");
    let _ = (epic_a.id, epic_b.id);
}

#[tokio::test]
async fn test_schema_sprint_ssot_is_epic() {
    // ADR-0014: Sprint SSOT 가 Epic. mission.sprint_id / issue.mission_id / issue.sprint_id 모두 제거.
    let db = setup().await;

    async fn cols(db: &Db, table: &str) -> Vec<String> {
        sqlx::query_as::<_, (String,)>("SELECT name FROM pragma_table_info(?)")
            .bind(table)
            .fetch_all(db.pool())
            .await
            .unwrap()
            .into_iter()
            .map(|(n,)| n)
            .collect()
    }

    let issues = cols(&db, "issues").await;
    assert!(!issues.contains(&"sprint_id".to_string()), "issues.sprint_id 는 제거됨");
    assert!(!issues.contains(&"mission_id".to_string()), "issues.mission_id 는 제거됨 (ADR-0014)");

    let epics = cols(&db, "epics").await;
    assert!(epics.contains(&"sprint_id".to_string()), "epics.sprint_id 는 추가됨 (ADR-0014)");
    assert!(epics.contains(&"mission_id".to_string()), "epics.mission_id 는 유지");

    let missions = cols(&db, "missions").await;
    assert!(!missions.contains(&"sprint_id".to_string()), "missions.sprint_id 는 제거됨 (ADR-0014)");
}

#[tokio::test]
async fn test_session_restore_size_guard() {
    let db = setup().await;
    let (_sprint_id, epic_id, _mission_id) = seed_sprint_epic(&db).await;

    for i in 1..=3 {
        let issue = db.issue_create(CreateIssueInput {
            epic_id,
            title: format!("Issue {}", i),
            description: Some("Long description text to inflate size".repeat(10)),
            goal: None,
            priority: None,
        }).await.unwrap();
        
        db.issue_update(issue.id, UpdateIssueInput {
            status: Some(IssueStatus::Ready),
            ..Default::default()
        }, "agent").await.unwrap();
    }

    let snap_full = db.session_restore(Some("test-project"), false, 120, None).await.unwrap();
    assert!(!snap_full.truncated);
    assert_eq!(snap_full.active_epics.len(), 1);

    let snap_tr = db.session_restore(Some("test-project"), false, 120, Some(300)).await.unwrap();
    assert!(snap_tr.truncated);
    assert_eq!(snap_tr.active_epics.len(), 0);
    assert!(snap_tr.truncated_count.unwrap() > 0);
    assert!(snap_tr.warnings.iter().any(|w| w.contains("크기 제한")));
}

#[tokio::test]
async fn test_session_restore_five_projects_regression() {
    let db = setup().await;

    // 5개 프로젝트 각각의 활성 스프린트, 에픽, 이슈 생성
    for p_idx in 1..=5 {
        let proj_key = format!("proj-{}", p_idx);
        
        // 1. 스프린트 생성
        let sprint = db.sprint_create(engram_core::models::CreateSprintInput {
            name: format!("Sprint {}", proj_key),
            goal: None,
            start_date: None,
            end_date: None,
        }).await.unwrap();

        // 2. 미션 생성
        let mission = db.mission_create(engram_core::models::CreateMissionInput {
            title: format!("Mission for {}", proj_key),
            description: None,
            jira_key: None,
        }).await.unwrap();

        // 3. 에픽 생성
        let epic = db.epic_create(engram_core::models::CreateEpicInput {
            project_key: proj_key.clone(),
            title: format!("Epic for {}", proj_key),
            description: Some("Long epic description text to test payload size. ".repeat(10)),
            mission_id: Some(mission.id),
            sprint_id: Some(sprint.id),
        }).await.unwrap();

        // 4. 이슈 3개씩 생성 및 ready 상태로 전이
        for i in 1..=3 {
            let issue = db.issue_create(engram_core::models::CreateIssueInput {
                epic_id: epic.id,
                title: format!("Issue {} in {}", i, proj_key),
                description: Some("Very long issue description text that could inflate the payload size. ".repeat(20)),
                goal: Some("Very long issue goal text that could inflate the payload size. ".repeat(20)),
                priority: None,
            }).await.unwrap();

            db.issue_update(issue.id, engram_core::models::UpdateIssueInput {
                status: Some(engram_core::models::IssueStatus::Ready),
                ..Default::default()
            }, "agent").await.unwrap();
        }
    }

    // 5. session_restore(project_key=None, compact=true)로 전역 세션 복원 호출
    let snap = db.session_restore(None, true, 120, None).await.unwrap();

    // 6. 응답 전체 크기 검증 (25,000자 이내)
    let serialized = serde_json::to_string(&snap).unwrap();
    assert!(serialized.len() < 25000, "Compact session restore payload is too large: {} chars", serialized.len());

    // 7. compact/unrelated 절단 검증: 모든 에픽의 description 과 모든 이슈의 description/goal 이 200자 이하로 절단되었는지 확인
    for epic_snap in &snap.active_epics {
        if let Some(ref desc) = epic_snap.epic.description {
            assert!(desc.len() <= 205, "Epic description not truncated correctly: {}", desc.len());
        }

        if let Some(ref issues_compact) = epic_snap.active_issues_compact {
            for issue_snap in issues_compact {
                if let Some(ref desc) = issue_snap.issue.description {
                    assert!(desc.len() <= 205, "Issue description not truncated: {}", desc.len());
                }
                if let Some(ref goal) = issue_snap.issue.goal {
                    assert!(goal.len() <= 205, "Issue goal not truncated: {}", goal.len());
                }
            }
        }
    }
}

#[tokio::test]
async fn test_note_list_derive_filtering() {
    let db = setup().await;
    let (sprint_id, epic_id, _mission_id) = seed_sprint_epic(&db).await;

    // 1. issue scope note
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "issue for note".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    let n_issue = db.note_add(CreateNoteInput {
        issue_id: issue.id,
        task_id: None,
        note_type: NoteType::Caveat,
        summary: "issue caveat".into(),
        detail: None,
        author: None,
        agent_id: Some("test".into()),
        scope: Some(NoteScope::Issue),
        scope_target_id: Some(issue.id),
        project_key: None,
    }).await.unwrap();

    // 2. epic scope note
    let n_epic = db.note_add(CreateNoteInput {
        issue_id: 0,
        task_id: None,
        note_type: NoteType::Decision,
        summary: "epic decision".into(),
        detail: None,
        author: None,
        agent_id: Some("test".into()),
        scope: Some(NoteScope::Epic),
        scope_target_id: Some(epic_id),
        project_key: None,
    }).await.unwrap();

    // 3. project scope note
    let n_proj = db.note_add(CreateNoteInput {
        issue_id: 0,
        task_id: None,
        note_type: NoteType::Discovery,
        summary: "project discovery".into(),
        detail: None,
        author: None,
        agent_id: Some("test".into()),
        scope: Some(NoteScope::Project),
        scope_target_id: None,
        project_key: Some("test-project".into()),
    }).await.unwrap();

    // project_key 필터 조회
    let notes_p = db.note_list(None, None, None, false, true, Some("test-project"), None, None, None, None).await.unwrap().items;
    assert_eq!(notes_p.len(), 3, "test-project에 속한 노트가 3개여야 함");

    // sprint_id 필터 조회
    let notes_s = db.note_list(None, None, None, false, true, None, Some(sprint_id), None, None, None).await.unwrap().items;
    assert_eq!(notes_s.len(), 2, "sprint에 속한 노트가 2개여야 함");
    assert!(notes_s.iter().any(|n| n.id == n_issue.id));
    assert!(notes_s.iter().any(|n| n.id == n_epic.id));

    // project_key + sprint_id 필터 조합 교집합 조회
    let notes_both = db.note_list(None, None, None, false, true, Some("test-project"), Some(sprint_id), None, None, None).await.unwrap().items;
    assert_eq!(notes_both.len(), 2, "교집합 조회 결과 2개여야 함");

    // 다른 project_key 필터 조회 -> 빈 결과여야 함
    let notes_other = db.note_list(None, None, None, false, true, Some("other-project"), None, None, None, None).await.unwrap().items;
    assert_eq!(notes_other.len(), 0);
}

#[tokio::test]
async fn test_mission_list_derive_filtering() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 미션 1 (seed로 생성된 mission_id) 은 sprint-project-epic 과 연관되어 있음
    // 미션 2 생성 (epic 및 sprint가 없음)
    let m2 = db.mission_create(CreateMissionInput {
        title: "unrelated mission".into(),
        description: None,
        jira_key: None,
    }).await.unwrap();

    // 1. project_key 필터 조회
    let missions_p = db.mission_list(MissionFilter {
        project_key: Some("test-project".into()),
        ..Default::default()
    }).await.unwrap();
    assert_eq!(missions_p.len(), 1, "test-project에 속한 미션이 1개여야 함");
    assert_eq!(missions_p[0].id, mission_id);

    // 2. sprint_id 필터 조회
    let missions_s = db.mission_list(MissionFilter {
        sprint_id: Some(sprint_id),
        ..Default::default()
    }).await.unwrap();
    assert_eq!(missions_s.len(), 1, "sprint_id에 속한 미션이 1개여야 함");
    assert_eq!(missions_s[0].id, mission_id);

    // 3. 교집합 필터 조회
    let missions_both = db.mission_list(MissionFilter {
        project_key: Some("test-project".into()),
        sprint_id: Some(sprint_id),
        ..Default::default()
    }).await.unwrap();
    assert_eq!(missions_both.len(), 1);

    // 4. 하위 호환 조회 (필터 미지정 시 둘 다 active 이므로 2개여야 함)
    let missions_all = db.mission_list(MissionFilter::default()).await.unwrap();
    assert_eq!(missions_all.len(), 2, "필터 없을 시 active 미션 2개 전체 반환");
}

#[tokio::test]
async fn test_list_pagination_and_compact() {
    let db = setup().await;
    let (sprint_id, epic_id, _mission_id) = seed_sprint_epic(&db).await;

    // 대용량 Mock 데이터 생성 (이슈 5개 생성)
    for i in 1..=5 {
        let title = format!("Issue {}", i);
        let desc = "A".repeat(300); // 300자 description
        let goal = "B".repeat(300); // 300자 goal
        let issue = db.issue_create(CreateIssueInput {
            epic_id,
            title,
            description: Some(desc),
            goal: Some(goal),
            priority: None,
        }).await.unwrap();

        // issue 1에 note 2개, task 3개 부착
        if i == 1 {
            db.task_create(CreateTaskInput {
                issue_id: issue.id,
                title: "Task 1".into(),
                description: None,
                goal: None,
                after_task_id: None,
                source: None,
            }).await.unwrap();
            db.task_create(CreateTaskInput {
                issue_id: issue.id,
                title: "Task 2".into(),
                description: None,
                goal: None,
                after_task_id: None,
                source: None,
            }).await.unwrap();
            db.task_create(CreateTaskInput {
                issue_id: issue.id,
                title: "Task 3".into(),
                description: None,
                goal: None,
                after_task_id: None,
                source: None,
            }).await.unwrap();

            db.note_add(CreateNoteInput {
                issue_id: issue.id,
                task_id: None,
                note_type: NoteType::Caveat,
                summary: "Caveat 1".into(),
                detail: Some("Detail".into()),
                author: None,
                agent_id: None,
                scope: None,
                scope_target_id: None,
                project_key: None,
            }).await.unwrap();
            db.note_add(CreateNoteInput {
                issue_id: issue.id,
                task_id: None,
                note_type: NoteType::Decision,
                summary: "Decision 1".into(),
                detail: Some("Detail".into()),
                author: None,
                agent_id: None,
                scope: None,
                scope_target_id: None,
                project_key: None,
            }).await.unwrap();
        }
    }

    // 1. issue_list 페이지네이션 검증 (limit=2, offset=1)
    let res = db.issue_list(IssueFilter {
        project_key: Some("test-project".into()),
        limit: Some(2),
        offset: Some(1),
        ..Default::default()
    }).await.unwrap();

    assert_eq!(res.items.len(), 2, "limit 2 설정으로 2개만 반환");
    assert_eq!(res.total, 5, "전체 개수는 5여야 함");
    assert!(res.has_more, "offset 1 + items 2 < total 5 이므로 has_more=true");

    // 2. issue_list compact 모드 검증
    let res_compact = db.issue_list(IssueFilter {
        project_key: Some("test-project".into()),
        compact: Some(true),
        limit: Some(10),
        ..Default::default()
    }).await.unwrap();

    // 첫 번째 생성한 이슈(Issue 1) 찾기
    let issue_1 = res_compact.items.iter().find(|i| i.title == "Issue 1").unwrap();
    assert_eq!(issue_1.note_count, Some(2), "note_count=2 검증");
    assert_eq!(issue_1.task_count, Some(3), "task_count=3 검증");
    
    // 장문 절단 검증 (SUBSTR 200자)
    assert_eq!(issue_1.description.as_ref().unwrap().len(), 200, "description 200자 절단 검증");
    assert_eq!(issue_1.goal.as_ref().unwrap().len(), 200, "goal 200자 절단 검증");

    // 3. note_list 페이지네이션 및 compact 검증
    // issue 1에 note 2건 생성됨
    let notes_res = db.note_list(
        Some(issue_1.id),
        None,
        None,
        false,
        false, // include_detail = false
        None,
        None,
        Some(1),
        Some(0),
        Some(true), // compact = true
    ).await.unwrap();

    assert_eq!(notes_res.items.len(), 1, "limit 1로 1개 반환");
    assert_eq!(notes_res.total, 2);
    assert!(notes_res.has_more);

    // 4. projection (필드 선택) 검증
    let paginated_val = serde_json::to_value(&res_compact).unwrap();
    let projected = engram_core::apply_projection(paginated_val, &vec!["id".into(), "title".into(), "status".into()]);
    
    let items_arr = projected.get("items").unwrap().as_array().unwrap();
    for item in items_arr {
        let obj: &serde_json::Map<String, serde_json::Value> = item.as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("title"));
        assert!(obj.contains_key("status"));
        assert!(!obj.contains_key("description"));
        assert!(!obj.contains_key("goal"));
        assert!(!obj.contains_key("note_count"));
    }
}

/// ============================================================
/// 이슈 #343: list 계열 필터·페이지네이션 토큰 한도 회귀 테스트
/// ============================================================


/// 25K 자 토큰 임계값 (session_restore 회귀 테스트와 동일 기준)
const TOKEN_LIMIT_CHARS: usize = 25_000;

/// note_list(project_key + sprint_id) derive 필터 정확성 검증
#[tokio::test]
async fn test_note_list_derive_filter_project_and_sprint() {
    let db = setup().await;

    // sprint/epic/issue/note 세팅
    let sprint_a = db.sprint_create(CreateSprintInput {
        name: "Sprint A".to_string(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    let sprint_b = db.sprint_create(CreateSprintInput {
        name: "Sprint B".to_string(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();

    let mission = db.mission_create(CreateMissionInput {
        title: "M".to_string(), description: None, jira_key: None,
    }).await.unwrap();

    let epic_a = db.epic_create(CreateEpicInput {
        project_key: "proj-a".to_string(), mission_id: Some(mission.id),
        sprint_id: Some(sprint_a.id), title: "Epic A".to_string(), description: None,
    }).await.unwrap();
    let epic_b = db.epic_create(CreateEpicInput {
        project_key: "proj-b".to_string(), mission_id: Some(mission.id),
        sprint_id: Some(sprint_b.id), title: "Epic B".to_string(), description: None,
    }).await.unwrap();

    let issue_a = db.issue_create(CreateIssueInput {
        epic_id: epic_a.id, title: "Issue A".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();
    let issue_b = db.issue_create(CreateIssueInput {
        epic_id: epic_b.id, title: "Issue B".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();

    // 각 이슈에 노트 추가
    db.note_add(CreateNoteInput {
        issue_id: issue_a.id, task_id: None, note_type: NoteType::Decision,
        summary: "note for proj-a sprint-a".to_string(), detail: None,
        author: Some("agent".to_string()), agent_id: None, scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();
    db.note_add(CreateNoteInput {
        issue_id: issue_b.id, task_id: None, note_type: NoteType::Decision,
        summary: "note for proj-b sprint-b".to_string(), detail: None,
        author: Some("agent".to_string()), agent_id: None, scope: None, scope_target_id: None, project_key: None,
    }).await.unwrap();

    // project_key 필터: proj-a 만 반환
    let res = db.note_list(None, None, None, true, false, Some("proj-a"), None, None, None, None).await.unwrap();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].summary, "note for proj-a sprint-a");

    // sprint_id 필터: sprint_b 만 반환
    let res2 = db.note_list(None, None, None, true, false, None, Some(sprint_b.id), None, None, None).await.unwrap();
    assert_eq!(res2.items.len(), 1);
    assert_eq!(res2.items[0].summary, "note for proj-b sprint-b");

    // 교집합 필터: proj-a + sprint_b → 0건
    let res3 = db.note_list(None, None, None, true, false, Some("proj-a"), Some(sprint_b.id), None, None, None).await.unwrap();
    assert_eq!(res3.items.len(), 0);
}

/// issue_list compact 모드 토큰 한도 회귀 검증 (25K 임계)
#[tokio::test]
async fn test_issue_list_compact_token_limit_regression() {
    let db = setup().await;

    let sprint = db.sprint_create(CreateSprintInput {
        name: "Sprint".to_string(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    let mission = db.mission_create(CreateMissionInput {
        title: "M".to_string(), description: None, jira_key: None,
    }).await.unwrap();
    let epic = db.epic_create(CreateEpicInput {
        project_key: "proj".to_string(), mission_id: Some(mission.id),
        sprint_id: Some(sprint.id), title: "Epic".to_string(), description: None,
    }).await.unwrap();

    // 이슈 30개를 긴 description/goal 로 생성
    let long_text = "a".repeat(2000);
    for i in 0..30 {
        db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            title: format!("Issue {}", i),
            description: Some(long_text.clone()),
            goal: Some(long_text.clone()),
            priority: None,
        }).await.unwrap();
    }

    // compact + limit=50 로 조회 → 직렬화 크기 25K 이내
    let res = db.issue_list(IssueFilter {
        compact: Some(true),
        limit: Some(50),
        ..Default::default()
    }).await.unwrap();

    assert_eq!(res.total, 30);
    assert!(!res.has_more, "30개 이슈 limit=50 이므로 has_more=false 기대");

    // compact 응답 직렬화 크기 25K 이내 검증
    let json_str = serde_json::to_string(&res).unwrap();
    assert!(
        json_str.len() < TOKEN_LIMIT_CHARS,
        "compact 응답이 토큰 한도 {}자 초과: {}자",
        TOKEN_LIMIT_CHARS,
        json_str.len()
    );

    // compact 시 description 200자 절단 확인
    for item in &res.items {
        if let Some(ref desc) = item.description {
            assert!(desc.len() <= 200, "compact 모드 description 200자 초과");
        }
        if let Some(ref goal) = item.goal {
            assert!(goal.len() <= 200, "compact 모드 goal 200자 초과");
        }
    }
}

/// note_list compact 모드 토큰 한도 + detail 절단 검증
#[tokio::test]
async fn test_note_list_compact_token_limit_regression() {
    let db = setup().await;

    let sprint = db.sprint_create(CreateSprintInput {
        name: "Sprint".to_string(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    let mission = db.mission_create(CreateMissionInput {
        title: "M".to_string(), description: None, jira_key: None,
    }).await.unwrap();
    let epic = db.epic_create(CreateEpicInput {
        project_key: "proj".to_string(), mission_id: Some(mission.id),
        sprint_id: Some(sprint.id), title: "Epic".to_string(), description: None,
    }).await.unwrap();
    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id, title: "Issue".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();

    // 긴 detail 의 노트 30개 생성
    let long_detail = "b".repeat(3000);
    for i in 0..30 {
        db.note_add(CreateNoteInput {
            issue_id: issue.id, task_id: None, note_type: NoteType::Discovery,
            summary: format!("summary {}", i),
            detail: Some(long_detail.clone()),
            author: Some("agent".to_string()), agent_id: None, scope: None, scope_target_id: None, project_key: None,
        }).await.unwrap();
    }

    // compact=true, limit=50 로 조회
    let res = db.note_list(
        Some(issue.id), None, None, true, false, None, None,
        Some(50), Some(0), Some(true),
    ).await.unwrap();

    assert_eq!(res.total, 30);

    // compact 응답 직렬화 크기 25K 이내
    let json_str = serde_json::to_string(&res).unwrap();
    assert!(
        json_str.len() < TOKEN_LIMIT_CHARS,
        "note_list compact 응답 토큰 한도 {}자 초과: {}자",
        TOKEN_LIMIT_CHARS,
        json_str.len()
    );

    // compact 시 detail 200자 절단 확인
    for note in &res.items {
        if let Some(ref detail) = note.detail {
            assert!(detail.len() <= 200, "compact 모드 note.detail 200자 초과");
        }
    }
}

/// issue_list 페이지네이션 메타(total/has_more) 정확성 검증
#[tokio::test]
async fn test_issue_list_pagination_meta_accuracy() {
    let db = setup().await;

    let sprint = db.sprint_create(CreateSprintInput {
        name: "Sprint".to_string(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    let mission = db.mission_create(CreateMissionInput {
        title: "M".to_string(), description: None, jira_key: None,
    }).await.unwrap();
    let epic = db.epic_create(CreateEpicInput {
        project_key: "proj".to_string(), mission_id: Some(mission.id),
        sprint_id: Some(sprint.id), title: "Epic".to_string(), description: None,
    }).await.unwrap();

    for i in 0..7 {
        db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            title: format!("Issue {}", i),
            description: None, goal: None, priority: None,
        }).await.unwrap();
    }

    // 첫 페이지: limit=3
    let p1 = db.issue_list(IssueFilter { limit: Some(3), offset: Some(0), ..Default::default() }).await.unwrap();
    assert_eq!(p1.items.len(), 3);
    assert_eq!(p1.total, 7);
    assert!(p1.has_more);

    // 두 번째 페이지: limit=3, offset=3
    let p2 = db.issue_list(IssueFilter { limit: Some(3), offset: Some(3), ..Default::default() }).await.unwrap();
    assert_eq!(p2.items.len(), 3);
    assert_eq!(p2.total, 7);
    assert!(p2.has_more);

    // 마지막 페이지: limit=3, offset=6 → 1개만
    let p3 = db.issue_list(IssueFilter { limit: Some(3), offset: Some(6), ..Default::default() }).await.unwrap();
    assert_eq!(p3.items.len(), 1);
    assert_eq!(p3.total, 7);
    assert!(!p3.has_more);
}

/// history_recent 기본 limit=20 동작 검증
#[tokio::test]
async fn test_history_recent_default_limit_20() {
    let db = setup().await;

    // 미션/에픽/이슈를 만들고 이슈 상태를 반복 변경해 30건 이상의 history 생성
    let mission = db.mission_create(CreateMissionInput {
        title: "M".to_string(), description: None, jira_key: None,
    }).await.unwrap();
    let epic = db.epic_create(CreateEpicInput {
        project_key: "proj".to_string(), mission_id: Some(mission.id),
        sprint_id: None, title: "Epic".to_string(), description: None,
    }).await.unwrap();
    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id, title: "Issue".to_string(),
        description: None, goal: None, priority: None,
    }).await.unwrap();

    // ready → working → demo → ready → working ... 순환으로 30회 history 생성
    let transitions = [IssueStatus::Ready, IssueStatus::Working];
    for t in transitions.iter().cycle().take(30) {
        let _ = db.issue_update(issue.id, UpdateIssueInput {
            status: Some(t.clone()),
            ..Default::default()
        }, "agent").await;
    }

    // limit=20 명시 호출 → 최대 20건
    let recent_20 = db.history_recent(20, None).await.unwrap();
    assert!(recent_20.len() <= 20, "limit=20 명시 시 최대 20건");

    // limit=5 명시 호출 → 최대 5건
    let recent_5 = db.history_recent(5, None).await.unwrap();
    assert!(recent_5.len() <= 5, "limit=5 명시 시 최대 5건");

    // limit=30 호출 → 실제 생성된 건수(30+α)까지 반환
    let recent_30 = db.history_recent(100, None).await.unwrap();
    assert!(recent_30.len() >= 30, "history 최소 30건 이상 생성됐어야 함");
}

/// ============================================================
/// 이슈 #344: demo gate 불변 보호 — user 외 주체의 finished/cancelled 거부
/// ============================================================

/// issue_update 를 통한 직접 finished 전이도 비 user agent_id 에서 거부됨을 검증
#[tokio::test]
async fn test_demo_gate_invariant_via_issue_update_various_agents() {
    let db = setup().await;
    let (_, epic_id, _) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "demo gate via issue_update".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    // ready → working → demo 까지 진행
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready), ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Working), ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo), ..Default::default()
    }, "agent").await.unwrap();

    // 다양한 비 user agent_id 로 finished 전이 시도 → 전부 거부
    for bad_agent in ["agent", "claude-opus@sess-abc", "main@0f22c068-issue1", "system"] {
        let result = db.issue_update(issue.id, UpdateIssueInput {
            status: Some(IssueStatus::Finished),
            ..Default::default()
        }, bad_agent).await;
        assert!(
            result.is_err(),
            "agent_id='{}' 로 finished 전이 시도가 허용되면 안 됨",
            bad_agent
        );
    }

    // user 만 허용
    let ok = db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Finished),
        ..Default::default()
    }, "user").await;
    assert!(ok.is_ok(), "user 는 finished 전이 가능해야 함");
    assert_eq!(ok.unwrap().status, IssueStatus::Finished);
}

/// ============================================================
/// 이슈 #346: 상태 전이 매트릭스 table-driven 테스트
/// ============================================================

/// 모든 (from, to, changed_by) 조합에 대해 허용/금지 기대값을 table-driven 으로 단언
#[tokio::test]
async fn test_status_transition_matrix_table_driven() {
    use IssueStatus::*;

    // (from_status, to_status, changed_by, should_succeed)
    // * finished/cancelled 전이는 "user" 만 허용
    // * 기타 전이는 모두 허용 (can_transition_to = true)
    let cases: &[(IssueStatus, IssueStatus, &str, bool)] = &[
        // 정상 흐름
        (Required, Ready,     "agent", true),
        (Ready,    Working,   "agent", true),
        (Working,  Demo,      "agent", true),
        // 역방향 (칸반 UX에서 허용)
        (Working,  Ready,     "agent", true),
        (Demo,     Working,   "agent", true),
        // agent 가 finished/cancelled 직접 시도 → 거부
        (Demo,     Finished,  "agent", false),
        (Demo,     Cancelled, "agent", false),
        (Working,  Finished,  "agent", false),
        (Ready,    Cancelled, "agent", false),
        (Required, Finished,  "agent", false),
        // user 는 finished/cancelled 허용
        (Demo,     Finished,  "user",  true),
        (Working,  Cancelled, "user",  true),
        (Ready,    Cancelled, "user",  true),
    ];

    let db = setup().await;
    let (_, epic_id, _) = seed_sprint_epic(&db).await;

    for (from, to, changed_by, should_succeed) in cases {
        // 매 케이스마다 fresh 이슈 생성
        let issue = db.issue_create(CreateIssueInput {
            epic_id,
            title: format!("{:?}→{:?} by {}", from, to, changed_by),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();

        // 이슈를 from 상태로 이동 (agent 로 진행, finished/cancelled 를 목적지로 사용하지 않음)
        let intermediate_states: Vec<IssueStatus> = match from {
            Required => vec![],
            Ready    => vec![Required, Ready],
            Working  => vec![Required, Ready, Working],
            Demo     => vec![Required, Ready, Working, Demo],
            Finished | Cancelled => vec![Required, Ready, Working, Demo],
        };
        for st in &intermediate_states {
            let _ = db.issue_update(issue.id, UpdateIssueInput {
                status: Some(st.clone()), ..Default::default()
            }, "agent").await;
        }

        // 목표 전이 시도
        let result = db.issue_update(issue.id, UpdateIssueInput {
            status: Some(to.clone()), ..Default::default()
        }, changed_by).await;

        if *should_succeed {
            assert!(
                result.is_ok(),
                "({:?}→{:?} by {}) 허용되어야 하는데 실패: {:?}",
                from, to, changed_by, result.err()
            );
        } else {
            assert!(
                result.is_err(),
                "({:?}→{:?} by {}) 거부되어야 하는데 성공",
                from, to, changed_by
            );
        }
    }
}

/// ============================================================
/// 이슈 #340: create 계열 create-then-read race 차단
/// ============================================================

/// mission_create 직후 즉시 조회가 성공함을 검증 (RETURNING 절 사용)
#[tokio::test]
async fn test_mission_create_then_read_immediate_success() {
    let db = setup().await;
    let m = db.mission_create(CreateMissionInput {
        title: "Race Test Mission".to_string(),
        description: Some("테스트".to_string()),
        jira_key: Some("RACE-1".to_string()),
    }).await.unwrap();

    // 생성 직후 즉시 조회 — WAL 가시성 문제 없어야 함
    let fetched = db.mission_get(m.id).await.unwrap();
    assert_eq!(fetched.id, m.id);
    assert_eq!(fetched.title, "Race Test Mission");
    assert_eq!(fetched.jira_key.as_deref(), Some("RACE-1"));
}

/// epic_create 직후 즉시 조회가 성공함을 검증
#[tokio::test]
async fn test_epic_create_then_read_immediate_success() {
    let db = setup().await;
    let m = db.mission_create(CreateMissionInput {
        title: "M".to_string(), description: None, jira_key: None,
    }).await.unwrap();

    let epic = db.epic_create(CreateEpicInput {
        project_key: "proj".to_string(),
        mission_id: Some(m.id),
        sprint_id: None,
        title: "Race Test Epic".to_string(),
        description: None,
    }).await.unwrap();

    let fetched = db.epic_get(epic.id).await.unwrap();
    assert_eq!(fetched.id, epic.id);
    assert_eq!(fetched.title, "Race Test Epic");
    assert_eq!(fetched.project_key, "proj");
}

/// issue_create 직후 즉시 조회가 성공하고 mission_id/sprint_id 가 derive 됨
#[tokio::test]
async fn test_issue_create_then_read_immediate_success() {
    let db = setup().await;
    let sprint = db.sprint_create(CreateSprintInput {
        name: "S".to_string(), goal: None, start_date: None, end_date: None,
    }).await.unwrap();
    let m = db.mission_create(CreateMissionInput {
        title: "M".to_string(), description: None, jira_key: None,
    }).await.unwrap();
    let epic = db.epic_create(CreateEpicInput {
        project_key: "proj".to_string(),
        mission_id: Some(m.id),
        sprint_id: Some(sprint.id),
        title: "Epic".to_string(),
        description: None,
    }).await.unwrap();

    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id,
        title: "Race Test Issue".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    // 생성 직후 즉시 조회 — RETURNING 으로 파생 필드까지 포함
    let fetched = db.issue_get(issue.id, false).await.unwrap();
    assert_eq!(fetched.id, issue.id);
    assert_eq!(fetched.mission_id, Some(m.id), "mission_id 가 derive 되어야 함");
    assert_eq!(fetched.sprint_id, Some(sprint.id), "sprint_id 가 derive 되어야 함");
}

/// task_create 직후 즉시 조회가 성공함을 검증
#[tokio::test]
async fn test_task_create_then_read_immediate_success() {
    let db = setup().await;
    let m = db.mission_create(CreateMissionInput {
        title: "M".to_string(), description: None, jira_key: None,
    }).await.unwrap();
    let epic = db.epic_create(CreateEpicInput {
        project_key: "proj".to_string(),
        mission_id: Some(m.id),
        sprint_id: None,
        title: "Epic".to_string(),
        description: None,
    }).await.unwrap();
    let issue = db.issue_create(CreateIssueInput {
        epic_id: epic.id, title: "Issue".to_string(),
        description: None, goal: None, priority: None,
    }).await.unwrap();

    let task = db.task_create(CreateTaskInput {
        issue_id: issue.id,
        title: "Race Test Task".to_string(),
        description: None,
        goal: None,
        after_task_id: None,
        source: None,
    }).await.unwrap();

    let fetched = db.task_get(task.id).await.unwrap();
    assert_eq!(fetched.id, task.id);
    assert_eq!(fetched.title, "Race Test Task");
    assert_eq!(fetched.issue_id, issue.id);
}

/// 동일 jira_key 로 연속 mission_create 시 중복 생성이 아닌 에러 반환 (중복 0건)
#[tokio::test]
async fn test_create_duplicate_jira_key_rejected_not_duplicated() {
    let db = setup().await;

    db.mission_create(CreateMissionInput {
        title: "M1".to_string(),
        description: None,
        jira_key: Some("PROJ-UNIQUE".to_string()),
    }).await.unwrap();

    // 동일 jira_key 로 재시도 → 에러
    let result = db.mission_create(CreateMissionInput {
        title: "M2".to_string(),
        description: None,
        jira_key: Some("PROJ-UNIQUE".to_string()),
    }).await;
    assert!(result.is_err(), "중복 jira_key 는 에러여야 함");

    // DB 에 실제 1건만 존재하는지 확인
    let list = db.mission_list(MissionFilter { include_completed: true, ..Default::default() }).await.unwrap();
    assert_eq!(list.len(), 1, "중복 생성 없이 1건만 존재해야 함");
}

/// ============================================================
/// 이슈 #345: note_add DX 개선 — 구조화 에러 메시지 + 회귀 테스트
/// ============================================================

/// note_add: issue scope 에서 issue_id 누락 시 구조화 에러 반환
#[tokio::test]
async fn test_note_add_issue_scope_missing_issue_id_returns_structured_error() {
    let db = setup().await;

    let result = db.note_add(CreateNoteInput {
        issue_id: 0, // 누락 (<=0)
        task_id: None,
        note_type: NoteType::Decision,
        summary: "test".into(),
        detail: None,
        author: None,
        agent_id: None,
        scope: Some(NoteScope::Issue),
        scope_target_id: None,
        project_key: None,
    }).await;

    assert!(result.is_err(), "issue_id 없이 issue scope 는 실패해야 함");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("expected_fields"), "에러에 expected_fields 포함 필요: {err}");
    assert!(err.contains("issue_id"), "에러에 issue_id 포함 필요: {err}");
    assert!(err.contains("\"scope\":\"issue\""), "에러에 scope 값 포함 필요: {err}");
}

/// note_add: task scope 에서 task_id 누락 시 구조화 에러 반환
#[tokio::test]
async fn test_note_add_task_scope_missing_task_id_returns_structured_error() {
    let db = setup().await;
    let (_, epic_id, _) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id, title: "I".into(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let result = db.note_add(CreateNoteInput {
        issue_id: issue.id,
        task_id: None, // 누락
        note_type: NoteType::Caveat,
        summary: "test".into(),
        detail: None,
        author: None,
        agent_id: None,
        scope: Some(NoteScope::Task),
        scope_target_id: None,
        project_key: None,
    }).await;

    assert!(result.is_err(), "task_id 없이 task scope 는 실패해야 함");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("expected_fields"), "에러에 expected_fields 포함 필요: {err}");
    assert!(err.contains("task_id"), "에러에 task_id 포함 필요: {err}");
    assert!(err.contains("\"scope\":\"task\""), "에러에 scope 값 포함 필요: {err}");
}

/// note_add: project scope 에서 project_key 누락 시 구조화 에러 반환
#[tokio::test]
async fn test_note_add_project_scope_missing_project_key_returns_structured_error() {
    let db = setup().await;

    let result = db.note_add(CreateNoteInput {
        issue_id: 0,
        task_id: None,
        note_type: NoteType::Discovery,
        summary: "test".into(),
        detail: None,
        author: None,
        agent_id: None,
        scope: Some(NoteScope::Project),
        scope_target_id: None,
        project_key: None, // 누락
    }).await;

    assert!(result.is_err(), "project_key 없이 project scope 는 실패해야 함");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("expected_fields"), "에러에 expected_fields 포함 필요: {err}");
    assert!(err.contains("project_key"), "에러에 project_key 포함 필요: {err}");
}

/// note_add: 정상 경로 회귀 없음 — issue scope 정상 성공
#[tokio::test]
async fn test_note_add_valid_issue_scope_succeeds_regression() {
    let db = setup().await;
    let (_, epic_id, _) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id, title: "I".into(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let result = db.note_add(CreateNoteInput {
        issue_id: issue.id,
        task_id: None,
        note_type: NoteType::Decision,
        summary: "정상 노트".into(),
        detail: Some("상세".into()),
        author: Some("agent".into()),
        agent_id: Some("test-agent".into()),
        scope: Some(NoteScope::Issue),
        scope_target_id: None,
        project_key: None,
    }).await;

    assert!(result.is_ok(), "정상 issue scope 노트 추가는 성공해야 함: {:?}", result.err());
    assert_eq!(result.unwrap().issue_id, Some(issue.id));
}

/// note_add: epic scope 에서 scope_target_id 누락 시 구조화 에러 반환
#[tokio::test]
async fn test_note_add_epic_scope_missing_target_id_returns_structured_error() {
    let db = setup().await;

    let result = db.note_add(CreateNoteInput {
        issue_id: 0,
        task_id: None,
        note_type: NoteType::Reference,
        summary: "test".into(),
        detail: None,
        author: None,
        agent_id: None,
        scope: Some(NoteScope::Epic),
        scope_target_id: None, // 누락
        project_key: None,
    }).await;

    assert!(result.is_err(), "scope_target_id 없이 epic scope 는 실패해야 함");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("expected_fields"), "에러에 expected_fields 포함 필요: {err}");
    assert!(err.contains("\"scope\":\"epic\""), "에러에 scope 값 포함 필요: {err}");
}

#[tokio::test]
async fn test_compact_session_restore_size_guard_truncated_false_bug() {
    let db = setup().await;
    let (sprint_id, epic_id, mission_id) = seed_sprint_epic(&db).await;

    // 한글이 다량 포함된 대용량 Mock 데이터 생성 (이슈 30개 생성)
    // description과 goal은 None으로 하여 chars().count()가 limit(25000)을 넘지 않게 조절
    let korean_text = "한글대용량테스트문장입니다".repeat(30); // 약 360자 (chars), 1080바이트
    for i in 1..=30 {
        let title = format!("이슈 {}: {}", i, korean_text);
        let issue = db.issue_create(CreateIssueInput {
            epic_id,
            title,
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();
        
        db.issue_update(issue.id, UpdateIssueInput {
            status: Some(IssueStatus::Ready),
            ..Default::default()
        }, "agent").await.unwrap();
    }

    // compact=true, size_limit=None (기본 25000자)로 호출
    let snap = db.session_restore(Some("test-project"), true, 120, None).await.unwrap();

    let serialized = serde_json::to_string(&snap).unwrap();
    let byte_len = serialized.len();
    let char_count = serialized.chars().count();
    
    println!("DEBUG: Byte Length = {}, Char Count = {}, Truncated = {}, Count = {:?}", 
             byte_len, char_count, snap.truncated, snap.truncated_count);

    // 버그 수정 후 검증 단언
    assert!(snap.truncated, "용량 한도 초과로 truncated 플래그가 true여야 합니다.");
    assert_eq!(snap.truncated_count, Some(31), "누락 항목 개수가 31(에픽 1 + 이슈 30)이어야 합니다. 실제: {:?}", snap.truncated_count);
}

#[tokio::test]
async fn test_compact_session_restore_omits_active_caveats_detail() {
    let db = setup().await;
    let (_sprint_id, epic_id, _mission_id) = seed_sprint_epic(&db).await;

    // active_epics에 포함되기 위해 이슈 생성 및 ready로 승격
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "테스트 이슈".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // unresolved caveat 추가 (scope=epic)
    let caveat = db.note_add(CreateNoteInput {
        issue_id: 0,
        task_id: None,
        note_type: NoteType::Caveat,
        summary: "주의사항 요약".into(),
        detail: Some("매우 상세하고 긴 주의사항 본문입니다.".into()),
        author: Some("agent".into()),
        agent_id: Some("test-agent".into()),
        scope: Some(NoteScope::Epic),
        scope_target_id: Some(epic_id),
        project_key: None,
    }).await.unwrap();

    // 1. compact=false 호출 시 detail이 존재해야 함
    let snap_full = db.session_restore(Some("test-project"), false, 120, None).await.unwrap();
    let found_caveat_full = snap_full.active_caveats.iter().find(|c| c.id == caveat.id).expect("caveat 존재해야 함");
    assert_eq!(found_caveat_full.detail.as_deref(), Some("매우 상세하고 긴 주의사항 본문입니다."));

    // 2. compact=true 호출 시 detail이 None이어야 함
    let snap_compact = db.session_restore(Some("test-project"), true, 120, None).await.unwrap();
    let found_caveat_compact = snap_compact.active_caveats.iter().find(|c| c.id == caveat.id).expect("caveat 존재해야 함");
    assert!(found_caveat_compact.detail.is_none(), "compact 모드에서는 caveat detail이 None이어야 함");

    // 3. 직렬화 시 detail 필드가 JSON에서 완전히 생략되었는지 확인
    let serialized = serde_json::to_string(&snap_compact).unwrap();
    assert!(!serialized.contains("매우 상세하고 긴 주의사항 본문입니다"), "직렬화 데이터에 detail 본문이 없어야 함");
    assert!(!serialized.contains("\"detail\""), "skip_serializing_if에 의해 detail 키 자체가 없어야 함");
}

#[tokio::test]
async fn test_session_restore_excludes_finished_issues() {
    let db = setup().await;
    let (_sprint_id, epic_id, _mission_id) = seed_sprint_epic(&db).await;

    // 1. Ready 상태 이슈 생성
    let issue_ready = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Ready Issue".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_ready.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // 2. Finished 상태로 갈 이슈 생성 및 Finished로 전이
    let issue_fin = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Finished Issue".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_fin.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue_fin.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_update(issue_fin.id, UpdateIssueInput {
        status: Some(IssueStatus::Demo),
        ..Default::default()
    }, "agent").await.unwrap();
    db.issue_finish(issue_fin.id, "user").await.unwrap();

    // 3. session_restore 호출
    let snap = db.session_restore(Some("test-project"), false, 120, None).await.unwrap();

    // 4. 에픽 진행률 검증 (done=1, total=2)
    assert_eq!(snap.active_epics.len(), 1, "에픽 개수 검증");
    let epic_snap = &snap.active_epics[0];
    assert_eq!(epic_snap.progress.done, 1, "done 카운트 검증");
    assert_eq!(epic_snap.progress.total, 2, "total 카운트 검증");

    // 5. active_issues 목록 검증 (Ready 이슈만 존재하고 Finished 이슈는 제외되어야 함)
    let active_ids: Vec<i64> = epic_snap.active_issues.iter().map(|s| s.issue.id).collect();
    assert!(active_ids.contains(&issue_ready.id), "Ready 이슈는 포함되어야 함");
    assert!(!active_ids.contains(&issue_fin.id), "Finished 이슈는 제외되어야 함");
}

#[tokio::test]
async fn test_session_restore_output_mode_agent() {
    let db = setup().await;
    let (_sprint_id, epic_id, _mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Test Issue for Agent Mode".into(),
        description: Some("상세 설명 한글".into()),
        goal: Some("목표 한글".into()),
        priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    // 1. OutputMode::Agent 로 호출하면 String 형태로 영문 마크다운 포맷이 반환되어야 함.
    let response = db.session_restore_mode(Some("test-project"), engram_core::models::OutputMode::Agent, 120, None).await.unwrap();
    
    // agent 텍스트 결과 검증
    match response {
        engram_core::repository::session::SessionResponse::Text(text) => {
            assert!(text.contains("=== ENGRAM SESSION CONTEXT ==="));
            assert!(text.contains("NEXT ACTION"));
            assert!(text.contains("Test Issue for Agent Mode"));
            assert!(!text.contains("상세 설명 한글"), "에이전트 텍스트 모드에서는 본문이 출력되지 않거나 콤팩트해야 함");
        },
        _ => panic!("Expected text response for Agent mode"),
    }
}

#[tokio::test]
async fn test_issue_and_epic_get_list_output_mode_agent() {
    let db = setup().await;
    let (_sprint_id, epic_id, _mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Test Issue For Spec".into(),
        description: Some("상세 본문".into()),
        goal: Some("성공 목표".into()),
        priority: None,
    }).await.unwrap();

    // 1. issue_get_mode 에 OutputMode::Agent를 전달하면 Text(String) 타입으로 반환되는가?
    let issue_resp = db.issue_get_mode(issue.id, engram_core::models::OutputMode::Agent).await.unwrap();
    match issue_resp {
        engram_core::models::CoreResponse::Text(text) => {
            assert!(text.contains("=== ISSUE SPECIFICATION ==="));
            assert!(text.contains("Test Issue For Spec"));
            assert!(text.contains("상세 본문"));
        },
        _ => panic!("Expected text response for issue_get agent mode"),
    }

    // 2. epic_get_mode 에 OutputMode::Agent를 전달하면 Text(String) 타입으로 반환되는가?
    let epic_resp = db.epic_get_mode(epic_id, engram_core::models::OutputMode::Agent).await.unwrap();
    match epic_resp {
        engram_core::models::CoreResponse::Text(text) => {
            assert!(text.contains("=== EPIC SPECIFICATION ==="));
            assert!(text.contains("Test Epic"));
        },
        _ => panic!("Expected text response for epic_get agent mode"),
    }
}

#[tokio::test]
async fn test_other_get_list_output_mode_agent() {
    let db = setup().await;
    let (_sprint_id, epic_id, _mission_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Test Issue for Tasks".into(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();

    let task = db.task_create(CreateTaskInput {
        issue_id: issue.id,
        title: "Test Task".into(),
        description: None,
        goal: None,
        after_task_id: None,
        source: None,
    }).await.unwrap();

    // 1. task_list_mode 에 OutputMode::Agent를 전달하면 Text(String) 타입으로 반환되는가?
    let task_list_resp = db.task_list_mode(issue.id, None, engram_core::models::OutputMode::Agent).await.unwrap();
    match task_list_resp {
        engram_core::models::CoreResponse::Text(text) => {
            assert!(text.contains("=== TASK LIST ==="));
            assert!(text.contains("Test Task"));
        },
        _ => panic!("Expected text response for task_list agent mode"),
    }

    // 2. task_next_mode 에 OutputMode::Agent를 전달하면 Text(String) 타입으로 반환되는가?
    // task를 ready로 전환해야 task_next에 잡힘
    db.task_update(task.id, UpdateTaskInput {
        status: Some(TaskStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();
    // issue 도 ready 여야 함
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }, "agent").await.unwrap();

    let task_next_resp = db.task_next_mode(Some("test-project"), None, engram_core::models::OutputMode::Agent).await.unwrap();
    match task_next_resp {
        engram_core::models::CoreResponse::Text(text) => {
            assert!(text.contains("=== NEXT TASK ==="));
            assert!(text.contains("Test Task"));
        },
        _ => panic!("Expected text response for task_next agent mode"),
    }

    // 3. note_list_mode 에 OutputMode::Agent를 전달하면 Text(String) 타입으로 반환되는가?
    let _note = db.note_add(CreateNoteInput {
        issue_id: issue.id,
        task_id: None,
        note_type: NoteType::Caveat,
        summary: "Test Caveat Note".into(),
        detail: None,
        author: None,
        agent_id: None,
        scope: None,
        scope_target_id: None,
        project_key: None,
    }).await.unwrap();

    let note_list_resp = db.note_list_mode(
        Some(issue.id),
        None,
        None,
        false,
        true,
        None,
        None,
        None,
        None,
        engram_core::models::OutputMode::Agent,
    ).await.unwrap();
    match note_list_resp {
        engram_core::models::CoreResponse::Text(text) => {
            assert!(text.contains("=== NOTE LIST ==="));
            assert!(text.contains("Test Caveat Note"));
        },
        _ => panic!("Expected text response for note_list agent mode"),
    }

    // 4. board_status_mode 에 OutputMode::Agent를 전달하면 Text(String) 타입으로 반환되는가?
    let board_resp = db.board_status_mode(Some("test-project"), engram_core::models::OutputMode::Agent, true).await.unwrap();
    match board_resp {
        engram_core::models::CoreResponse::Text(text) => {
            assert!(text.contains("=== BOARD STATUS ==="));
            assert!(text.contains("Test Issue for Tasks"));
        },
        _ => panic!("Expected text response for board_status agent mode"),
    }

    // 5. stalled_issues_mode 에 OutputMode::Agent를 전달하면 Text(String) 타입으로 반환되는가?
    // issue 상태를 working으로
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }, "agent").await.unwrap();

    let stalled_resp = db.stalled_issues_mode(Some("test-project"), IssueStatus::Working, 0, engram_core::models::OutputMode::Agent).await.unwrap();
    match stalled_resp {
        engram_core::models::CoreResponse::Text(text) => {
            assert!(text.contains("=== STALLED ISSUES ==="));
            assert!(text.contains("Test Issue for Tasks"));
        },
        _ => panic!("Expected text response for stalled_issues agent mode"),
    }
}

#[tokio::test]
async fn test_task_list_status_filtering() {
    let db = setup().await;
    let (_, epic_id, _) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id, title: "I".into(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let t1 = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T1".into(), description: None, goal: None, after_task_id: None, source: None,
    }).await.unwrap();
    let t2 = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T2".into(), description: None, goal: None, after_task_id: None, source: None,
    }).await.unwrap();

    // t1은 finished로 변경
    db.task_update(t1.id, UpdateTaskInput {
        status: Some(TaskStatus::Finished), ..Default::default()
    }, "agent").await.unwrap();

    // 1. status=None 이면 2개 태스크 모두 반환
    let all_tasks = db.task_list(issue.id, None).await.unwrap();
    assert_eq!(all_tasks.len(), 2);

    // 2. status=Some(TaskStatus::Required) 이면 T2(Required)만 반환
    let required_tasks = db.task_list(issue.id, Some(TaskStatus::Required)).await.unwrap();
    assert_eq!(required_tasks.len(), 1);
    assert_eq!(required_tasks[0].id, t2.id);

    // 3. status=Some(TaskStatus::Finished) 이면 T1(Finished)만 반환
    let finished_tasks = db.task_list(issue.id, Some(TaskStatus::Finished)).await.unwrap();
    assert_eq!(finished_tasks.len(), 1);
    assert_eq!(finished_tasks[0].id, t1.id);
}

#[tokio::test]
async fn test_task_test_list_by_issue_id() {
    let db = setup().await;
    let (_, epic_id, _) = seed_sprint_epic(&db).await;
    let issue = db.issue_create(CreateIssueInput {
        epic_id, title: "Issue with tests".into(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let t1 = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T1".into(), description: None, goal: None, after_task_id: None, source: None,
    }).await.unwrap();
    let t2 = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T2".into(), description: None, goal: None, after_task_id: None, source: None,
    }).await.unwrap();

    // t1에 테스트 2개, t2에 테스트 1개 추가
    db.task_test_add(t1.id, "Test 1 for T1".into()).await.unwrap();
    db.task_test_add(t1.id, "Test 2 for T1".into()).await.unwrap();
    db.task_test_add(t2.id, "Test 1 for T2".into()).await.unwrap();

    // issue_id 로 조회
    let tests = db.task_test_list(None, Some(issue.id)).await.unwrap();
    assert_eq!(tests.len(), 3);
    assert_eq!(tests[0].label, "Test 1 for T1");
    assert_eq!(tests[1].label, "Test 2 for T1");
    assert_eq!(tests[2].label, "Test 1 for T2");

    // task_id 와 issue_id 가 둘 다 지정되지 않은 경우 에러 반환
    let err = db.task_test_list(None, None).await;
    assert!(err.is_err());
}





