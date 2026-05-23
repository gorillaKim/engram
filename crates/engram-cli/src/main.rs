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

    /// 호출 액터 식별자 (예: 'gemini-cli', 'user'). ADR-0010 참조.
    #[arg(long, global = true)]
    agent_id: Option<String>,

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
    /// 칸반 보드 현황
    Board(commands::board::BoardArgs),
    /// 블로킹 의존성 그래프
    Blocked(commands::blocked::BlockedArgs),
    /// 정체된 이슈 — threshold-minutes 이상 머문 이슈
    Stalled(commands::stalled::StalledArgs),
    /// 변경 이력 — recent / for / by-agent
    History(commands::history::HistoryArgs),
    /// 미션 관리 (Sprint→Mission→Epic 계층, ADR-0012)
    Mission(commands::mission::MissionArgs),
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
    let agent_id = cli.agent_id.as_deref().unwrap_or("user");

    match cli.command {
        Commands::Sprint(args)  => commands::sprint::run(db, args, fmt, agent_id).await?,
        Commands::Epic(args)    => commands::epic::run(db, args, fmt, agent_id).await?,
        Commands::Issue(args)   => commands::issue::run(db, args, fmt, agent_id).await?,
        Commands::Task(args)    => commands::task::run(db, args, fmt, agent_id).await?,
        Commands::Note(args)    => commands::note::run(db, args, fmt, agent_id).await?,
        Commands::Session(args) => commands::session::run(db, args, fmt).await?,
        Commands::Board(args)   => commands::board::run(db, args, fmt).await?,
        Commands::Blocked(args) => commands::blocked::run(db, args, fmt).await?,
        Commands::Stalled(args) => commands::stalled::run(db, args, fmt).await?,
        Commands::History(args) => commands::history::run(db, args, fmt).await?,
        Commands::Mission(args) => commands::mission::run(db, args, fmt, agent_id).await?,
        Commands::Retro(args)   => commands::retro::run(db, args, fmt).await?,
        Commands::Hook(args)    => commands::hook::run(args, fmt).await?,
        Commands::SnapshotText { project } => {
            commands::session::snapshot_text(db, project, fmt).await?;
        }
    }

    Ok(())
}
