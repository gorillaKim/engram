use clap::{Args, Subcommand};
use engram_core::{Db, models::sprint::CreateSprintInput};

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
}

pub async fn run(db: Db, args: SprintArgs) -> anyhow::Result<()> {
    match args.command {
        SprintCommand::Create { name, goal, start, end } => {
            let sprint = db.sprint_create(CreateSprintInput {
                name, goal, start_date: start, end_date: end,
            }).await?;
            println!("{}", serde_json::to_string_pretty(&sprint)?);
        }
        SprintCommand::List => {
            println!("{}", serde_json::to_string_pretty(&db.sprint_list(None).await?)?);
        }
        SprintCommand::Current => {
            println!("{}", serde_json::to_string_pretty(&db.sprint_current().await?)?);
        }
    }
    Ok(())
}
