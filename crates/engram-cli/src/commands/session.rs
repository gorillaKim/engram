use clap::{Args, Subcommand};
use engram_core::Db;
use crate::output::{self, OutputFormat};

#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub command: SessionCommand,
}

#[derive(Subcommand)]
pub enum SessionCommand {
    /// 세션 복원 — 현재 스프린트 상태와 다음 태스크를 JSON으로 출력
    Restore {
        #[arg(long)] project: Option<String>,
        /// 노트/태스크를 count만 반환해 페이로드를 70%+ 줄인다
        #[arg(long)] compact: bool,
    },
    /// 세션 종료 체크리스트 — context note 누락 시 경고
    End {
        #[arg(long)] project: Option<String>,
    },
}

pub async fn run(db: Db, args: SessionArgs, fmt: OutputFormat) -> anyhow::Result<()> {
    match args.command {
        SessionCommand::Restore { project, compact } => {
            let snapshot = db.session_restore(project.as_deref(), compact).await?;
            output::print_value(&snapshot, fmt)?;
        }
        SessionCommand::End { project } => {
            let result = db.session_end(project.as_deref()).await?;
            // ADR-0010: --json 모드는 raw payload, --pretty 모드는 부가 안내 텍스트도 stdout 으로.
            output::print_value(&result, fmt)?;
            if matches!(fmt, OutputFormat::Pretty) {
                if result.ok {
                    output::print_human("✅ 세션 종료 체크리스트 통과", fmt);
                } else {
                    output::print_human("⚠️  세션 종료 전 처리 필요:", fmt);
                    for w in &result.warnings {
                        output::print_human(&format!("   - {w}"), fmt);
                    }
                }
            }
        }
    }
    Ok(())
}

/// `engram snapshot-text` — Hook 에서 호출되는 인간용 텍스트 출력.
/// `--json` 모드에서는 session_restore 의 raw JSON 을 그대로 emit.
pub async fn snapshot_text(
    db: Db,
    project: Option<String>,
    fmt: OutputFormat,
) -> anyhow::Result<()> {
    let snapshot = db.session_restore(project.as_deref(), false).await?;

    if matches!(fmt, OutputFormat::Json) {
        output::print_value(&snapshot, fmt)?;
        return Ok(());
    }

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
    Ok(())
}
