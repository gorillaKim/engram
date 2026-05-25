use clap::{Args, Subcommand};
use engram_core::{Db, models::issue::{
    CreateIssueInput, IssueFilter, IssuePriority, IssueStatus, LinkType, UpdateIssueInput,
}};
use crate::output::{self, OutputFormat};

fn parse_status(s: &str) -> anyhow::Result<IssueStatus> {
    match s {
        "required"  => Ok(IssueStatus::Required),
        "ready"     => Ok(IssueStatus::Ready),
        "working"   => Ok(IssueStatus::Working),
        "demo"      => Ok(IssueStatus::Demo),
        "finished"  => Ok(IssueStatus::Finished),
        "cancelled" => Ok(IssueStatus::Cancelled),
        other       => Err(anyhow::anyhow!("알 수 없는 status: {other}")),
    }
}

/// `issue release --transition-to` 가 받을 수 있는 값은 MCP `issue_release` 스키마와 동일하게
/// ready / demo / required 만 허용 (working/finished/cancelled 거부).
fn parse_release_transition(s: &str) -> anyhow::Result<IssueStatus> {
    match s {
        "ready"    => Ok(IssueStatus::Ready),
        "demo"     => Ok(IssueStatus::Demo),
        "required" => Ok(IssueStatus::Required),
        other      => Err(anyhow::anyhow!(
            "issue release --transition-to 는 ready/demo/required 만 허용 (받은 값: {other})"
        )),
    }
}

fn parse_priority(s: &str) -> anyhow::Result<IssuePriority> {
    match s {
        "critical" => Ok(IssuePriority::Critical),
        "high"     => Ok(IssuePriority::High),
        "medium"   => Ok(IssuePriority::Medium),
        "low"      => Ok(IssuePriority::Low),
        other      => Err(anyhow::anyhow!("알 수 없는 priority: {other}")),
    }
}

fn parse_link_type(s: &str) -> anyhow::Result<LinkType> {
    match s {
        "blocks"     => Ok(LinkType::Blocks),
        "relates_to" => Ok(LinkType::RelatesTo),
        "duplicates" => Ok(LinkType::Duplicates),
        other        => Err(anyhow::anyhow!("알 수 없는 link_type: {other}")),
    }
}

#[derive(Args)]
pub struct IssueArgs {
    #[command(subcommand)]
    pub command: IssueCommand,
}

#[derive(Subcommand)]
pub enum IssueCommand {
    Create {
        #[arg(long)] epic: i64,
        #[arg(long)] title: String,
        #[arg(long)] goal: Option<String>,
        #[arg(long)] description: Option<String>,
    },
    /// 이슈 목록. IssueFilter 전체 노출.
    List {
        #[arg(long)] project: Option<String>,
        #[arg(long)] epic: Option<i64>,
        /// 특정 미션 소속 이슈만 필터링
        #[arg(long)] mission: Option<i64>,
        #[arg(long)] sprint: Option<i64>,
        #[arg(long = "backlog-only")] backlog_only: bool,
        #[arg(long)] status: Vec<String>,
        #[arg(long)] priority: Option<String>,
    },
    Get {
        id: i64,
        #[arg(long)] compact: bool,
    },
    Ready { id: i64 },
    /// 임의 상태 전이 / 우선순위 변경
    Update {
        id: i64,
        #[arg(long)] status: Option<String>,
        #[arg(long)] priority: Option<String>,
        #[arg(long)] title: Option<String>,
        #[arg(long)] goal: Option<String>,
        #[arg(long)] description: Option<String>,
    },
    /// 두 이슈 간 관계 생성
    Link {
        #[arg(long)] source: i64,
        #[arg(long)] target: i64,
        #[arg(long, value_name = "TYPE", default_value = "blocks")] r#type: String,
    },
    /// 이슈 관계 제거
    Unlink {
        #[arg(long = "link-id")] link_id: i64,
    },
    /// 이슈 점유 (CAS). 멀티 에이전트 안전. ADR-0009 / agent-demo-gate.md 참조.
    Claim {
        id: i64,
        /// 글로벌 --agent-id 로 대체 가능. 둘 다 없으면 에러.
        #[arg(long = "agent-id")] agent_id: Option<String>,
    },
    /// 점유 해제 + 지정 상태로 전이. transition_to ∈ {ready, demo, required}.
    Release {
        id: i64,
        /// 글로벌 --agent-id 로 대체 가능. 둘 다 없으면 에러.
        #[arg(long = "agent-id")] agent_id: Option<String>,
        #[arg(long = "transition-to")] transition_to: String,
        #[arg(long)] force: bool,
    },
    /// 이슈 삭제 (하위 task/notes/links cascade)
    Delete {
        id: i64,
    },
    /// 이슈 완료 (demo -> finished 사용자 전용)
    Finish {
        id: i64,
    },
    /// 이슈 취소 (사용자 전용)
    Cancel {
        id: i64,
        #[arg(long)] reason: String,
    },
    /// 이슈 일괄 업데이트
    BulkUpdate {
        /// 업데이트할 이슈 ID 목록 (쉼표 구분)
        #[arg(long, value_delimiter = ',')]
        ids: Option<Vec<i64>>,

        /// 표준 입력(stdin)에서 쉼표나 개행으로 구분된 이슈 ID 목록을 입력받음
        #[arg(long = "ids-from-stdin")]
        ids_from_stdin: bool,

        #[arg(long)]
        status: Option<String>,

        #[arg(long)]
        priority: Option<String>,
    },
}

