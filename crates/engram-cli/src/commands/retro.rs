use clap::Args;
use engram_core::Db;

#[derive(Args)]
pub struct RetroArgs {
    /// 회고할 스프린트 ID (미입력 시 현재 스프린트)
    #[arg(long)]
    pub sprint: Option<i64>,
}

pub async fn run(db: Db, args: RetroArgs) -> anyhow::Result<()> {
    let sprint_id = match args.sprint {
        Some(id) => id,
        None => {
            let current = db.sprint_current().await?;
            current.ok_or_else(|| anyhow::anyhow!("활성 스프린트가 없습니다"))?.id
        }
    };

    let report = db.retro_report(sprint_id).await?;

    println!("# Sprint {} — 회고 리포트", report.sprint_name);
    println!();
    println!("## 요약");
    println!("- 전체 이슈: {}건", report.total_issues);
    println!("- 완료 이슈: {}건", report.finished_issues);
    println!();

    if !report.scope_expansions.is_empty() {
        println!("## 스코프 팽창");
        for s in &report.scope_expansions {
            println!("- **{}**: planned {}건 + discovered {}건 (팽창률 {}%)",
                s.title, s.planned, s.discovered, s.expansion_rate);
        }
        println!();
    }

    if !report.issue_timelines.is_empty() {
        println!("## 이슈 상태 타임라인");
        for tl in &report.issue_timelines {
            println!("### {} (id={})", tl.title, tl.issue_id);
            for t in &tl.transitions {
                println!("- `{}`: {:?} → {:?} ({})",
                    t.field, t.old_value, t.new_value, t.changed_at);
            }
        }
    }

    Ok(())
}
