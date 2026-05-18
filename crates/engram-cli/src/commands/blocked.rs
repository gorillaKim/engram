use clap::{Args, Subcommand};
use engram_core::Db;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct BlockedArgs {
    #[command(subcommand)]
    pub command: BlockedCommand,
}

#[derive(Subcommand)]
pub enum BlockedCommand {
    /// 현재 프로젝트의 블로킹 의존성 그래프 — 해소 가능한 leaf blocker + 체인 경로.
    List {
        #[arg(long)] project: String,
    },
}

pub async fn run(db: Db, args: BlockedArgs, fmt: OutputFormat) -> anyhow::Result<()> {
    match args.command {
        BlockedCommand::List { project } => {
            let graph = db.blocked_issues_graph(&project).await?;
            output::print_value(&graph, fmt)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap { #[command(subcommand)] cmd: BlockedCommand }

    #[test]
    fn test_parse_list_requires_project() {
        let w = Wrap::try_parse_from(["x", "list", "--project", "engram"]).unwrap();
        match w.cmd {
            BlockedCommand::List { project } => assert_eq!(project, "engram"),
        }
    }

    #[test]
    fn test_parse_list_missing_project_fails() {
        assert!(Wrap::try_parse_from(["x", "list"]).is_err());
    }
}
