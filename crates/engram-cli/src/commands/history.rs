use clap::{Args, Subcommand};
use engram_core::{Db, models::history::EntityType};
use crate::output::{self, OutputFormat};

fn parse_entity_type(s: &str) -> anyhow::Result<EntityType> {
    match s {
        "sprint" => Ok(EntityType::Sprint),
        "epic"   => Ok(EntityType::Epic),
        "issue"  => Ok(EntityType::Issue),
        "task"   => Ok(EntityType::Task),
        "note"   => Ok(EntityType::Note),
        other    => Err(anyhow::anyhow!("알 수 없는 entity_type: {other}")),
    }
}

#[derive(Args)]
pub struct HistoryArgs {
    #[command(subcommand)]
    pub command: HistoryCommand,
}

#[derive(Subcommand)]
pub enum HistoryCommand {
    /// 최근 변경 이력 — limit / since-minutes 필터.
    Recent {
        #[arg(long, default_value_t = 20)] limit: i64,
        #[arg(long = "since-minutes")] since_minutes: Option<i64>,
    },
    /// 특정 엔티티의 시간순 이력.
    For {
        #[arg(long = "entity-type")] entity_type: String,
        #[arg(long = "entity-id")] entity_id: i64,
    },
    /// 특정 에이전트의 최근 활동.
    ByAgent {
        #[arg(long = "agent-id")] agent_id: String,
        #[arg(long, default_value_t = 50)] limit: i64,
    },
}

pub async fn run(db: Db, args: HistoryArgs, fmt: OutputFormat) -> anyhow::Result<()> {
    match args.command {
        HistoryCommand::Recent { limit, since_minutes } => {
            output::print_value(
                &db.history_recent(limit, since_minutes).await?,
                fmt,
            )?;
        }
        HistoryCommand::For { entity_type, entity_id } => {
            let et = parse_entity_type(&entity_type)?;
            output::print_value(
                &db.history_list(et, entity_id).await?,
                fmt,
            )?;
        }
        HistoryCommand::ByAgent { agent_id, limit } => {
            output::print_value(
                &db.history_by_agent(&agent_id, limit).await?,
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
    struct Wrap { #[command(subcommand)] cmd: HistoryCommand }

    #[test]
    fn test_parse_recent_defaults() {
        let w = Wrap::try_parse_from(["x", "recent"]).unwrap();
        match w.cmd {
            HistoryCommand::Recent { limit, since_minutes } => {
                assert_eq!(limit, 20, "기본 limit=20");
                assert_eq!(since_minutes, None);
            }
            _ => panic!("Recent 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_recent_with_filters() {
        let w = Wrap::try_parse_from(
            ["x", "recent", "--limit", "20", "--since-minutes", "5"]
        ).unwrap();
        match w.cmd {
            HistoryCommand::Recent { limit, since_minutes } => {
                assert_eq!(limit, 20);
                assert_eq!(since_minutes, Some(5));
            }
            _ => panic!("Recent 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_for() {
        let w = Wrap::try_parse_from(
            ["x", "for", "--entity-type", "issue", "--entity-id", "12"]
        ).unwrap();
        match w.cmd {
            HistoryCommand::For { entity_type, entity_id } => {
                assert_eq!(entity_type, "issue");
                assert_eq!(entity_id, 12);
            }
            _ => panic!("For 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_by_agent_default_limit() {
        let w = Wrap::try_parse_from(
            ["x", "by-agent", "--agent-id", "leader@s"]
        ).unwrap();
        match w.cmd {
            HistoryCommand::ByAgent { agent_id, limit } => {
                assert_eq!(agent_id, "leader@s");
                assert_eq!(limit, 50, "기본 limit=50");
            }
            _ => panic!("ByAgent 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_entity_type_helper() {
        assert!(matches!(parse_entity_type("epic").unwrap(),   EntityType::Epic));
        assert!(matches!(parse_entity_type("sprint").unwrap(), EntityType::Sprint));
        assert!(parse_entity_type("bogus").is_err());
    }
}
