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
        /// 소속 미션 ID (생략 가능 — nullable)
        #[arg(long = "mission-id")] mission_id: Option<i64>,
        /// 실행 스프린트 (생략 시 백로그)
        #[arg(long)] sprint: Option<i64>,
    },
    List {
        #[arg(long)] project: Option<String>,
        #[arg(long)] status: Option<String>,
        /// 특정 sprint 의 에픽만
        #[arg(long)] sprint: Option<i64>,
        /// sprint_id IS NULL 인 에픽만 (백로그)
        #[arg(long = "backlog-only")] backlog_only: bool,
        /// completed 에픽도 포함 (기본: completed 제외)
        #[arg(long = "include-completed")] include_completed: bool,
    },
    Get { id: i64 },
    /// 에픽 상태/제목/설명 수정
    Update {
        id: i64,
        #[arg(long)] status: Option<String>,
        #[arg(long)] title: Option<String>,
        #[arg(long)] description: Option<String>,
    },
    /// 에픽의 스프린트 변경 (sprint=None 이면 백로그). 산하 이슈가 자동으로 따라옵니다.
    SetSprint {
        #[arg(long = "epic-id")] epic_id: i64,
        #[arg(long)] sprint: Option<i64>,
    },
    /// 에픽 삭제 (하위 이슈/태스크/노트/링크 cascade — 비가역)
    Delete {
        id: i64,
    },
}

pub async fn run(db: Db, args: EpicArgs, fmt: OutputFormat, agent_id: &str) -> anyhow::Result<()> {
    match args.command {
        EpicCommand::Create { project, title, mission_id, sprint } => {
            let epic = db.epic_create(CreateEpicInput {
                project_key: project,
                mission_id,
                sprint_id: sprint,
                title: crate::commands::unescape_newlines(title),
                description: None,
            }).await?;
            output::print_value(&epic, fmt)?;
        }
        EpicCommand::List { project, status: _, sprint, backlog_only, include_completed } => {
            output::print_value(
                &db.epic_list_filtered(project.as_deref(), include_completed, sprint, backlog_only).await?,
                fmt,
            )?;
        }
        EpicCommand::Get { id } => {
            output::print_value(&db.epic_get(id).await?, fmt)?;
        }
        EpicCommand::Update { id, status, title, description } => {
            let epic = db.epic_update(id, UpdateEpicInput {
                status: status.as_deref().map(parse_epic_status).transpose()?,
                title: crate::commands::unescape_newlines_opt(title),
                description: crate::commands::unescape_newlines_opt(description),
                mission_id: None,
                sprint_id: None,
                update_sprint_id: false,
            }, agent_id).await?;
            output::print_value(&epic, fmt)?;
        }
        EpicCommand::SetSprint { epic_id, sprint } => {
            let epic = db.epic_set_sprint(epic_id, sprint, agent_id).await?;
            output::print_value(&epic, fmt)?;
        }
        EpicCommand::Delete { id } => {
            db.epic_delete(id, agent_id).await?;
            output::print_value(
                &serde_json::json!({ "ok": true, "deleted_id": id }),
                fmt,
            )?;
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

    #[test]
    fn test_parse_delete() {
        let w = Wrap::try_parse_from(["x", "delete", "12"]).unwrap();
        match w.cmd {
            EpicCommand::Delete { id } => assert_eq!(id, 12),
            _ => panic!("Delete 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_list_with_status() {
        let w = Wrap::try_parse_from(["x", "list", "--project", "engram", "--status", "active"]).unwrap();
        match w.cmd {
            EpicCommand::List { project, status, .. } => {
                assert_eq!(project.as_deref(), Some("engram"));
                assert_eq!(status.as_deref(), Some("active"));
            }
            _ => panic!("List 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_set_sprint() {
        let w = Wrap::try_parse_from(["x", "set-sprint", "--epic-id", "8", "--sprint", "3"]).unwrap();
        match w.cmd {
            EpicCommand::SetSprint { epic_id, sprint } => {
                assert_eq!(epic_id, 8);
                assert_eq!(sprint, Some(3));
            }
            _ => panic!("SetSprint 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_set_sprint_to_backlog() {
        let w = Wrap::try_parse_from(["x", "set-sprint", "--epic-id", "8"]).unwrap();
        match w.cmd {
            EpicCommand::SetSprint { epic_id, sprint } => {
                assert_eq!(epic_id, 8);
                assert!(sprint.is_none());
            }
            _ => panic!("SetSprint 변형이 파싱되어야 함"),
        }
    }
}
