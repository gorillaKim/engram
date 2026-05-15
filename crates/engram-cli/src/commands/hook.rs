use clap::{Args, Subcommand};
use engram_core::Db;

#[derive(Args)]
pub struct HookArgs {
    #[command(subcommand)]
    pub command: HookCommand,
}

#[derive(Subcommand)]
pub enum HookCommand {
    /// Claude Code settings.json에 Engram Hook 등록
    Install,
    /// Hook 제거
    Uninstall,
    /// 세션 종료 시 체크리스트 출력 (Stop 훅에서 호출)
    PostSessionCheck {
        #[arg(long)]
        project: Option<String>,
    },
}

pub async fn run(args: HookArgs) -> anyhow::Result<()> {
    match args.command {
        HookCommand::Install                     => install().await,
        HookCommand::Uninstall                   => uninstall().await,
        HookCommand::PostSessionCheck { project } => post_session_check(project).await,
    }
}

async fn install() -> anyhow::Result<()> {
    let home = std::env::var("HOME")?;
    let settings_path = format!("{home}/.claude/settings.json");

    // settings.json 읽기 (없으면 빈 객체)
    let content = tokio::fs::read_to_string(&settings_path)
        .await
        .unwrap_or_else(|_| "{}".to_string());

    let mut settings: serde_json::Value = serde_json::from_str(&content)?;

    // hooks 섹션 추가
    let hooks = serde_json::json!({
        "PreToolUse": [{
            "matcher": "Bash",
            "hooks": [{
                "type": "command",
                "command": "engram snapshot-text"
            }]
        }],
        "Stop": [{
            "hooks": [{
                "type": "command",
                "command": "engram hook post-session-check"
            }]
        }]
    });

    settings["hooks"] = hooks;

    tokio::fs::write(&settings_path, serde_json::to_string_pretty(&settings)?).await?;
    println!("✅ Engram Hook이 {settings_path}에 등록되었습니다.");
    println!("   각 프로젝트 CLAUDE.md에 아래 내용을 추가하세요:");
    println!();
    println!("   ## Engram");
    println!("   project_key: your-project-name");
    Ok(())
}

async fn uninstall() -> anyhow::Result<()> {
    let home = std::env::var("HOME")?;
    let settings_path = format!("{home}/.claude/settings.json");

    let content = tokio::fs::read_to_string(&settings_path).await?;
    let mut settings: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = settings.as_object_mut() {
        obj.remove("hooks");
    }

    tokio::fs::write(&settings_path, serde_json::to_string_pretty(&settings)?).await?;
    println!("✅ Engram Hook이 제거되었습니다.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap { #[command(subcommand)] cmd: HookCommand }

    #[test]
    fn test_parse_install_uninstall() {
        assert!(matches!(
            Wrap::try_parse_from(["x", "install"]).unwrap().cmd,
            HookCommand::Install
        ));
        assert!(matches!(
            Wrap::try_parse_from(["x", "uninstall"]).unwrap().cmd,
            HookCommand::Uninstall
        ));
    }

    #[test]
    fn test_parse_post_session_check_with_project() {
        let w = Wrap::try_parse_from(["x", "post-session-check", "--project", "p1"]).unwrap();
        match w.cmd {
            HookCommand::PostSessionCheck { project } => {
                assert_eq!(project.as_deref(), Some("p1"));
            }
            _ => panic!("PostSessionCheck 변형이 파싱되어야 함"),
        }
    }
}

async fn post_session_check(project: Option<String>) -> anyhow::Result<()> {
    let db = Db::open_default().await?;
    let result = db.session_end(project.as_deref()).await?;
    if result.warnings.is_empty() && result.in_progress_tasks.is_empty() {
        println!("✅ 세션 종료: 미완료 항목 없음");
    } else {
        for w in &result.warnings {
            println!("⚠️  {w}");
        }
        for t in &result.in_progress_tasks {
            println!("⏳ {} — {}", t.issue_title, t.title);
        }
    }
    Ok(())
}
