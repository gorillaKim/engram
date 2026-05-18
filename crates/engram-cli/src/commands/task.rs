use clap::{Args, Subcommand};
use engram_core::{Db, models::task::{CreateTaskInput, TaskSource, TaskStatus, UpdateTaskInput}};
use crate::output::{self, OutputFormat};

fn parse_task_status(s: &str) -> anyhow::Result<TaskStatus> {
    match s {
        "required"  => Ok(TaskStatus::Required),
        "ready"     => Ok(TaskStatus::Ready),
        "working"   => Ok(TaskStatus::Working),
        "demo"      => Ok(TaskStatus::Demo),
        "finished"  => Ok(TaskStatus::Finished),
        "cancelled" => Ok(TaskStatus::Cancelled),
        other       => Err(anyhow::anyhow!("알 수 없는 task status: {other}")),
    }
}

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
    /// 태스크 상태/제목 수정
    Update {
        id: i64,
        #[arg(long)] status: Option<String>,
        #[arg(long)] title: Option<String>,
    },
    /// 특정 태스크 다음에 새 태스크를 삽입 (source=agent_discovered)
    InsertAfter {
        #[arg(long)] issue: i64,
        #[arg(long = "after")] after_task_id: i64,
        #[arg(long)] title: String,
    },
}

pub async fn run(db: Db, args: TaskArgs, fmt: OutputFormat) -> anyhow::Result<()> {
    match args.command {
        TaskCommand::Create { issue, title, goal } => {
            let task = db.task_create(CreateTaskInput {
                issue_id: issue, title, description: None, goal, after_task_id: None, source: None,
            }).await?;
            output::print_value(&task, fmt)?;
        }
        TaskCommand::List { issue } => {
            output::print_value(&db.task_list(issue, None).await?, fmt)?;
        }
        TaskCommand::Finish { id } => {
            let task = db.task_update(id, UpdateTaskInput {
                status: Some(TaskStatus::Finished), ..Default::default()
            }, "user").await?;
            output::print_value(&task, fmt)?;
        }
        TaskCommand::Next { project } => {
            let next = db.task_next(project.as_deref(), None).await?;
            // null vs object 둘 다 valid JSON.
            output::print_value(&next, fmt)?;
        }
        TaskCommand::Update { id, status, title } => {
            let task = db.task_update(id, UpdateTaskInput {
                status: status.as_deref().map(parse_task_status).transpose()?,
                title,
                ..Default::default()
            }, "user").await?;
            output::print_value(&task, fmt)?;
        }
        TaskCommand::InsertAfter { issue, after_task_id, title } => {
            let task = db.task_create(CreateTaskInput {
                issue_id: issue,
                title,
                description: None,
                goal: None,
                after_task_id: Some(after_task_id),
                source: Some(TaskSource::AgentDiscovered),
            }).await?;
            output::print_value(&task, fmt)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap { #[command(subcommand)] cmd: TaskCommand }

    #[test]
    fn test_parse_update_status() {
        let w = Wrap::try_parse_from(["x", "update", "3", "--status", "working"]).unwrap();
        match w.cmd {
            TaskCommand::Update { id, status, .. } => {
                assert_eq!(id, 3);
                assert_eq!(status.as_deref(), Some("working"));
            }
            _ => panic!("Update 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_insert_after() {
        let w = Wrap::try_parse_from(
            ["x", "insert-after", "--issue", "1", "--after", "5", "--title", "discovered"]
        ).unwrap();
        match w.cmd {
            TaskCommand::InsertAfter { issue, after_task_id, title } => {
                assert_eq!(issue, 1);
                assert_eq!(after_task_id, 5);
                assert_eq!(title, "discovered");
            }
            _ => panic!("InsertAfter 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_task_status_helper() {
        assert_eq!(parse_task_status("ready").unwrap(), TaskStatus::Ready);
        assert!(parse_task_status("xx").is_err());
    }
}
