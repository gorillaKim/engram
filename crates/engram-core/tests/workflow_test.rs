use engram_core::{
    Db,
    models::{
        sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus},
        epic::CreateEpicInput,
        issue::CreateIssueInput,
        task::CreateTaskInput,
        note::{CreateNoteInput, NoteType},
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

    // sprint을 active로 변경 (sprint_update 사용)
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

    // 이슈 생성 (draft)
    let issue = db.issue_create(CreateIssueInput {
        epic_id,
        title: "Test Issue".to_string(),
        description: None,
        priority: None,
    }).await.unwrap();

    assert_eq!(issue.status, engram_core::models::IssueStatus::Draft);

    // 이슈 승인
    let approved = db.issue_update(issue.id, engram_core::models::issue::UpdateIssueInput {
        status: Some(engram_core::models::IssueStatus::Approved),
        ..Default::default()
    }).await.unwrap();
    assert_eq!(approved.status, engram_core::models::IssueStatus::Approved);

    // 태스크 생성
    let t1 = db.task_create(CreateTaskInput {
        issue_id: issue.id,
        title: "Task 1".to_string(),
        description: None,
        after_task_id: None,
        source: None,
    }).await.unwrap();

    // caveat note 추가
    let note = db.note_add(CreateNoteInput {
        issue_id: issue.id,
        task_id: None,
        note_type: NoteType::Caveat,
        summary: "조심할 점".to_string(),
        detail: None,
        author: None,
    }).await.unwrap();

    // session_restore 확인
    let snapshot = db.session_restore(Some("test-project")).await.unwrap();
    assert!(!snapshot.active_epics.is_empty());

    let epic_snap = &snapshot.active_epics[0];
    assert!(!epic_snap.active_issues.is_empty());

    let issue_snap = &epic_snap.active_issues[0];
    assert_eq!(issue_snap.active_notes.len(), 1);
    assert_eq!(issue_snap.active_notes[0].note_type, NoteType::Caveat);

    // task_next 확인
    let next = db.task_next(Some("test-project"), None).await.unwrap();
    assert!(next.is_some());
    assert_eq!(next.unwrap().task_id, t1.id);

    println!("✅ 전체 워크플로우 테스트 통과");
}

#[tokio::test]
async fn test_blocked_by_reverse_query() {
    let db = setup().await;
    let (_, epic_id) = seed_sprint_epic(&db).await;

    let issue_a = db.issue_create(CreateIssueInput {
        epic_id, title: "Issue A".to_string(), description: None, priority: None,
    }).await.unwrap();

    let issue_b = db.issue_create(CreateIssueInput {
        epic_id, title: "Issue B".to_string(), description: None, priority: None,
    }).await.unwrap();

    // A blocks B
    db.issue_link(issue_a.id, issue_b.id, engram_core::models::LinkType::Blocks).await.unwrap();

    // B의 blocked_by 역방향 조회 → A가 반환돼야 함
    let blockers = db.issue_blocked_by(issue_b.id).await.unwrap();
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0].source_id, issue_a.id);

    println!("✅ blocked_by 역방향 쿼리 테스트 통과");
}

#[tokio::test]
async fn test_fractional_ord_insert() {
    let db = setup().await;
    let (_, epic_id) = seed_sprint_epic(&db).await;

    let issue = db.issue_create(CreateIssueInput {
        epic_id, title: "Issue".to_string(), description: None, priority: None,
    }).await.unwrap();

    let t1 = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T1".to_string(),
        description: None, after_task_id: None, source: None,
    }).await.unwrap(); // ord = 1.0

    let t2 = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T2".to_string(),
        description: None, after_task_id: None, source: None,
    }).await.unwrap(); // ord = 2.0

    // T1과 T2 사이에 삽입
    let t_mid = db.task_create(CreateTaskInput {
        issue_id: issue.id, title: "T_mid".to_string(),
        description: None, after_task_id: Some(t1.id), source: None,
    }).await.unwrap(); // ord = 1.5

    assert!(t_mid.ord > t1.ord && t_mid.ord < t2.ord);

    let tasks = db.task_list(issue.id, None).await.unwrap();
    assert_eq!(tasks[0].title, "T1");
    assert_eq!(tasks[1].title, "T_mid");
    assert_eq!(tasks[2].title, "T2");

    println!("✅ Fractional index 삽입 테스트 통과");
}
