use clap::{Args, Subcommand};
use engram_core::{Db, models::epic::{CreateEpicInput, EpicStatus, UpdateEpicInput}};
use crate::output::{self, OutputFormat};

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
        #[arg(long)] project: String,
        #[arg(long)] title: String,
    },
    List {
        #[arg(long)] project: Option<String>,
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

pub async fn run(db: Db, args: EpicArgs, fmt: OutputFormat) -> anyhow::Result<()> {
    match args.command {
        EpicCommand::Create { project, title } => {
            let epic = db.epic_create(CreateEpicInput {
                project_key: project, title, description: None,
            }).await?;
            output::print_value(&epic, fmt)?;
        }
        EpicCommand::List { project } => {
            output::print_value(
                &db.epic_list(project.as_deref(), None).await?,
                fmt,
            )?;
        }
        EpicCommand::Get { id } => {
            output::print_value(&db.epic_get(id).await?, fmt)?;
        }
        EpicCommand::Update { id, status, title, description } => {
            let epic = db.epic_update(id, UpdateEpicInput {
                status: status.as_deref().map(parse_epic_status).transpose()?,
                title,
                description,
            }, "user").await?;
            output::print_value(&epic, fmt)?;
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
