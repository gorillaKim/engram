use clap::{Args, Subcommand};
use engram_core::{Db, models::issue::{CreateIssueInput, IssueFilter, IssueStatus, UpdateIssueInput}};

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
    }
    Ok(())
}
