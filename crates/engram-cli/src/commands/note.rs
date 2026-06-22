use clap::{Args, Subcommand};
use engram_core::{Db, models::note::{CreateNoteInput, NoteScope, NoteType}};
use crate::output::{self, OutputFormat};

fn parse_note_type(s: &str) -> NoteType {
    match s {
        "caveat"         => NoteType::Caveat,
        "decision"       => NoteType::Decision,
        "discovery"      => NoteType::Discovery,
        "blocker_detail" => NoteType::BlockerDetail,
        "reference"      => NoteType::Reference,
        "comment"        => NoteType::Comment,
        "evaluation"     => NoteType::Evaluation,
        _                => NoteType::Context,
    }
}

fn parse_note_type_filter(s: &str) -> anyhow::Result<NoteType> {
    match s {
        "caveat"         => Ok(NoteType::Caveat),
        "decision"       => Ok(NoteType::Decision),
        "discovery"      => Ok(NoteType::Discovery),
        "blocker_detail" => Ok(NoteType::BlockerDetail),
        "context"        => Ok(NoteType::Context),
        "reference"      => Ok(NoteType::Reference),
        "comment"        => Ok(NoteType::Comment),
        "evaluation"     => Ok(NoteType::Evaluation),
        other            => Err(anyhow::anyhow!("알 수 없는 note_type: {other}")),
    }
}

fn parse_scope(s: &str) -> anyhow::Result<NoteScope> {
    match s {
        "issue"   => Ok(NoteScope::Issue),
        "task"    => Ok(NoteScope::Task),
        "project" => Ok(NoteScope::Project),
        "sprint"  => Ok(NoteScope::Sprint),
        "epic"    => Ok(NoteScope::Epic),
        other     => Err(anyhow::anyhow!("알 수 없는 scope: {other}")),
    }
}

#[derive(Args)]
pub struct NoteArgs {
    #[command(subcommand)]
    pub command: NoteCommand,
}

#[derive(Subcommand)]
pub enum NoteCommand {
    /// 노트 추가. scope=issue/task 면 --issue 또는 --task 필수.
    /// broadcast (project/sprint/epic) 는 --scope-target-id (또는 --project-key) 필수.
    Add {
        /// scope='issue' 또는 자동판정용 issue_id. broadcast scope 면 0.
        #[arg(long, default_value_t = 0)] issue: i64,
        #[arg(long)] task: Option<i64>,
        #[arg(long, value_name = "TYPE")] r#type: String,
        #[arg(long)] summary: String,
        #[arg(long)] detail: Option<String>,
        #[arg(long)] scope: Option<String>,
        #[arg(long = "scope-target-id")] scope_target_id: Option<i64>,
        #[arg(long = "project-key")] project_key: Option<String>,
        #[arg(long)] author: Option<String>,
        #[arg(long = "agent-id")] agent_id: Option<String>,
    },
    /// 노트 목록 조회. --type 필터 + --include-resolved 옵션 지원.
    List {
        #[arg(long)] issue: Option<i64>,
        #[arg(long)] task: Option<i64>,
        #[arg(long, value_name = "TYPE")] r#type: Option<String>,
        #[arg(long = "types", value_delimiter = ',')] r#types: Option<Vec<String>>,
        #[arg(long = "include-resolved")] include_resolved: bool,
        #[arg(long = "include-detail")] include_detail: bool,
        #[arg(long = "project-key")] project_key: Option<String>,
        #[arg(long = "sprint-id")] sprint_id: Option<i64>,
        #[arg(long)] epic: Option<i64>,
        #[arg(long)] rollup: bool,
        #[arg(long)] limit: Option<i64>,
        #[arg(long)] offset: Option<i64>,
        #[arg(long)] compact: bool,
        #[arg(long, value_delimiter = ',')] projection: Option<Vec<String>>,
        #[arg(long = "updated-after")] updated_after: Option<String>,
    },
    Resolve { id: i64 },
    /// 노트 상세 조회 (detail 포함)
    Get {
        /// 조회할 노트 ID 목록 (쉼표 구분 또는 복수 지정)
        #[arg(value_delimiter = ',')]
        ids: Vec<i64>,
        #[arg(long)] compact: bool,
    },
}

