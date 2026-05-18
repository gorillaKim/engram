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
    },
}

pub async fn run(db: Db, args: BoardArgs, fmt: OutputFormat) -> anyhow::Result<()> {
    match args.command {
        BoardCommand::Status { project } => {
            let board = db.board_status_query(project.as_deref()).await?;
            output::print_value(&board, fmt)?;
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
            BoardCommand::Status { project } => assert_eq!(project.as_deref(), Some("engram")),
        }
    }

    #[test]
    fn test_parse_status_no_project() {
        let w = Wrap::try_parse_from(["x", "status"]).unwrap();
        match w.cmd {
            BoardCommand::Status { project } => assert_eq!(project, None),
        }
    }
}
