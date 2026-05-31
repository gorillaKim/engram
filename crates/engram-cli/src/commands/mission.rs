use clap::{Args, Subcommand};
use engram_core::{
    Db,
    models::mission::{CreateMissionInput, MissionFilter, MissionStatus, UpdateMissionInput},
};
use crate::output::{self, OutputFormat};

fn parse_mission_status(s: &str) -> anyhow::Result<MissionStatus> {
    match s {
        "active"    => Ok(MissionStatus::Active),
        "completed" => Ok(MissionStatus::Completed),
        "cancelled" => Ok(MissionStatus::Cancelled),
        other       => Err(anyhow::anyhow!("알 수 없는 mission status: {other}")),
    }
}

#[derive(Args)]
pub struct MissionArgs {
    #[command(subcommand)]
    pub command: MissionCommand,
}

#[derive(Subcommand)]
pub enum MissionCommand {
    /// 미션 생성 (sprint-agnostic 한 전략 목표 — sprint 는 산하 에픽이 보유)
    Create {
        #[arg(long)] title: String,
        #[arg(long)] description: Option<String>,
        #[arg(long = "jira-key")] jira_key: Option<String>,
    },
    /// 미션 목록 조회
    List {
        /// completed/cancelled 미션도 포함
        #[arg(long = "include-completed")] include_completed: bool,
        #[arg(long = "project-key")] project_key: Option<String>,
        #[arg(long = "sprint-id")] sprint_id: Option<i64>,
    },
    /// 미션 상세 조회
    Get {
        id: i64,
    },
    /// 미션 수정
    Update {
        id: i64,
        #[arg(long)] title: Option<String>,
        #[arg(long)] description: Option<String>,
        #[arg(long)] status: Option<String>,
        #[arg(long = "jira-key")] jira_key: Option<String>,
    },
    /// 미션 삭제 (하위 epic이 없어야 함)
    Delete {
        id: i64,
    },
    /// 미션 계층 트리 조회 (Mission → Epics → Issues)
    GetTree {
        id: i64,
        /// jira_key 로 미션을 조회 (id 대신 사용 가능)
        #[arg(long = "jira-key")] jira_key: Option<String>,
    },
}

pub async fn run(db: Db, args: MissionArgs, fmt: OutputFormat, agent_id: &str) -> anyhow::Result<()> {
    match args.command {
        MissionCommand::Create { title, description, jira_key } => {
            let mission = db.mission_create(CreateMissionInput {
                title,
                description,
                jira_key,
            }).await?;
            output::print_value(&mission, fmt)?;
        }
        MissionCommand::List { include_completed, project_key, sprint_id } => {
            let missions = db.mission_list(MissionFilter {
                status: None,
                include_completed,
                project_key,
                sprint_id,
            }).await?;
            output::print_value(&missions, fmt)?;
        }
        MissionCommand::Get { id } => {
            output::print_value(&db.mission_get(id).await?, fmt)?;
        }
        MissionCommand::Update { id, title, description, status, jira_key } => {
            let mission = db.mission_update(id, UpdateMissionInput {
                title,
                description,
                jira_key,
                status: status.as_deref().map(parse_mission_status).transpose()?,
            }, agent_id).await?;
            output::print_value(&mission, fmt)?;
        }
        MissionCommand::Delete { id } => {
            db.mission_delete(id).await?;
            output::print_value(
                &serde_json::json!({ "ok": true, "deleted_id": id }),
                fmt,
            )?;
        }
        MissionCommand::GetTree { id, jira_key } => {
            let resolved_id = if let Some(ref key) = jira_key {
                db.mission_get_by_jira_key(key).await?.id
            } else {
                id
            };
            let tree = db.mission_get_tree(resolved_id).await?;
            output::print_value(&tree, fmt)?;
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
        cmd: MissionCommand,
    }

    fn parse(args: &[&str]) -> MissionCommand {
        Wrap::try_parse_from(std::iter::once(&"engram-test").chain(args.iter())).unwrap().cmd
    }

    #[test]
    fn test_parse_create_minimal() {
        let cmd = parse(&["create", "--title", "M6 Mission"]);
        match cmd {
            MissionCommand::Create { title, description, jira_key } => {
                assert_eq!(title, "M6 Mission");
                assert!(description.is_none());
                assert!(jira_key.is_none());
            }
            _ => panic!("Create 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_create_full() {
        let cmd = parse(&[
            "create", "--title", "Full Mission",
            "--description", "Desc",
            "--jira-key", "PROJ-1",
        ]);
        match cmd {
            MissionCommand::Create { title, description, jira_key } => {
                assert_eq!(title, "Full Mission");
                assert_eq!(description.as_deref(), Some("Desc"));
                assert_eq!(jira_key.as_deref(), Some("PROJ-1"));
            }
            _ => panic!("Create 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_list_defaults() {
        let cmd = parse(&["list"]);
        match cmd {
            MissionCommand::List { include_completed, .. } => {
                assert!(!include_completed);
            }
            _ => panic!("List 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_list_include_completed() {
        let cmd = parse(&["list", "--include-completed"]);
        match cmd {
            MissionCommand::List { include_completed, .. } => {
                assert!(include_completed);
            }
            _ => panic!("List 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_get() {
        let cmd = parse(&["get", "7"]);
        match cmd {
            MissionCommand::Get { id } => assert_eq!(id, 7),
            _ => panic!("Get 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_update_status() {
        let cmd = parse(&["update", "5", "--status", "completed"]);
        match cmd {
            MissionCommand::Update { id, status, .. } => {
                assert_eq!(id, 5);
                assert_eq!(status.as_deref(), Some("completed"));
            }
            _ => panic!("Update 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_update_full() {
        let cmd = parse(&[
            "update", "5",
            "--title", "New Title",
            "--jira-key", "X-99",
        ]);
        match cmd {
            MissionCommand::Update { id, title, jira_key, .. } => {
                assert_eq!(id, 5);
                assert_eq!(title.as_deref(), Some("New Title"));
                assert_eq!(jira_key.as_deref(), Some("X-99"));
            }
            _ => panic!("Update 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let cmd = parse(&["delete", "12"]);
        match cmd {
            MissionCommand::Delete { id } => assert_eq!(id, 12),
            _ => panic!("Delete 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_get_tree_by_id() {
        let cmd = parse(&["get-tree", "3"]);
        match cmd {
            MissionCommand::GetTree { id, jira_key } => {
                assert_eq!(id, 3);
                assert!(jira_key.is_none());
            }
            _ => panic!("GetTree 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_mission_status_helper() {
        assert_eq!(parse_mission_status("active").unwrap(), MissionStatus::Active);
        assert_eq!(parse_mission_status("completed").unwrap(), MissionStatus::Completed);
        assert_eq!(parse_mission_status("cancelled").unwrap(), MissionStatus::Cancelled);
        assert!(parse_mission_status("invalid").is_err());
    }
}
