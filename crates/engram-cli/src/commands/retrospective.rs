use clap::{Args, Subcommand};
use engram_core::{
    models::retrospective::*,
    Db, Result,
};

#[derive(Args)]
pub struct RetrospectiveArgs {
    #[command(subcommand)]
    pub command: RetrospectiveSubcommands,
}

#[derive(Subcommand)]
pub enum RetrospectiveSubcommands {
    /// 회고 문서 작성
    Create {
        #[arg(long)]
        project: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
        #[arg(long)]
        sprint_id: Option<i64>,
        #[arg(long)]
        mission_id: Option<i64>,
        #[arg(long)]
        epic_id: Option<i64>,
    },
    /// 회고 목록 조회
    List {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        sprint_id: Option<i64>,
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// 특정 회고 상세 조회
    Get {
        #[arg(long)]
        id: i64,
    },
    /// 회고 내용 수정
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    /// 회고 삭제
    Delete {
        #[arg(long)]
        id: i64,
    },
    /// 액션 아이템 관리
    ActionItem {
        #[command(subcommand)]
        command: ActionItemSubcommands,
    },
}

#[derive(Subcommand)]
pub enum ActionItemSubcommands {
    /// 액션 아이템 추가
    Add {
        #[arg(long)]
        retro_id: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        description: Option<String>,
    },
    /// 액션 아이템 수정
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// 액션 아이템을 이슈로 변환 (단일 또는 전체)
    Convert {
        #[arg(long)]
        id: Option<i64>,
        #[arg(long)]
        retro_id: Option<i64>,
    },
}

pub async fn run(db: &Db, args: RetrospectiveArgs, agent_id: Option<&str>) -> Result<serde_json::Value> {
    match args.command {
        RetrospectiveSubcommands::Create {
            project,
            title,
            content,
            sprint_id,
            mission_id,
            epic_id,
        } => {
            let input = CreateRetrospectiveInput {
                project_key: project,
                title,
                content,
                sprint_id,
                mission_id,
                epic_id,
                agent_id: agent_id.map(|s| s.to_string()),
                action_items: None,
            };
            let res = db.retrospective_create(input).await?;
            Ok(serde_json::to_value(res).unwrap())
        }
        RetrospectiveSubcommands::List {
            project,
            sprint_id,
            limit,
        } => {
            let res = db.retrospective_list(project.as_deref(), sprint_id, limit).await?;
            Ok(serde_json::to_value(res).unwrap())
        }
        RetrospectiveSubcommands::Get { id } => {
            let res = db.retrospective_get(id).await?;
            Ok(serde_json::to_value(res).unwrap())
        }
        RetrospectiveSubcommands::Update { id, title, content } => {
            let input = UpdateRetrospectiveInput {
                title,
                content,
                ..Default::default()
            };
            let res = db.retrospective_update(id, input, agent_id).await?;
            Ok(serde_json::to_value(res).unwrap())
        }
        RetrospectiveSubcommands::Delete { id } => {
            db.retrospective_delete(id).await?;
            Ok(serde_json::json!({ "success": true, "deleted_id": id }))
        }
        RetrospectiveSubcommands::ActionItem { command } => match command {
            ActionItemSubcommands::Add {
                retro_id,
                title,
                description,
            } => {
                let input = CreateRetroActionItemInput {
                    title,
                    description,
                    linked_issue_id: None,
                    linked_note_id: None,
                    ord: None,
                };
                let res = db.retro_action_item_create(retro_id, input).await?;
                Ok(serde_json::to_value(res).unwrap())
            }
            ActionItemSubcommands::Update { id, title, status } => {
                let input = UpdateRetroActionItemInput {
                    title,
                    status,
                    ..Default::default()
                };
                let res = db.retro_action_item_update(id, input).await?;
                Ok(serde_json::to_value(res).unwrap())
            }
            ActionItemSubcommands::Convert { id, retro_id } => {
                if let Some(item_id) = id {
                    let issue = db.retro_action_item_convert_to_issue(item_id, agent_id).await?;
                    Ok(serde_json::to_value(issue).unwrap())
                } else if let Some(r_id) = retro_id {
                    let issues = db.retrospective_bulk_convert_actions_to_issues(r_id, agent_id).await?;
                    Ok(serde_json::to_value(issues).unwrap())
                } else {
                    Err(engram_core::Error::Validation("Either --id or --retro-id is required for convert".into()))
                }
            }
        },
    }
}
