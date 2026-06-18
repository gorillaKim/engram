use clap::{Args, Subcommand};
use engram_core::Db;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct BoardArgs {
    #[command(subcommand)]
    pub command: BoardCommand,
}

#[derive(Subcommand)]
pub enum BoardCommand {
    /// 현재 스프린트 전체 칸반 보드 — 프로젝트별 에픽/이슈 분포 + 블로킹 체인.
    Status {
        #[arg(long)] project: Option<String>,
        #[arg(long)] compact: bool,
        #[arg(long = "no-chains")] no_chains: bool,
    },
}

pub async fn run(db: Db, args: BoardArgs, fmt: OutputFormat, mode: engram_core::models::OutputMode) -> anyhow::Result<()> {
    match args.command {
        BoardCommand::Status { project, compact, no_chains } => {
            let actual_mode = if compact {
                engram_core::models::OutputMode::Compact
            } else {
                mode
            };
            let res = db.board_status_mode(project.as_deref(), actual_mode, !no_chains).await?;
            output::print_core_response(res, fmt)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap { #[command(subcommand)] cmd: BoardCommand }

    #[test]
    fn test_parse_status_with_project() {
        let w = Wrap::try_parse_from(["x", "status", "--project", "engram"]).unwrap();
        match w.cmd {
            BoardCommand::Status { project, .. } => assert_eq!(project.as_deref(), Some("engram")),
        }
    }

    #[test]
    fn test_parse_status_no_project() {
        let w = Wrap::try_parse_from(["x", "status"]).unwrap();
        match w.cmd {
            BoardCommand::Status { project, .. } => assert_eq!(project, None),
        }
    }

    #[test]
    fn test_parse_status_with_flags() {
        let w = Wrap::try_parse_from(["x", "status", "--compact", "--no-chains"]).unwrap();
        match w.cmd {
            BoardCommand::Status { compact, no_chains, .. } => {
                assert!(compact);
                assert!(no_chains);
            }
        }
    }
}
