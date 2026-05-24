use crate::Db;
use sqlx::Row;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockingGraph {
    pub chains: Vec<Vec<i64>>,   // blocker → blocked 경로 (issue id 배열)
    pub leaf_blockers: Vec<i64>, // 해소 가능한 최상위 blocker ids
    pub has_cycle: bool,
}

impl Db {
    pub async fn blocked_issues_graph(&self, project_key: &str) -> crate::Result<BlockingGraph> {
        let rows = sqlx::query(r#"
            SELECT il.source_id, il.target_id
            FROM issue_links il
            JOIN issues si ON il.source_id = si.id
            JOIN issues ti ON il.target_id = ti.id
            JOIN epics e ON si.epic_id = e.id
            WHERE il.link_type = 'blocks'
              AND si.status NOT IN ('finished','cancelled')
              AND ti.status NOT IN ('finished','cancelled')
              AND e.project_key = ?
        "#)
        .bind(project_key)
        .fetch_all(&self.pool)
        .await?;

        let mut adj: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
        let mut all_targets: std::collections::HashSet<i64> = std::collections::HashSet::new();

        for row in &rows {
            let src: i64 = row.get("source_id");
            let tgt: i64 = row.get("target_id");
            adj.entry(src).or_default().push(tgt);
            all_targets.insert(tgt);
        }

        if adj.is_empty() {
            return Ok(BlockingGraph { chains: vec![], leaf_blockers: vec![], has_cycle: false });
        }

        // leaf_blockers = sources that are not themselves targets (nothing blocks them)
        let mut leaf_blockers: Vec<i64> = adj.keys()
            .filter(|k| !all_targets.contains(*k))
            .copied()
            .collect();
        leaf_blockers.sort();

        // BFS to build chains from each leaf blocker
        let mut chains = Vec::new();
        for &leaf in &leaf_blockers {
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(vec![leaf]);
            while let Some(path) = queue.pop_front() {
                let last = *path.last().unwrap();
                if let Some(nexts) = adj.get(&last) {
                    for &next in nexts {
                        let mut new_path = path.clone();
                        if new_path.contains(&next) {
                            return Ok(BlockingGraph { chains, leaf_blockers, has_cycle: true });
                        }
                        new_path.push(next);
                        queue.push_back(new_path);
                    }
                } else {
                    chains.push(path);
                }
            }
        }

        Ok(BlockingGraph { chains, leaf_blockers, has_cycle: false })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        issue::{CreateIssueInput, UpdateIssueInput, IssueStatus, LinkType},
        epic::CreateEpicInput,
        sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus},
    };

    async fn setup() -> Db {
        Db::open_in_memory().await.unwrap()
    }

    async fn seed(db: &Db) -> (i64, i64, i64) { // sprint_id, epic_id, mission_id
        let sprint = db.sprint_create(CreateSprintInput { name: "S".to_string(), goal: None, start_date: None, end_date: None }).await.unwrap();
        db.sprint_update(sprint.id, UpdateSprintInput { status: Some(SprintStatus::Active), ..Default::default() }, "agent").await.unwrap();
        let mission = db.mission_create(crate::models::mission::CreateMissionInput {
            title: "M".to_string(), description: None, jira_key: None, sprint_id: Some(sprint.id)
        }).await.unwrap();
        let epic = db.epic_create(CreateEpicInput { project_key: "p".to_string(), mission_id: Some(mission.id), title: "E".to_string(), description: None }).await.unwrap();
        (sprint.id, epic.id, mission.id)
    }

    async fn make_issue(db: &Db, mission_id: i64, epic_id: i64, title: &str) -> i64 {
        db.issue_create(CreateIssueInput { epic_id, mission_id: Some(mission_id), sprint_id: None, title: title.to_string(), description: None, goal: None, priority: None }).await.unwrap().id
    }

    #[tokio::test]
    async fn test_simple_block() {
        let db = setup().await;
        let (_sprint_id, epic_id, mission_id) = seed(&db).await;
        let a = make_issue(&db, mission_id, epic_id, "A").await;
        let b = make_issue(&db, mission_id, epic_id, "B").await;
        db.issue_link(a, b, LinkType::Blocks).await.unwrap(); // A blocks B

        let graph = db.blocked_issues_graph("p").await.unwrap();
        assert!(!graph.chains.is_empty(), "체인이 있어야 함");
        assert!(graph.leaf_blockers.contains(&a), "A가 리프 blocker여야 함");
        assert!(!graph.has_cycle);
    }

    #[tokio::test]
    async fn test_chain_block() {
        let db = setup().await;
        let (_sprint_id, epic_id, mission_id) = seed(&db).await;
        let a = make_issue(&db, mission_id, epic_id, "A").await;
        let b = make_issue(&db, mission_id, epic_id, "B").await;
        let c = make_issue(&db, mission_id, epic_id, "C").await;
        db.issue_link(a, b, LinkType::Blocks).await.unwrap(); // A → B
        db.issue_link(b, c, LinkType::Blocks).await.unwrap(); // B → C

        let graph = db.blocked_issues_graph("p").await.unwrap();
        // A는 리프(아무도 A를 block하지 않음), B/C는 blocked
        assert!(graph.leaf_blockers.contains(&a));
        assert!(!graph.leaf_blockers.contains(&b)); // B는 A에 blocked
    }

    #[tokio::test]
    async fn test_no_blocks() {
        let db = setup().await;
        let (_sprint_id, epic_id, mission_id) = seed(&db).await;
        make_issue(&db, mission_id, epic_id, "A").await;

        let graph = db.blocked_issues_graph("p").await.unwrap();
        assert!(graph.chains.is_empty());
        assert!(graph.leaf_blockers.is_empty());
        assert!(!graph.has_cycle);
    }

    #[tokio::test]
    async fn test_finished_blocker_excluded() {
        let db = setup().await;
        let (_sprint_id, epic_id, mission_id) = seed(&db).await;
        let a = make_issue(&db, mission_id, epic_id, "A").await;
        let b = make_issue(&db, mission_id, epic_id, "B").await;
        db.issue_link(a, b, LinkType::Blocks).await.unwrap();

        // A를 finished로 전환 (required → ready → working → finished, 사용자 권한으로)
        db.issue_update(a, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "agent").await.unwrap();
        db.issue_update(a, UpdateIssueInput { status: Some(IssueStatus::Working), ..Default::default() }, "agent").await.unwrap();
        db.issue_update(a, UpdateIssueInput { status: Some(IssueStatus::Finished), ..Default::default() }, "user").await.unwrap();

        let graph = db.blocked_issues_graph("p").await.unwrap();
        assert!(graph.chains.is_empty(), "finished blocker는 체인에 포함 안 됨");
    }
}