pub async fn run(db: Db, args: NoteArgs, fmt: OutputFormat, global_agent_id: &str, mode: engram_core::models::OutputMode) -> anyhow::Result<()> {
    match args.command {
        NoteCommand::Add {
            issue, task, r#type, summary, detail,
            scope, scope_target_id, project_key, author, agent_id,
        } => {
            let note = db.note_add(CreateNoteInput {
                issue_id: issue,
                task_id: task,
                note_type: parse_note_type(&r#type),
                summary: crate::commands::unescape_newlines(summary),
                detail: crate::commands::unescape_newlines_opt(detail),
                author,
                agent_id: agent_id.or_else(|| Some(global_agent_id.to_string())),
                scope: scope.as_deref().map(parse_scope).transpose()?,
                scope_target_id,
                project_key,
            }).await?;
            output::print_value(&note, fmt)?;
        }
        NoteCommand::List {
            issue, task, r#type, r#types, include_resolved, include_detail,
            project_key, sprint_id, epic, rollup, limit, offset, compact: _, projection, updated_after,
        } => {
            let nt = r#type.as_deref().map(parse_note_type_filter).transpose()?;
            let nts = if let Some(ref list) = r#types {
                let mut parsed = Vec::new();
                for s in list {
                    parsed.push(parse_note_type_filter(s)?);
                }
                Some(parsed)
            } else {
                None
            };
            let res = db.note_list_mode(
                issue,
                task,
                nt,
                nts,
                include_resolved,
                include_detail,
                project_key.as_deref(),
                sprint_id,
                epic,
                Some(rollup),
                limit,
                offset,
                mode,
                updated_after,
            ).await?;
 
            if let Some(ref fields) = projection {
                let val = serde_json::to_value(&res).unwrap();
                let filtered = engram_core::apply_projection(val, fields);
                output::print_value(&filtered, fmt)?;
            } else {
                output::print_core_response(res, fmt)?;
            }
        }
        NoteCommand::Resolve { id } => {
            db.note_resolve(id, global_agent_id).await?;
            output::print_value(
                &serde_json::json!({ "ok": true, "resolved_id": id }),
                fmt,
            )?;
        }
        NoteCommand::Get { ids, compact } => {
            if ids.is_empty() {
                anyhow::bail!("조회할 노트 ID를 지정해야 합니다.");
            }
            if ids.len() == 1 {
                output::print_value(&db.note_get(ids[0], compact).await?, fmt)?;
            } else {
                let res = db.note_get_batch(&ids, compact, mode).await?;
                output::print_core_response(res, fmt)?;
            }
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
        let w = Wrap::try_parse_from(["x", "get", "42,43"]).unwrap();
        match w.cmd {
            NoteCommand::Get { ids, .. } => assert_eq!(ids, vec![42, 43]),
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

    #[test]
    fn test_parse_add_broadcast_epic_scope() {
        let w = Wrap::try_parse_from([
            "x", "add", "--type", "decision", "--summary", "ADR-X",
            "--scope", "epic", "--scope-target-id", "4",
            "--agent-id", "leader@s",
        ]).unwrap();
        match w.cmd {
            NoteCommand::Add { issue, scope, scope_target_id, agent_id, .. } => {
                assert_eq!(issue, 0, "broadcast scope 면 --issue 생략 시 0");
                assert_eq!(scope.as_deref(), Some("epic"));
                assert_eq!(scope_target_id, Some(4));
                assert_eq!(agent_id.as_deref(), Some("leader@s"));
            }
            _ => panic!("Add 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_list_with_filters() {
        let w = Wrap::try_parse_from([
            "x", "list", "--issue", "5", "--type", "caveat", "--include-resolved",
        ]).unwrap();
        match w.cmd {
            NoteCommand::List { issue, r#type, include_resolved, .. } => {
                assert_eq!(issue, Some(5));
                assert_eq!(r#type.as_deref(), Some("caveat"));
                assert!(include_resolved);
            }
            _ => panic!("List 변형이 파싱되어야 함"),
        }
    }

    #[test]
    fn test_parse_note_type_filter_helper() {
        assert!(matches!(parse_note_type_filter("comment").unwrap(), NoteType::Comment));
        assert!(matches!(parse_note_type_filter("context").unwrap(), NoteType::Context));
        assert!(parse_note_type_filter("bogus").is_err());
    }

    #[test]
    fn test_parse_scope_helper() {
        assert!(matches!(parse_scope("project").unwrap(), NoteScope::Project));
        assert!(matches!(parse_scope("epic").unwrap(),    NoteScope::Epic));
        assert!(parse_scope("bogus").is_err());
    }
}