pub async fn run(db: Db, args: IssueArgs, fmt: OutputFormat, agent_id: &str) -> anyhow::Result<()> {
    match args.command {
        IssueCommand::Create { epic, title, goal, description } => {
            let issue = db.issue_create(CreateIssueInput {
                epic_id: epic, title, description, goal, priority: None,
            }).await?;
            output::print_value(&issue, fmt)?;
        }
        IssueCommand::List { project, epic, mission, sprint, backlog_only, status, priority } => {
            let mut target_statuses = Vec::new();
            for s in status {
                target_statuses.push(parse_status(&s)?);
            }
            let list = db.issue_list(IssueFilter {
                epic_id: epic,
                mission_id: mission,
                sprint_id: sprint,
                backlog_only,
                project_key: project,
                status: None,
                statuses: if target_statuses.is_empty() { None } else { Some(target_statuses) },
                priority: priority.as_deref().map(parse_priority).transpose()?,
            }).await?;
            output::print_value(&list, fmt)?;
        }
        IssueCommand::Get { id, compact } => {
            output::print_value(&db.issue_get(id, compact).await?, fmt)?;
        }
        IssueCommand::Ready { id } => {
            let issue = db.issue_update(id, UpdateIssueInput {
                status: Some(IssueStatus::Ready), ..Default::default()
            }, agent_id).await?;
            output::print_value(&issue, fmt)?;
        }
        IssueCommand::Update { id, status, priority, title, goal, description } => {
            let issue = db.issue_update(id, UpdateIssueInput {
                status: status.as_deref().map(parse_status).transpose()?,
                priority: priority.as_deref().map(parse_priority).transpose()?,
                title,
                description,
                goal,
                epic_id: None,
            }, agent_id).await?;
            output::print_value(&issue, fmt)?;
        }
        IssueCommand::Link { source, target, r#type } => {
            let lt = parse_link_type(&r#type)?;
            let link = db.issue_link(source, target, lt).await?;
            output::print_value(&link, fmt)?;
        }
        IssueCommand::Unlink { link_id } => {
            db.issue_unlink(link_id).await?;
            output::print_value(
                &serde_json::json!({ "ok": true, "deleted_id": link_id }),
                fmt,
            )?;
        }
        IssueCommand::Claim { id, agent_id: cmd_agent_id } => {
            let effective = cmd_agent_id.as_deref().unwrap_or(agent_id);
            if effective == "user" {
                anyhow::bail!("issue claim 은 --agent-id 가 필요합니다 (글로벌 또는 서브커맨드)");
            }
            let issue = db.issue_claim(id, effective).await?;
            output::print_value(&issue, fmt)?;
        }
        IssueCommand::Release { id, agent_id: cmd_agent_id, transition_to, force } => {
            let effective = cmd_agent_id.as_deref().unwrap_or(agent_id);
            if effective == "user" {
                anyhow::bail!("issue release 은 --agent-id 가 필요합니다 (글로벌 또는 서브커맨드)");
            }
            let st = parse_release_transition(&transition_to)?;
            let issue = db.issue_release(id, st, effective, force).await?;
            output::print_value(&issue, fmt)?;
        }
        IssueCommand::Delete { id } => {
            db.issue_delete(id, agent_id).await?;
            output::print_value(
                &serde_json::json!({ "ok": true, "deleted_id": id }),
                fmt,
            )?;
        }
        IssueCommand::Finish { id } => {
            let issue = db.issue_finish(id, agent_id).await?;
            output::print_value(&issue, fmt)?;
        }
        IssueCommand::Cancel { id, reason } => {
            let issue = db.issue_cancel(id, &reason, agent_id).await?;
            output::print_value(&issue, fmt)?;
        }
        IssueCommand::BulkUpdate { ids, ids_from_stdin, status, priority } => {
            let mut target_ids = ids.unwrap_or_default();
            if ids_from_stdin {
                use std::io::{self, Read};
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                for token in buffer.split(|c: char| c == ',' || c.is_whitespace()) {
                    let token = token.trim();
                    if !token.is_empty() {
                        if let Ok(id) = token.parse::<i64>() {
                            target_ids.push(id);
                        } else {
                            anyhow::bail!("stdin에서 올바르지 않은 ID 토큰 발견: '{}'", token);
                        }
                    }
                }
            }

            if target_ids.is_empty() {
                anyhow::bail!("업데이트할 이슈 ID가 지정되지 않았습니다. --ids 또는 --ids-from-stdin을 사용하세요.");
            }

            let input = engram_core::models::issue::BulkUpdateInput {
                status: status.as_deref().map(parse_status).transpose()?,
                priority: priority.as_deref().map(parse_priority).transpose()?,
            };

            let result = db.issue_bulk_update(target_ids, input, agent_id).await?;
            output::print_value(&result, fmt)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap {
        #[command(subcommand)]
        cmd: IssueCommand,
    }

    fn parse(args: &[&str]) -> IssueCommand {
        Wrap::try_parse_from(std::iter::once(&"engram-test").chain(args.iter())).unwrap().cmd
    }

    #[test]
    fn test_parse_update_with_status() {
        let cmd = parse(&["update", "1", "--status", "working"]);
        match cmd {
            IssueCommand::Update { id, status, .. } => {
                assert_eq!(id, 1);
                assert_eq!(status.as_deref(), Some("working"));
            }
            _ => panic!("Update 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_create_full() {
        let cmd = parse(&["create", "--epic", "2", "--title", "Hello", "--goal", "MyGoal", "--description", "Desc"]);
        match cmd {
            IssueCommand::Create { epic, title, goal, description } => {
                assert_eq!(epic, 2);
                assert_eq!(title, "Hello");
                assert_eq!(goal.as_deref(), Some("MyGoal"));
                assert_eq!(description.as_deref(), Some("Desc"));
            }
            _ => panic!("Create 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_update_full() {
        let cmd = parse(&["update", "5", "--goal", "NewGoal", "--description", "NewDesc"]);
        match cmd {
            IssueCommand::Update { id, goal, description, .. } => {
                assert_eq!(id, 5);
                assert_eq!(goal.as_deref(), Some("NewGoal"));
                assert_eq!(description.as_deref(), Some("NewDesc"));
            }
            _ => panic!("Update 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_link_default_blocks() {
        let cmd = parse(&["link", "--source", "10", "--target", "20"]);
        match cmd {
            IssueCommand::Link { source, target, r#type } => {
                assert_eq!(source, 10);
                assert_eq!(target, 20);
                assert_eq!(r#type, "blocks", "기본 link_type은 blocks");
            }
            _ => panic!("Link 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_unlink() {
        let cmd = parse(&["unlink", "--link-id", "7"]);
        match cmd {
            IssueCommand::Unlink { link_id } => assert_eq!(link_id, 7),
            _ => panic!("Unlink 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_status_helpers() {
        assert_eq!(parse_status("working").unwrap(), IssueStatus::Working);
        assert!(parse_status("nonsense").is_err());
        assert_eq!(parse_priority("critical").unwrap(), IssuePriority::Critical);
        assert_eq!(parse_link_type("relates_to").unwrap(), LinkType::RelatesTo);
    }

    #[test]
    fn test_parse_claim() {
        let cmd = parse(&["claim", "7", "--agent-id", "me@sess-1"]);
        match cmd {
            IssueCommand::Claim { id, agent_id } => {
                assert_eq!(id, 7);
                assert_eq!(agent_id.as_deref(), Some("me@sess-1"));
            }
            _ => panic!("Claim 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_claim_without_agent_id() {
        let cmd = parse(&["claim", "7"]);
        match cmd {
            IssueCommand::Claim { id, agent_id } => {
                assert_eq!(id, 7);
                assert_eq!(agent_id, None);
            }
            _ => panic!("Claim 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_release_with_force() {
        let cmd = parse(&[
            "release", "7", "--agent-id", "me@s", "--transition-to", "demo", "--force",
        ]);
        match cmd {
            IssueCommand::Release { id, agent_id, transition_to, force } => {
                assert_eq!(id, 7);
                assert_eq!(agent_id.as_deref(), Some("me@s"));
                assert_eq!(transition_to, "demo");
                assert!(force);
            }
            _ => panic!("Release 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_release_transition_helper() {
        assert_eq!(parse_release_transition("ready").unwrap(),    IssueStatus::Ready);
        assert_eq!(parse_release_transition("demo").unwrap(),     IssueStatus::Demo);
        assert_eq!(parse_release_transition("required").unwrap(), IssueStatus::Required);
        assert!(parse_release_transition("working").is_err());
        assert!(parse_release_transition("finished").is_err());
    }

    #[test]
    fn test_parse_set_sprint_command_removed() {
        // issue set-sprint 는 ADR-0014 로 제거됨 — clap 파싱 실패
        let res = Wrap::try_parse_from(&["x", "set-sprint", "5", "--sprint", "3"]);
        assert!(res.is_err(), "issue set-sprint 는 제거되었어야 함");
    }

    #[test]
    fn test_parse_delete() {
        let cmd = parse(&["delete", "9"]);
        match cmd {
            IssueCommand::Delete { id } => assert_eq!(id, 9),
            _ => panic!("Delete 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_finish() {
        let cmd = parse(&["finish", "10"]);
        match cmd {
            IssueCommand::Finish { id } => assert_eq!(id, 10),
            _ => panic!("Finish 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_cancel() {
        let cmd = parse(&["cancel", "11", "--reason", "Not needed anymore"]);
        match cmd {
            IssueCommand::Cancel { id, reason } => {
                assert_eq!(id, 11);
                assert_eq!(reason, "Not needed anymore");
            }
            _ => panic!("Cancel 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_list_full_filters() {
        let cmd = parse(&[
            "list", "--project", "engram", "--epic", "4",
            "--sprint", "2", "--backlog-only",
            "--status", "ready", "--status", "working", "--priority", "high",
        ]);
        match cmd {
            IssueCommand::List { project, epic, sprint, backlog_only, status, priority, .. } => {
                assert_eq!(project.as_deref(), Some("engram"));
                assert_eq!(epic, Some(4));
                assert_eq!(sprint, Some(2));
                assert!(backlog_only);
                assert_eq!(status, vec!["ready".to_string(), "working".to_string()]);
                assert_eq!(priority.as_deref(), Some("high"));
            }
            _ => panic!("List 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_list_with_mission() {
        let cmd = parse(&["list", "--mission", "7"]);
        match cmd {
            IssueCommand::List { mission, .. } => {
                assert_eq!(mission, Some(7), "--mission 옵션이 파싱되어야 함");
            }
            _ => panic!("List 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_create_with_mission_id_fails() {
        // --mission-id 옵션은 ADR-0014 로 제거됨 — clap 파싱 실패해야 함.
        let args = &["create", "--epic", "2", "--title", "Hello", "--mission-id", "5"];
        let res = Wrap::try_parse_from(std::iter::once(&"engram-test").chain(args.iter()));
        assert!(res.is_err(), "issue create --mission-id should fail parsing");
    }
}
