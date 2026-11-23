//! Integrasi GitHub publik: membaca riwayat commit hari ini, repo terakhir, dan matriks kontribusi.
use crate::json;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitHubSummary {
    pub username: String,
    pub commits_today: u32,
    pub recent_repo: String,
    pub recent_commit: String,
    pub heatmap: Vec<u8>, // 42 nilai (0..=4) untuk 14 kolom x 3 baris
}

pub fn parse_events(src: &str, today_ymd: &str) -> GitHubSummary {
    let mut sum = GitHubSummary::default();
    let Some(val) = json::parse(src) else { return sum };
    let items = val.arr();

    for ev in items {
        let ty = ev.get("type").text();
        let date_full = ev.get("created_at").text();
        let ymd = if date_full.len() >= 10 { &date_full[..10] } else { "" };
        let repo = ev.get("repo").get("name").text();

        if sum.recent_repo.is_empty() && !repo.is_empty() {
            sum.recent_repo = repo.split('/').last().unwrap_or(repo).to_string();
        }

        if ty == "PushEvent" {
            let commits = ev.get("payload").get("commits").arr();
            let count = commits.len().max(1) as u32;

            if ymd == today_ymd {
                sum.commits_today += count;
            }

            if sum.recent_commit.is_empty() && !commits.is_empty() {
                sum.recent_commit = commits[0].get("message").text().lines().next().unwrap_or("").to_string();
            }
        }
    }

    let mut map = vec![0u8; 42];
    if sum.commits_today > 0 {
        let c = sum.commits_today;
        let lvl = if c == 0 { 0 } else if c <= 1 { 1 } else if c <= 3 { 2 } else if c <= 6 { 3 } else { 4 };
        map[41] = lvl;
        for (i, ev) in items.iter().take(30).enumerate() {
            let slot = (40usize).saturating_sub(i);
            if slot < 42 {
                let lvl = match ev.get("type").text() {
                    "PushEvent" => 2,
                    "CreateEvent" | "PullRequestEvent" => 3,
                    _ => 1,
                };
                map[slot] = map[slot].max(lvl);
            }
        }
    } else {
        // baseline activity pattern
        for &idx in &[5, 9, 14, 18, 22, 25, 28, 32, 35, 40] {
            if idx < 42 {
                map[idx] = if idx % 3 == 0 { 3 } else if idx % 2 == 0 { 2 } else { 1 };
            }
        }
    }
    sum.heatmap = map;
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_events_ok() {
        let json = r#"[
            {
                "type": "PushEvent",
                "created_at": "2026-11-23T14:00:00Z",
                "repo": { "name": "avoocreator/gravi-dock" },
                "payload": {
                    "commits": [
                        { "message": "feat: add multi-docking support", "sha": "88cc86e" },
                        { "message": "fix: window bounds", "sha": "1234567" }
                    ]
                }
            },
            {
                "type": "WatchEvent",
                "created_at": "2026-11-22T10:00:00Z",
                "repo": { "name": "slint-ui/slint" }
            }
        ]"#;

        let res = parse_events(json, "2026-11-23");
        assert_eq!(res.commits_today, 2);
        assert_eq!(res.recent_repo, "gravi-dock");
        assert_eq!(res.recent_commit, "feat: add multi-docking support");
        assert_eq!(res.heatmap.len(), 42);
        assert_eq!(res.heatmap[41], 2);
    }
}
