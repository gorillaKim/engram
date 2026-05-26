use engram_core::{repository::session::StalledIssueBrief, Db};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TrayStallEntry {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TrayBoardSummary {
    pub inbox: u32,
    pub demo_review: u32,
    pub working: u32,
    /// "active" | "pending" | "stalled" | "none"
    pub working_state: String,
    /// 작업중단 의심 이슈 최대 3개 (tray 팝오버 표시용)
    pub stalled_issues: Vec<TrayStallEntry>,
    pub stalled_total: u32,
}

struct BoardSnapshot {
    required: HashMap<i64, String>,
    demo: HashMap<i64, String>,
    working: HashMap<i64, String>,
    blockers: u32,
    /// 초 단위 경과 시간. u64::MAX = 히스토리 없음
    last_activity_secs: u64,
    stalled: Vec<StalledIssueBrief>,
}

impl Default for BoardSnapshot {
    fn default() -> Self {
        Self {
            required: HashMap::new(),
            demo: HashMap::new(),
            working: HashMap::new(),
            blockers: 0,
            last_activity_secs: u64::MAX,
            stalled: vec![],
        }
    }
}

pub async fn run(app: AppHandle, db: Arc<Db>) {
    let mut last = BoardSnapshot::default();
    let mut cooldown_map: HashMap<String, Instant> = HashMap::new();
    let cooldown = Duration::from_secs(30);
    let mut tick: u32 = 0;
    // 첫 틱은 기준선 수립만 — 앱 시작 시 기존 이슈를 "새 알림"으로 오발송 방지
    let mut initialized = false;

    // 애니메이션 공유 상태
    let working_count = Arc::new(AtomicU32::new(0));
    let demo_only = Arc::new(AtomicBool::new(false));
    let alert_count = Arc::new(AtomicU32::new(0)); // inbox + demo_review
    let last_activity_secs = Arc::new(AtomicU64::new(u64::MAX));
    let warn_secs_atom = Arc::new(AtomicU64::new(1800));   // 30min default
    let stall_secs_atom = Arc::new(AtomicU64::new(7200));  // 120min default

    {
        let app2 = app.clone();
        let wc = working_count.clone();
        let dc = demo_only.clone();
        let la = last_activity_secs.clone();
        let ws = warn_secs_atom.clone();
        let ss = stall_secs_atom.clone();
        tokio::spawn(async move {
            // 브레일 스피너 프레임 — working 상태 시 회전 애니메이션
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut i = 0usize;
            loop {
                tokio::time::sleep(Duration::from_millis(120)).await;
                i = (i + 1) % frames.len();
                let w = wc.load(Ordering::Relaxed);
                let d = dc.load(Ordering::Relaxed);
                let las = la.load(Ordering::Relaxed);
                let w_secs = ws.load(Ordering::Relaxed);
                let s_secs = ss.load(Ordering::Relaxed);
                if let Some(tray) = app2.tray_by_id("default") {
                    let title = if w > 0 {
                        match classify_working(las, w_secs, s_secs) {
                            "active"  => format!("{} 작업중", frames[i]),
                            "pending" => "⏸ 작업예상".to_string(),
                            _         => "⚠ 작업중단".to_string(),
                        }
                    } else if d {
                        "👀 검토대기".to_string()
                    } else {
                        "💤 대기중".to_string()
                    };
                    let _ = tray.set_title(Some(&title));
                }
            }
        });
    }

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        tick = tick.wrapping_add(1);

        let activity = crate::settings::load()
            .unwrap_or_default()
            .activity;
        warn_secs_atom.store((activity.warn_minutes * 60) as u64, Ordering::Relaxed);
        stall_secs_atom.store((activity.stall_minutes * 60) as u64, Ordering::Relaxed);

        let cur = match snapshot(&db, activity.stall_minutes).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("watcher snapshot error: {e}");
                continue;
            }
        };

        // 팝오버용 요약 emit
        let w = cur.working.len() as u32;
        let w_secs = warn_secs_atom.load(Ordering::Relaxed);
        let s_secs = stall_secs_atom.load(Ordering::Relaxed);
        let state = if w > 0 { classify_working(cur.last_activity_secs, w_secs, s_secs) } else { "none" };
        let stalled_total = cur.stalled.len() as u32;
        let stalled_issues: Vec<TrayStallEntry> = cur.stalled.iter().take(3)
            .map(|s| TrayStallEntry { id: s.id, title: s.title.clone() })
            .collect();
        let summary = TrayBoardSummary {
            inbox: cur.required.len() as u32,
            demo_review: cur.demo.len() as u32,
            working: w,
            working_state: state.to_string(),
            stalled_issues,
            stalled_total,
        };
        let _ = app.emit("tray://summary", &summary);

        // 애니메이션 상태 갱신
        working_count.store(w, Ordering::Relaxed);
        demo_only.store(w == 0 && summary.demo_review > 0, Ordering::Relaxed);
        alert_count.store(summary.inbox + summary.demo_review, Ordering::Relaxed);
        last_activity_secs.store(cur.last_activity_secs, Ordering::Relaxed);

        // 첫 틱: 기준선만 수립하고 알림 로직은 건너뜀
        if !initialized {
            last = cur;
            initialized = true;
            continue;
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

        // Detect new blockers (count 증가 시만)
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

        // 약 1시간마다 만료된 cooldown 항목 정리
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

async fn snapshot(db: &Db, stall_minutes: i64) -> engram_core::Result<BoardSnapshot> {
    let board = db.board_issues_query(None, stall_minutes).await?;
    let status = db.board_status_query(None, false, true).await?;

    let mut required = HashMap::new();
    let mut demo = HashMap::new();
    let mut working = HashMap::new();

    for b in &board.boards {
        for i in &b.required {
            required.insert(i.id, i.title.clone());
        }
        for i in &b.demo {
            demo.insert(i.id, i.title.clone());
        }
        for i in &b.working {
            working.insert(i.id, i.title.clone());
        }
    }

    let blockers = match &status.blocked_chains {
        Some(serde_json::Value::Array(arr)) => arr.len() as u32,
        Some(serde_json::Value::Object(map)) => map.len() as u32,
        _ => 0,
    };

    let working_ids: Vec<i64> = working.keys().copied().collect();
    let last_activity_secs = match db.history_last_activity_secs_for_issues(&working_ids).await {
        Ok(Some(secs)) => secs.max(0) as u64,
        Ok(None) => u64::MAX,
        Err(e) => {
            tracing::warn!("history activity query error: {e}");
            u64::MAX
        }
    };

    let stalled = board.stalled_issues;

    Ok(BoardSnapshot { required, demo, working, blockers, last_activity_secs, stalled })
}

/// 히스토리 경과 시간으로 작업 상태 분류.
/// u64::MAX = 히스토리 없음 → "pending" (방금 시작 또는 기록 없음)
fn classify_working(last_secs: u64, warn_secs: u64, stall_secs: u64) -> &'static str {
    if last_secs == u64::MAX { "pending" }
    else if last_secs <= warn_secs { "active" }
    else if last_secs <= stall_secs { "pending" }
    else { "stalled" }
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
        assert_eq!(s.working, 0);
        assert_eq!(s.working_state, "");
    }

    #[test]
    fn test_classify_working() {
        let (w, s) = (1800u64, 7200u64);
        assert_eq!(classify_working(u64::MAX, w, s), "pending");
        assert_eq!(classify_working(0, w, s), "active");
        assert_eq!(classify_working(1800, w, s), "active");
        assert_eq!(classify_working(1801, w, s), "pending");
        assert_eq!(classify_working(7200, w, s), "pending");
        assert_eq!(classify_working(7201, w, s), "stalled");
    }
}
