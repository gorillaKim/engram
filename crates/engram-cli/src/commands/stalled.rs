use clap::Args;
use engram_core::{Db, models::issue::IssueStatus};
use crate::output::{self, OutputFormat};

fn parse_stalled_status(s: &str) -> anyhow::Result<IssueStatus> {
    match s {
        "required"  => Ok(IssueStatus::Required),
        "ready"     => Ok(IssueStatus::Ready),
        "working"   => Ok(IssueStatus::Working),
        "demo"      => Ok(IssueStatus::Demo),
        "finished"  => Ok(IssueStatus::Finished),
        "cancelled" => Ok(IssueStatus::Cancelled),
        other       => Err(anyhow::anyhow!("알 수 없는 status: {other}")),
    }
}

/// MCP `stalled_issues` 와 1:1 매핑. 기본 status=working, threshold_minutes 필수.
#[derive(Args)]
pub struct StalledArgs {
    #[arg(long = "threshold-minutes")] pub threshold_minutes: i64,
    #[arg(long, default_value = "working")] pub status: String,
    #[arg(long)] pub project: Option<String>,
}

pub async fn run(db: Db, args: StalledArgs, fmt: OutputFormat, mode: engram_core::models::OutputMode) -> anyhow::Result<()> {
    let st = parse_stalled_status(&args.status)?;
    let res = db.stalled_issues_mode(args.project.as_deref(), st, args.threshold_minutes, mode).await?;
    output::print_core_response(res, fmt)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap { #[command(flatten)] cmd: StalledArgs }

    #[test]
    fn test_parse_threshold_required() {
        let w = Wrap::try_parse_from(["x", "--threshold-minutes", "10"]).unwrap();
        assert_eq!(w.cmd.threshold_minutes, 10);
        assert_eq!(w.cmd.status, "working", "기본 status=working");
        assert_eq!(w.cmd.project, None);
    }

    #[test]
    fn test_parse_with_project_and_status() {
        let w = Wrap::try_parse_from([
            "x", "--threshold-minutes", "30", "--status", "ready", "--project", "engram",
        ]).unwrap();
        assert_eq!(w.cmd.threshold_minutes, 30);
        assert_eq!(w.cmd.status, "ready");
        assert_eq!(w.cmd.project.as_deref(), Some("engram"));
    }

    #[test]
    fn test_parse_stalled_status_helper() {
        assert_eq!(parse_stalled_status("working").unwrap(), IssueStatus::Working);
        assert!(parse_stalled_status("bogus").is_err());
    }
}
