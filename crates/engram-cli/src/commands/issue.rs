use clap::{Args, Subcommand};
use engram_core::{Db, models::issue::{
    CreateIssueInput, IssueFilter, IssuePriority, IssueStatus, LinkType, UpdateIssueInput,
}};

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
    Create { #[arg(long)] epic: i64, #[arg(long)] title: String },
    List { #[arg(long)] project: Option<String>, #[arg(long)] epic: Option<i64> },
    Get { id: i64 },
    Ready { id: i64 },
    /// 임의 상태 전이 / 우선순위 변경
    Update {
        id: i64,
        #[arg(long)] status: Option<String>,
        #[arg(long)] priority: Option<String>,
        #[arg(long)] title: Option<String>,
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
}

pub async fn run(db: Db, args: IssueArgs) -> anyhow::Result<()> {
    match args.command {
        IssueCommand::Create { epic, title } => {
            let issue = db.issue_create(CreateIssueInput {
                epic_id: epic, title, description: None, goal: None, priority: None,
            }).await?;
            println!("{}", serde_json::to_string_pretty(&issue)?);
        }
        IssueCommand::List { project, epic } => {
            let list = db.issue_list(IssueFilter {
                epic_id: epic, project_key: project, ..Default::default()
            }).await?;
            println!("{}", serde_json::to_string_pretty(&list)?);
        }
        IssueCommand::Get { id } => {
            println!("{}", serde_json::to_string_pretty(&db.issue_get(id).await?)?);
        }
        IssueCommand::Ready { id } => {
            let issue = db.issue_update(id, UpdateIssueInput {
                status: Some(IssueStatus::Ready), ..Default::default()
            }).await?;
            println!("✅ 이슈 준비됨: {}", issue.title);
        }
        IssueCommand::Update { id, status, priority, title } => {
            let issue = db.issue_update(id, UpdateIssueInput {
                status: status.as_deref().map(parse_status).transpose()?,
                priority: priority.as_deref().map(parse_priority).transpose()?,
                title,
                ..Default::default()
            }).await?;
            println!("{}", serde_json::to_string_pretty(&issue)?);
        }
        IssueCommand::Link { source, target, r#type } => {
            let lt = parse_link_type(&r#type)?;
            let link = db.issue_link(source, target, lt).await?;
            println!("{}", serde_json::to_string_pretty(&link)?);
        }
        IssueCommand::Unlink { link_id } => {
            db.issue_unlink(link_id).await?;
            println!("✅ 링크 제거됨: #{link_id}");
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
}
