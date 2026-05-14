mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "engram", version, about = "Agent Issue Management System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 스프린트 관리
    Sprint(commands::sprint::SprintArgs),
    /// 에픽 관리
    Epic(commands::epic::EpicArgs),
    /// 이슈 관리
    Issue(commands::issue::IssueArgs),
    /// 태스크 관리
    Task(commands::task::TaskArgs),
    /// 노트 관리
    Note(commands::note::NoteArgs),
    /// 세션 관리
    Session(commands::session::SessionArgs),
    /// Claude Code Hook 설치/제거
    Hook(commands::hook::HookArgs),
    /// Hook에서 사용: 현재 세션 컨텍스트를 텍스트로 출력
    SnapshotText {
        #[arg(long)]
        project: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let db = engram_core::Db::open_default().await?;

    match cli.command {
        Commands::Sprint(args)  => commands::sprint::run(db, args).await?,
        Commands::Epic(args)    => commands::epic::run(db, args).await?,
        Commands::Issue(args)   => commands::issue::run(db, args).await?,
        Commands::Task(args)    => commands::task::run(db, args).await?,
        Commands::Note(args)    => commands::note::run(db, args).await?,
        Commands::Session(args) => commands::session::run(db, args).await?,
        Commands::Hook(args)    => commands::hook::run(args).await?,
        Commands::SnapshotText { project } => {
            let snapshot = db.session_restore(project.as_deref()).await?;
            println!("=== ENGRAM SESSION CONTEXT ===");
            if let Some(next) = &snapshot.next_action {
                println!("📋 다음 태스크: {} ({})", next.task_title, next.project_key);
                println!("   이슈: {}", next.issue_title);
                println!("   에픽: {}", next.epic_title);
            }
            for epic in &snapshot.active_epics {
                for issue in &epic.active_issues {
                    for note in &issue.active_notes {
                        if note.note_type == engram_core::models::NoteType::Caveat {
                            println!("⚠️  [caveat] {}", note.summary);
                        }
                    }
                }
            }
            for w in &snapshot.warnings {
                println!("⏳ {w}");
            }
            println!("==============================");
        }
    }

    Ok(())
}
