use clap::{Args, Subcommand};

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
}

pub async fn run(args: HookArgs) -> anyhow::Result<()> {
    match args.command {
        HookCommand::Install   => install().await,
        HookCommand::Uninstall => uninstall().await,
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
