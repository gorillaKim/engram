use clap::{Args, Subcommand};
use engram_core::{Db, models::sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus}};
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct SprintArgs {
    #[command(subcommand)]
    pub command: SprintCommand,
}

#[derive(Subcommand)]
pub enum SprintCommand {
    Create {
        #[arg(long)] name: String,
        #[arg(long)] goal: Option<String>,
        #[arg(long)] start: Option<String>,
        #[arg(long)] end: Option<String>,
    },
    List,
    Current,
    Update {
        id: i64,
        #[arg(long)] name: Option<String>,
        #[arg(long)] status: Option<String>,
        #[arg(long)] goal: Option<String>,
    },
    Delete {
        id: i64,
    },
}

pub async fn run(db: Db, args: SprintArgs, fmt: OutputFormat) -> anyhow::Result<()> {
    match args.command {
        SprintCommand::Create { name, goal, start, end } => {
            let sprint = db.sprint_create(CreateSprintInput {
                name, goal, start_date: start, end_date: end,
            }).await?;
            output::print_value(&sprint, fmt)?;
        }
        SprintCommand::List => {
            output::print_value(&db.sprint_list(None).await?, fmt)?;
        }
        SprintCommand::Current => {
            output::print_value(&db.sprint_current().await?, fmt)?;
        }
        SprintCommand::Delete { id } => {
            db.sprint_delete(id).await?;
            output::print_value(&serde_json::json!({ "ok": true, "deleted_id": id }), fmt)?;
        }
        SprintCommand::Update { id, name, status, goal } => {
            let parsed_status = status.as_deref().map(|s| match s {
                "planning"  => Ok(SprintStatus::Planning),
                "active"    => Ok(SprintStatus::Active),
                "completed" => Ok(SprintStatus::Completed),
                "cancelled" => Ok(SprintStatus::Cancelled),
                other       => Err(anyhow::anyhow!("알 수 없는 status: {other}")),
            }).transpose()?;
            let sprint = db.sprint_update(id, UpdateSprintInput {
                name, status: parsed_status, goal, start_date: None, end_date: None,
            }, "user").await?;
            output::print_value(&sprint, fmt)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap { #[command(subcommand)] cmd: SprintCommand }

    #[test]
    fn test_parse_create() {
        let w = Wrap::try_parse_from(["x", "create", "--name", "Sprint #1"]).unwrap();
        match w.cmd {
            SprintCommand::Create { name, .. } => assert_eq!(name, "Sprint #1"),
            _ => panic!("Create 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_update_with_status() {
        let w = Wrap::try_parse_from(["x", "update", "1", "--status", "active"]).unwrap();
        match w.cmd {
            SprintCommand::Update { id, status, .. } => {
                assert_eq!(id, 1);
                assert_eq!(status.as_deref(), Some("active"));
            }
            _ => panic!("Update 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_current_and_list() {
        assert!(matches!(
            Wrap::try_parse_from(["x", "current"]).unwrap().cmd,
            SprintCommand::Current
        ));
        assert!(matches!(
            Wrap::try_parse_from(["x", "list"]).unwrap().cmd,
            SprintCommand::List
        ));
    }
}
