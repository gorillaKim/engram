use engram_core::{
    Db,
    models::{
        sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus},
        epic::CreateEpicInput,
        issue::{CreateIssueInput, UpdateIssueInput, IssueStatus, IssuePriority},
        task::{CreateTaskInput, UpdateTaskInput, TaskStatus},
        note::{CreateNoteInput, NoteType},
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
    }).await.unwrap();

    let epic = db.epic_create(CreateEpicInput {
        sprint_id: sprint.id,
        project_key: "test-project".to_string(),
        title: "Test Epic".to_string(),
        description: None,
    }).await.unwrap();

    (sprint.id, epic.id)
}

#[tokio::test]
async fn test_full_sprint_workflow() {
    let db = setup().await;
    let (_, epic_id) = seed_sprint_epic(&db).await;

    // 이슈 생성 (required)
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Test Issue".to_string(),
        description: None,
        goal: Some("인증 흐름 완전 전환".to_string()),
        priority: None,
    }).await.unwrap();

    assert_eq!(issue.status, IssueStatus::Required);

    // 이슈 준비 완료 (required → ready)
    let ready_issue = db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }).await.unwrap();
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
    }).await.unwrap();

    // caveat note 추가
    db.note_add(CreateNoteInput {
        issue_id: issue.id,
        task_id: None,
        note_type: NoteType::Caveat,
        summary: "조심할 점".to_string(),
        detail: None,
        author: None,
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
    let (_, epic_id) = seed_sprint_epic(&db).await;

    let issue_a = db.issue_create(CreateIssueInput {
        epic_id, title: "Issue A".to_string(), description: None, goal: None, priority: None,
    }).await.unwrap();

    let issue_b = db.issue_create(CreateIssueInput {
        epic_id, title: "Issue B".to_string(), description: None, goal: None, priority: None,
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
    let (_, epic_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id, title: "Issue".to_string(), description: None, goal: None, priority: None,
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
    }).await.unwrap();

    let epic_a = db.epic_create(CreateEpicInput {
        sprint_id: sprint.id,
        project_key: "proj-a".to_string(),
        title: "Epic A".to_string(),
        description: None,
    }).await.unwrap();

    let epic_b = db.epic_create(CreateEpicInput {
        sprint_id: sprint.id,
        project_key: "proj-b".to_string(),
        title: "Epic B".to_string(),
        description: None,
    }).await.unwrap();

    // proj-a 이슈 생성 후 Ready 전환
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id: epic_a.id,
        title: "Issue A".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }).await.unwrap();

    // proj-b 이슈 생성 후 Ready 전환
    let issue_b = db.issue_create(CreateIssueInput {
        epic_id: epic_b.id,
        title: "Issue B".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_b.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }).await.unwrap();

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
    let (_, epic_id) = seed_sprint_epic(&db).await;

    // 이슈 A: Critical
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Critical Issue".to_string(),
        description: None,
        goal: None,
        priority: Some(IssuePriority::Critical),
    }).await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }).await.unwrap();

    // 이슈 B: High
    let issue_b = db.issue_create(CreateIssueInput {
        epic_id,
        title: "High Issue".to_string(),
        description: None,
        goal: None,
        priority: Some(IssuePriority::High),
    }).await.unwrap();
    db.issue_update(issue_b.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }).await.unwrap();

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
    }).await.unwrap();

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
    }).await.unwrap();

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
    }).await.unwrap();

    let epic_a = db.epic_create(CreateEpicInput {
        sprint_id: sprint.id,
        project_key: "proj-a".to_string(),
        title: "Epic A".to_string(),
        description: None,
    }).await.unwrap();
    let epic_b = db.epic_create(CreateEpicInput {
        sprint_id: sprint.id,
        project_key: "proj-b".to_string(),
        title: "Epic B".to_string(),
        description: None,
    }).await.unwrap();

    // proj-a 이슈 A (Ready)
    let issue_a = db.issue_create(CreateIssueInput {
        epic_id: epic_a.id,
        title: "Issue A (blocker)".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }).await.unwrap();

    // proj-b 이슈 B (Ready)
    let issue_b = db.issue_create(CreateIssueInput {
        epic_id: epic_b.id,
        title: "Issue B (blocked)".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue_b.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }).await.unwrap();

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
    }).await.unwrap();

    // 이슈 B는 blocked → task_next(proj-b) None 반환
    let next_before = db.task_next(Some("proj-b"), None).await.unwrap();
    assert!(next_before.is_none(), "이슈 B가 blocked 상태일 때 task_next는 None이어야 함");

    // 이슈 A를 Finished로 전환 (Required → Ready → Working → Finished 순이나 test에선 직접 DB 우회 불가 — Working 거쳐야 함)
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Working),
        ..Default::default()
    }).await.unwrap();
    db.issue_update(issue_a.id, UpdateIssueInput {
        status: Some(IssueStatus::Finished),
        ..Default::default()
    }).await.unwrap();

    // 이제 이슈 B의 blocker가 finished → task_next(proj-b) 태스크 반환
    let next_after = db.task_next(Some("proj-b"), None).await.unwrap();
    assert!(next_after.is_some(), "blocker가 finished 된 후 task_next는 태스크를 반환해야 함");
    assert_eq!(next_after.unwrap().task_id, task_b.id, "이슈 B의 태스크가 반환돼야 함");
}

#[tokio::test]
async fn test_scope_expansion_warning() {
    let db = setup().await;
    let (_, epic_id) = seed_sprint_epic(&db).await;

    // 이슈 생성 및 Ready 전환
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Scope Expansion Issue".to_string(),
        description: None,
        goal: None,
        priority: None,
    }).await.unwrap();
    db.issue_update(issue.id, UpdateIssueInput {
        status: Some(IssueStatus::Ready),
        ..Default::default()
    }).await.unwrap();

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
