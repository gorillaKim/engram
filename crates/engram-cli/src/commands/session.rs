use clap::{Args, Subcommand};
use engram_core::Db;

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
    },
    /// 세션 종료 체크리스트 — context note 누락 시 경고
    End {
        #[arg(long)] project: Option<String>,
    },
}

pub async fn run(db: Db, args: SessionArgs) -> anyhow::Result<()> {
    match args.command {
        SessionCommand::Restore { project } => {
            let snapshot = db.session_restore(project.as_deref()).await?;
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
        }
        SessionCommand::End { project } => {
            let result = db.session_end(project.as_deref()).await?;
            if result.ok {
                println!("✅ 세션 종료 체크리스트 통과");
            } else {
                println!("⚠️  세션 종료 전 처리 필요:");
                for w in &result.warnings {
                    println!("   - {w}");
                }
            }
        }
    }
    Ok(())
}
