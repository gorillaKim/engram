use clap::{Args, Subcommand};
use engram_core::{Db, models::epic::CreateEpicInput};

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
    }
    Ok(())
}
