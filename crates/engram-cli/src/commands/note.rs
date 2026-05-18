use clap::{Args, Subcommand};
use engram_core::{Db, models::note::{CreateNoteInput, NoteType}};
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct NoteArgs {
    #[command(subcommand)]
    pub command: NoteCommand,
}

#[derive(Subcommand)]
pub enum NoteCommand {
    Add {
        #[arg(long)] issue: i64,
        #[arg(long, value_name = "TYPE")] r#type: String,
        #[arg(long)] summary: String,
        #[arg(long)] detail: Option<String>,
    },
    List { #[arg(long)] issue: i64 },
    Resolve { id: i64 },
    /// 노트 상세 조회 (detail 포함)
    Get { id: i64 },
}

pub async fn run(db: Db, args: NoteArgs, fmt: OutputFormat) -> anyhow::Result<()> {
    match args.command {
        NoteCommand::Add { issue, r#type, summary, detail } => {
            let note_type = match r#type.as_str() {
                "caveat"         => NoteType::Caveat,
                "decision"       => NoteType::Decision,
                "discovery"      => NoteType::Discovery,
                "blocker_detail" => NoteType::BlockerDetail,
                "reference"      => NoteType::Reference,
                _                => NoteType::Context,
            };
            let note = db.note_add(CreateNoteInput {
                issue_id: issue, task_id: None, note_type, summary, detail, author: None, agent_id: None,
                scope: None, scope_target_id: None, project_key: None,
            }).await?;
            output::print_value(&note, fmt)?;
        }
        NoteCommand::List { issue } => {
            output::print_value(
                &db.note_list(Some(issue), None, None, false).await?,
                fmt,
            )?;
        }
        NoteCommand::Resolve { id } => {
            db.note_resolve(id, "user").await?;
            output::print_value(
                &serde_json::json!({ "ok": true, "resolved_id": id }),
                fmt,
            )?;
        }
        NoteCommand::Get { id } => {
            output::print_value(&db.note_get(id).await?, fmt)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap { #[command(subcommand)] cmd: NoteCommand }

    #[test]
    fn test_parse_get() {
        let w = Wrap::try_parse_from(["x", "get", "42"]).unwrap();
        match w.cmd {
            NoteCommand::Get { id } => assert_eq!(id, 42),
            _ => panic!("Get 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_add_with_type() {
        let w = Wrap::try_parse_from(
            ["x", "add", "--issue", "1", "--type", "caveat", "--summary", "주의"]
        ).unwrap();
        match w.cmd {
            NoteCommand::Add { issue, r#type, summary, .. } => {
                assert_eq!(issue, 1);
                assert_eq!(r#type, "caveat");
                assert_eq!(summary, "주의");
            }
            _ => panic!("Add 변형이 파싱되어야 함"),
        }
    }
}
