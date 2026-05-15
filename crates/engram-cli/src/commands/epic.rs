use clap::{Args, Subcommand};
use engram_core::{Db, models::epic::{CreateEpicInput, EpicStatus, UpdateEpicInput}};

fn parse_epic_status(s: &str) -> anyhow::Result<EpicStatus> {
    match s {
        "active"    => Ok(EpicStatus::Active),
        "completed" => Ok(EpicStatus::Completed),
        "cancelled" => Ok(EpicStatus::Cancelled),
        other       => Err(anyhow::anyhow!("알 수 없는 epic status: {other}")),
    }
}

#[derive(Args)]
pub struct EpicArgs {
    #[command(subcommand)]
    pub command: EpicCommand,
}

#[derive(Subcommand)]
pub enum EpicCommand {
    Create {
        #[arg(long)] sprint: i64,
        #[arg(long)] project: String,
        #[arg(long)] title: String,
    },
    List {
        #[arg(long)] project: Option<String>,
        #[arg(long)] sprint: Option<i64>,
    },
    Get { id: i64 },
    /// 에픽 상태/제목/설명 수정
    Update {
        id: i64,
        #[arg(long)] status: Option<String>,
        #[arg(long)] title: Option<String>,
        #[arg(long)] description: Option<String>,
    },
}

pub async fn run(db: Db, args: EpicArgs) -> anyhow::Result<()> {
    match args.command {
        EpicCommand::Create { sprint, project, title } => {
            let epic = db.epic_create(CreateEpicInput {
                sprint_id: sprint, project_key: project, title, description: None,
            }).await?;
            println!("{}", serde_json::to_string_pretty(&epic)?);
        }
        EpicCommand::List { project, sprint } => {
            println!("{}", serde_json::to_string_pretty(
                &db.epic_list(sprint, project.as_deref(), None).await?
            )?);
        }
        EpicCommand::Get { id } => {
            println!("{}", serde_json::to_string_pretty(&db.epic_get(id).await?)?);
        }
        EpicCommand::Update { id, status, title, description } => {
            let epic = db.epic_update(id, UpdateEpicInput {
                status: status.as_deref().map(parse_epic_status).transpose()?,
                title,
                description,
            }).await?;
            println!("{}", serde_json::to_string_pretty(&epic)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap { #[command(subcommand)] cmd: EpicCommand }

    #[test]
    fn test_parse_update_status() {
        let w = Wrap::try_parse_from(["x", "update", "5", "--status", "completed"]).unwrap();
        match w.cmd {
            EpicCommand::Update { id, status, .. } => {
                assert_eq!(id, 5);
                assert_eq!(status.as_deref(), Some("completed"));
            }
            _ => panic!("Update 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_epic_status_helper() {
        assert_eq!(parse_epic_status("active").unwrap(), EpicStatus::Active);
        assert!(parse_epic_status("garbage").is_err());
    }
}
