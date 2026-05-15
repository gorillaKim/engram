use clap::{Args, Subcommand};
use engram_core::{Db, models::task::{CreateTaskInput, TaskStatus, UpdateTaskInput}};

#[derive(Args)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub command: TaskCommand,
}

#[derive(Subcommand)]
pub enum TaskCommand {
    Create   { #[arg(long)] issue: i64, #[arg(long)] title: String, #[arg(long)] goal: Option<String> },
    List     { #[arg(long)] issue: i64 },
    Finish   { id: i64 },
    Next     { #[arg(long)] project: Option<String> },
}

pub async fn run(db: Db, args: TaskArgs) -> anyhow::Result<()> {
    match args.command {
        TaskCommand::Create { issue, title, goal } => {
            let task = db.task_create(CreateTaskInput {
                issue_id: issue, title, description: None, goal, after_task_id: None, source: None,
            }).await?;
            println!("{}", serde_json::to_string_pretty(&task)?);
        }
        TaskCommand::List { issue } => {
            println!("{}", serde_json::to_string_pretty(&db.task_list(issue, None).await?)?);
        }
        TaskCommand::Finish { id } => {
            let task = db.task_update(id, UpdateTaskInput {
                status: Some(TaskStatus::Finished), ..Default::default()
            }).await?;
            println!("✅ 태스크 완료: {}", task.title);
        }
        TaskCommand::Next { project } => {
            let next = db.task_next(project.as_deref(), None).await?;
            match next {
                Some(t) => println!("{}", serde_json::to_string_pretty(&t)?),
                None    => println!("처리할 태스크가 없습니다."),
            }
        }
    }
    Ok(())
}
