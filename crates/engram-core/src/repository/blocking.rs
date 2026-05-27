use crate::Db;
use sqlx::Row;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockingGraph {
    pub chains: Vec<Vec<i64>>,   // blocker → blocked 경로 (issue id 배열)
    pub leaf_blockers: Vec<i64>, // 해소 가능한 최상위 blocker ids
    pub has_cycle: bool,
    /// issue_id → status 문자열. 프론트에서 finished/cancelled 노드를 시각적으로 구분.
    #[serde(default)]
    pub node_statuses: std::collections::HashMap<i64, String>,
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
            return Ok(BlockingGraph { chains: vec![], leaf_blockers: vec![], has_cycle: false, node_statuses: std::collections::HashMap::new() });
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
                            return Ok(BlockingGraph { chains, leaf_blockers, has_cycle: true, node_statuses: std::collections::HashMap::new() });
                        }
                        new_path.push(next);
                        queue.push_back(new_path);
                    }
                } else {
                    chains.push(path);
                }
            }
        }

        Ok(BlockingGraph { chains, leaf_blockers, has_cycle: false, node_statuses: std::collections::HashMap::new() })
    }

    /// 특정 이슈를 중심으로 프로젝트 경계 없이 블로킹 관계 그래프를 반환한다.
    /// 해당 이슈에서 양방향 BFS로 연결된 서브그래프만 포함.
    pub async fn blocking_graph_for_issue(&self, issue_id: i64) -> crate::Result<BlockingGraph> {
        // 1. 모든 blocks 링크 로드 (status 필터 없음 — finished/cancelled 도 포함)
        let rows = sqlx::query(r#"
            SELECT il.source_id, il.target_id,
                   si.status AS source_status, ti.status AS target_status
            FROM issue_links il
            JOIN issues si ON il.source_id = si.id
            JOIN issues ti ON il.target_id = ti.id
            WHERE il.link_type = 'blocks'
        "#)
        .fetch_all(&self.pool)
        .await?;

        // adjacency: source → [targets] (forward), target → [sources] (backward)
        let mut fwd: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
        let mut bwd: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
        let mut statuses: std::collections::HashMap<i64, String> = std::collections::HashMap::new();

        for row in &rows {
            let src: i64 = row.get("source_id");
            let tgt: i64 = row.get("target_id");
            let src_status: String = row.get("source_status");
            let tgt_status: String = row.get("target_status");
            fwd.entry(src).or_default().push(tgt);
            bwd.entry(tgt).or_default().push(src);
            statuses.insert(src, src_status);
            statuses.insert(tgt, tgt_status);
        }

        // 2. issue_id에서 양방향 BFS로 연결된 노드 수집
        //    finished/cancelled 노드는 직접 연결(terminal)만 포함 — 거기서 더 확장하지 않음
        let is_resolved = |id: i64| -> bool {
            matches!(statuses.get(&id).map(|s| s.as_str()), Some("finished" | "cancelled"))
        };

        let mut connected: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        connected.insert(issue_id);
        queue.push_back(issue_id);

        while let Some(cur) = queue.pop_front() {
            // forward: cur가 block하는 이슈들
            if let Some(targets) = fwd.get(&cur) {
                for &t in targets {
                    if connected.insert(t) {
                        // resolved 노드는 추가만 하고 큐에 넣지 않음 (확장 중단)
                        if !is_resolved(t) {
                            queue.push_back(t);
                        }
                    }
                }
            }
            // backward: cur를 block하는 이슈들
            if let Some(sources) = bwd.get(&cur) {
                for &s in sources {
                    if connected.insert(s) {
                        if !is_resolved(s) {
                            queue.push_back(s);
                        }
                    }
                }
            }
        }

        // 3. 연결된 서브그래프의 adjacency 재구성
        let mut adj: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
        let mut all_targets: std::collections::HashSet<i64> = std::collections::HashSet::new();

        for row in &rows {
            let src: i64 = row.get("source_id");
            let tgt: i64 = row.get("target_id");
            if connected.contains(&src) && connected.contains(&tgt) {
                adj.entry(src).or_default().push(tgt);
                all_targets.insert(tgt);
            }
        }

        if adj.is_empty() {
            return Ok(BlockingGraph { chains: vec![], leaf_blockers: vec![], has_cycle: false, node_statuses: std::collections::HashMap::new() });
        }

        // leaf_blockers
        let mut leaf_blockers: Vec<i64> = adj.keys()
            .filter(|k| !all_targets.contains(*k))
            .copied()
            .collect();
        leaf_blockers.sort();

        // BFS to build chains
        let mut chains = Vec::new();
        for &leaf in &leaf_blockers {
            let mut bfs_queue = std::collections::VecDeque::new();
            bfs_queue.push_back(vec![leaf]);
            while let Some(path) = bfs_queue.pop_front() {
                let last = *path.last().unwrap();
                if let Some(nexts) = adj.get(&last) {
                    for &next in nexts {
                        let mut new_path = path.clone();
                        if new_path.contains(&next) {
                            let node_statuses: std::collections::HashMap<i64, String> = connected.iter().filter_map(|id| statuses.get(id).map(|s| (*id, s.clone()))).collect();
                            return Ok(BlockingGraph { chains, leaf_blockers, has_cycle: true, node_statuses });
                        }
                        new_path.push(next);
                        bfs_queue.push_back(new_path);
                    }
                } else {
                    chains.push(path);
                }
            }
        }

        let node_statuses: std::collections::HashMap<i64, String> = connected.iter().filter_map(|id| statuses.get(id).map(|s| (*id, s.clone()))).collect();
        Ok(BlockingGraph { chains, leaf_blockers, has_cycle: false, node_statuses })
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
            title: "M".to_string(), description: None, jira_key: None,
        }).await.unwrap();
        let epic = db.epic_create(CreateEpicInput { project_key: "p".to_string(), mission_id: Some(mission.id), sprint_id: Some(sprint.id), title: "E".to_string(), description: None }).await.unwrap();
        (sprint.id, epic.id, mission.id)
    }

    async fn make_issue(db: &Db, _mission_id: i64, epic_id: i64, title: &str) -> i64 {
        db.issue_create(CreateIssueInput { epic_id, title: title.to_string(), description: None, goal: None, priority: None }).await.unwrap().id
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

    // ── blocking_graph_for_issue tests ────────────────────────────────────

    #[tokio::test]
    async fn test_for_issue_same_project() {
        let db = setup().await;
        let (_sprint_id, epic_id, mission_id) = seed(&db).await;
        let a = make_issue(&db, mission_id, epic_id, "A").await;
        let b = make_issue(&db, mission_id, epic_id, "B").await;
        db.issue_link(a, b, LinkType::Blocks).await.unwrap();

        // B 관점에서 조회 — A→B 체인이 보여야 함
        let graph = db.blocking_graph_for_issue(b).await.unwrap();
        assert!(!graph.chains.is_empty());
        assert!(graph.leaf_blockers.contains(&a));
        assert!(!graph.has_cycle);

        // A 관점에서도 동일 서브그래프
        let graph2 = db.blocking_graph_for_issue(a).await.unwrap();
        assert_eq!(graph2.chains.len(), graph.chains.len());
    }

    #[tokio::test]
    async fn test_for_issue_cross_project() {
        let db = setup().await;
        let sprint = db.sprint_create(CreateSprintInput { name: "S".to_string(), goal: None, start_date: None, end_date: None }).await.unwrap();
        db.sprint_update(sprint.id, UpdateSprintInput { status: Some(SprintStatus::Active), ..Default::default() }, "agent").await.unwrap();
        let mission = db.mission_create(crate::models::mission::CreateMissionInput {
            title: "M".to_string(), description: None, jira_key: None,
        }).await.unwrap();

        // 프로젝트 A
        let epic_a = db.epic_create(CreateEpicInput { project_key: "proj-a".to_string(), mission_id: Some(mission.id), sprint_id: Some(sprint.id), title: "EA".to_string(), description: None }).await.unwrap();
        // 프로젝트 B
        let epic_b = db.epic_create(CreateEpicInput { project_key: "proj-b".to_string(), mission_id: Some(mission.id), sprint_id: Some(sprint.id), title: "EB".to_string(), description: None }).await.unwrap();

        let issue_a = make_issue(&db, mission.id, epic_a.id, "Issue in A").await;
        let issue_b = make_issue(&db, mission.id, epic_b.id, "Issue in B").await;

        // 크로스 프로젝트: proj-a 이슈가 proj-b 이슈를 block
        db.issue_link(issue_a, issue_b, LinkType::Blocks).await.unwrap();

        // 기존 프로젝트 단위 API — proj-a 기준이면 source만 보이고 target은 체인에 포함
        let graph_a = db.blocked_issues_graph("proj-a").await.unwrap();
        assert!(!graph_a.chains.is_empty(), "proj-a 기준: source가 proj-a라 체인 포함");

        // 새 이슈 중심 API — 어느 쪽에서든 크로스 프로젝트 관계 표시
        let graph_for_b = db.blocking_graph_for_issue(issue_b).await.unwrap();
        assert!(!graph_for_b.chains.is_empty(), "이슈 중심: issue_b에서 크로스 프로젝트 체인 보여야 함");
        assert!(graph_for_b.leaf_blockers.contains(&issue_a));

        let graph_for_a = db.blocking_graph_for_issue(issue_a).await.unwrap();
        assert!(!graph_for_a.chains.is_empty(), "이슈 중심: issue_a에서도 체인 보여야 함");
    }

    #[tokio::test]
    async fn test_for_issue_no_relations() {
        let db = setup().await;
        let (_sprint_id, epic_id, mission_id) = seed(&db).await;
        let a = make_issue(&db, mission_id, epic_id, "Alone").await;

        let graph = db.blocking_graph_for_issue(a).await.unwrap();
        assert!(graph.chains.is_empty());
        assert!(graph.leaf_blockers.is_empty());
        assert!(!graph.has_cycle);
    }
}
