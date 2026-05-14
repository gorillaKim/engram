use clap::{Args, Subcommand};
use engram_core::{Db, models::note::{CreateNoteInput, NoteType}};

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
}

pub async fn run(db: Db, args: NoteArgs) -> anyhow::Result<()> {
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
                issue_id: issue, task_id: None, note_type, summary, detail, author: None,
            }).await?;
            println!("{}", serde_json::to_string_pretty(&note)?);
        }
        NoteCommand::List { issue } => {
            println!("{}", serde_json::to_string_pretty(
                &db.note_list(Some(issue), None, None, false).await?
            )?);
        }
        NoteCommand::Resolve { id } => {
            db.note_resolve(id).await?;
            println!("✅ 노트 해결됨: #{id}");
        }
    }
    Ok(())
}
