mod commands;
mod output;

use clap::{Parser, Subcommand};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "engram", version, about = "Agent Issue Management System")]
struct Cli {
    /// 머신 파싱용 JSON 출력 (이모지/배너 없는 단일 JSON object/array). ADR-0010 참조.
    #[arg(long, global = true)]
    json: bool,

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
    /// 스프린트 회고 리포트 생성
    Retro(commands::retro::RetroArgs),
    /// Claude Code Hook 설치/제거
    Hook(commands::hook::HookArgs),
    /// Hook에서 사용: 현재 세션 컨텍스트를 텍스트로 출력
    SnapshotText {
        #[arg(long)]
        project: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let fmt = OutputFormat::from_flags(cli.json);

    match run(cli, fmt).await {
        Ok(()) => {}
        Err(err) => {
            output::print_error(&err, fmt);
            std::process::exit(output::error_exit_code(&err));
        }
    }
}

async fn run(cli: Cli, fmt: OutputFormat) -> anyhow::Result<()> {
    let db = engram_core::Db::open_default().await?;

    match cli.command {
        Commands::Sprint(args)  => commands::sprint::run(db, args, fmt).await?,
        Commands::Epic(args)    => commands::epic::run(db, args, fmt).await?,
        Commands::Issue(args)   => commands::issue::run(db, args, fmt).await?,
        Commands::Task(args)    => commands::task::run(db, args, fmt).await?,
        Commands::Note(args)    => commands::note::run(db, args, fmt).await?,
        Commands::Session(args) => commands::session::run(db, args, fmt).await?,
        Commands::Retro(args)   => commands::retro::run(db, args, fmt).await?,
        Commands::Hook(args)    => commands::hook::run(args, fmt).await?,
        Commands::SnapshotText { project } => {
            commands::session::snapshot_text(db, project, fmt).await?;
        }
    }

    Ok(())
}
