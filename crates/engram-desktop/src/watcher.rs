use engram_core::Db;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TrayBoardSummary {
    pub inbox: u32,       // required 전체
    pub demo_review: u32, // demo 전체
    pub blockers: u32,    // blocker 전체
}

#[derive(Default)]
struct BoardSnapshot {
    required: HashMap<i64, String>, // id → title
    demo: HashMap<i64, String>,
    blockers: u32,
}

pub async fn run(app: AppHandle, db: Arc<Db>) {
    let mut last = BoardSnapshot::default();
    let mut cooldown_map: HashMap<String, Instant> = HashMap::new();
    let cooldown = Duration::from_secs(30);
    let mut tick: u32 = 0;

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        tick = tick.wrapping_add(1);

        let cur = match snapshot(&db).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("watcher snapshot error: {e}");
                continue;
            }
        };

        // Emit summary for popover
        let summary = TrayBoardSummary {
            inbox: cur.required.len() as u32,
            demo_review: cur.demo.len() as u32,
            blockers: cur.blockers,
        };
        let _ = app.emit("tray://summary", &summary);

        // Update tray title
        if let Some(tray) = app.tray_by_id("default") {
            let title = format!("📦 {} · ⚠ {}", summary.inbox, summary.demo_review);
            let _ = tray.set_title(Some(&title));
        }

        // Detect new_required
        for (id, title) in &cur.required {
            if !last.required.contains_key(id) {
                let key = format!("req:{id}");
                if should_notify(&cooldown_map, &key, cooldown) {
                    send_notification(&app, "🆕 새 이슈 승인 대기", &format!("#{id} {title}"));
                    let _ = app.emit("tray://new_required", serde_json::json!({ "id": id, "title": title }));
                    cooldown_map.insert(key, Instant::now());
                }
            }
        }

        // Detect entered_demo
        for (id, title) in &cur.demo {
            if !last.demo.contains_key(id) {
                let key = format!("demo:{id}");
                if should_notify(&cooldown_map, &key, cooldown) {
                    send_notification(&app, "👀 검토 대기", &format!("#{id} {title}"));
                    let _ = app.emit("tray://entered_demo", serde_json::json!({ "id": id, "title": title }));
                    cooldown_map.insert(key, Instant::now());
                }
            }
        }

        // Detect new blockers (count-based)
        if cur.blockers > last.blockers {
            let key = "blocker:count".to_string();
            if should_notify(&cooldown_map, &key, cooldown) {
                let body = format!("{} 개 이슈가 블로킹됨", cur.blockers);
                send_notification(&app, "🚫 새 블로커 발생", &body);
                let _ = app.emit("tray://new_blocker", serde_json::json!({ "count": cur.blockers }));
                cooldown_map.insert(key, Instant::now());
            }
        }

        last = cur;

        // Periodic eviction: every ~1 hour (720 ticks × 5s) remove expired cooldown entries
        if tick % 720 == 0 {
            cooldown_map.retain(|_, t| t.elapsed() < cooldown);
        }
    }
}

pub fn should_notify(map: &HashMap<String, Instant>, key: &str, cooldown: Duration) -> bool {
    map.get(key).map_or(true, |t| t.elapsed() > cooldown)
}

fn send_notification(app: &AppHandle, title: &str, body: &str) {
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        tracing::warn!("notification error: {e}");
    }
}

async fn snapshot(db: &Db) -> engram_core::Result<BoardSnapshot> {
    let board = db.board_issues_query(None).await?;
    let status = db.board_status_query(None).await?;

    let mut required = HashMap::new();
    let mut demo = HashMap::new();

    for b in &board.boards {
        for i in &b.required {
            required.insert(i.id, i.title.clone());
        }
        for i in &b.demo {
            demo.insert(i.id, i.title.clone());
        }
    }

    // blockers come from blocked_chains count in BoardStatus
    let blockers = status.blocked_chains.len() as u32;

    Ok(BoardSnapshot {
        required,
        demo,
        blockers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_notify_first_time() {
        let map = HashMap::new();
        assert!(should_notify(&map, "key1", Duration::from_secs(30)));
    }

    #[test]
    fn test_should_notify_within_cooldown() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Instant::now());
        assert!(!should_notify(&map, "key1", Duration::from_secs(30)));
    }

    #[test]
    fn test_should_notify_after_cooldown() {
        let mut map = HashMap::new();
        // Insert with time far in the past using a zero-duration cooldown check
        map.insert(
            "key1".to_string(),
            Instant::now() - Duration::from_secs(60),
        );
        assert!(should_notify(&map, "key1", Duration::from_secs(30)));
    }

    #[test]
    fn test_tray_summary_default() {
        let s = TrayBoardSummary::default();
        assert_eq!(s.inbox, 0);
        assert_eq!(s.demo_review, 0);
        assert_eq!(s.blockers, 0);
    }
}
