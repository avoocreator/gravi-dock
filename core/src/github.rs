//! GitHub: commit hari ini, repo terakhir yang disentuh, dan contribution graph.
//! Mode tanpa token: cukup nama pengguna (API publik + halaman kontribusi). Dengan token: GraphQL (termasuk repo privat).
//! Pengambilan HTTP dilakukan app lewat curl.exe; di sini hanya URL, badan permintaan, dan parser.
use crate::json::{self, Value};
use crate::prayer::days_from_civil;
use crate::timeutil::{date_key, local_day, parse_rfc3339, weekday};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Day { pub date: String, pub count: u32, pub level: u8 }

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Heat { pub days: Vec<Day>, pub total: u32 }

#[derive(Clone, Debug, PartialEq)]
pub struct CommitItem { pub repo: String, pub message: String, pub at: i64, pub count: u32 }

#[derive(Clone, Debug, PartialEq)]
pub struct RepoItem { pub name: String, pub at: i64, pub url: String }

#[derive(Clone, Debug, Default)]
pub struct Snapshot { pub user: String, pub heat: Heat, pub commits: Vec<CommitItem>, pub repos: Vec<RepoItem>, pub today_commits: u32, pub streak: u32 }

pub fn events_url(user: &str) -> String { format!("https://api.github.com/users/{user}/events?per_page=60") }
pub fn contrib_url(user: &str) -> String { format!("https://github.com/users/{user}/contributions") }
pub const GRAPHQL_URL: &str = "https://api.github.com/graphql";

/// Badan GraphQL: kalender kontribusi + 6 repo yang terakhir di-push. `viewer` = pemilik token.
pub fn graphql_body() -> String {
    let q = "query { viewer { login contributionsCollection { contributionCalendar { totalContributions weeks { contributionDays { contributionCount date contributionLevel } } } } repositories(first: 6, orderBy: {field: PUSHED_AT, direction: DESC}, ownerAffiliations: [OWNER, COLLABORATOR]) { nodes { nameWithOwner pushedAt url } } } }";
    Value::obj(vec![("query", Value::s(q))]).to_json()
}

/// Nama pengguna layak dipakai di URL? (huruf/angka/tanda hubung, maks 39 karakter)
pub fn valid_user(u: &str) -> bool { !u.is_empty() && u.len() <= 39 && u.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') }

fn level_from_name(s: &str) -> u8 {
    match s { "FIRST_QUARTILE" => 1, "SECOND_QUARTILE" => 2, "THIRD_QUARTILE" => 3, "FOURTH_QUARTILE" => 4, _ => 0 }
}
fn level_from_count(c: u32, max: u32) -> u8 { if c == 0 || max == 0 { 0 } else { (((c as f64 / max as f64) * 4.0).ceil() as u8).clamp(1, 4) } }

/// -> (login, heat, repo terbaru)
pub fn parse_graphql(src: &str) -> Option<(String, Heat, Vec<RepoItem>)> {
    let v = json::parse(src)?;
    let me = v.get("data").get("viewer");
    if me.is_null() { return None; }
    let cal = me.get("contributionsCollection").get("contributionCalendar");
    let mut days = vec![];
    for w in cal.get("weeks").arr() {
        for d in w.get("contributionDays").arr() {
            days.push(Day { date: d.get("date").text().to_string(), count: d.get("contributionCount").num().unwrap_or(0.0) as u32, level: level_from_name(d.get("contributionLevel").text()) });
        }
    }
    let total = cal.get("totalContributions").num().unwrap_or(0.0) as u32;
    let repos = me.get("repositories").get("nodes").arr().iter().map(|r| RepoItem {
        name: r.get("nameWithOwner").text().to_string(), at: parse_rfc3339(r.get("pushedAt").text()).unwrap_or(0), url: r.get("url").text().to_string(),
    }).collect();
    Some((me.get("login").text().to_string(), Heat { days, total }, repos))
}

/// Parser halaman `github.com/users/<u>/contributions`: ambil `data-date` + `data-level` dari tiap sel,
/// total dari kalimat "N contributions in the last year".
pub fn parse_contrib_html(html: &str) -> Heat {
    let mut days: Vec<Day> = vec![];
    let mut rest = html;
    while let Some(i) = rest.find("data-date=\"") {
        let after = &rest[i + 11..];
        let Some(end) = after.find('"') else { break };
        let date = after[..end].to_string();
        // atribut data-level berada dalam tag yang sama (sebelum '>')
        let tag_end = after.find('>').unwrap_or(after.len());
        let tag = &after[..tag_end];
        let lvl = tag.find("data-level=\"").and_then(|p| tag[p + 12..].chars().next()).and_then(|c| c.to_digit(10)).unwrap_or(0) as u8;
        if date.len() == 10 && !days.iter().any(|d| d.date == date) { days.push(Day { date, count: 0, level: lvl.min(4) }); }
        rest = &after[end..];
    }
    days.sort_by(|a, b| a.date.cmp(&b.date));
    let total = html.find("contributions in the last year").map(|p| {
        let head = html[..p].trim_end();
        let digits: String = head.chars().rev().take_while(|c| c.is_ascii_digit() || *c == ',').collect::<Vec<_>>().into_iter().rev().filter(|c| *c != ',').collect();
        digits.parse::<u32>().unwrap_or(0)
    }).unwrap_or(0);
    for d in days.iter_mut() { d.count = d.level as u32; } // perkiraan kasar; level yang dipakai untuk warna
    Heat { days, total }
}

/// Parser `/users/<u>/events`: commit hari ini (menurut hari lokal) dan repo yang paling baru disentuh.
pub fn parse_events(src: &str, tz: f64, today_local_day: i64) -> (Vec<CommitItem>, Vec<RepoItem>) {
    let (mut commits, mut repos): (Vec<CommitItem>, Vec<RepoItem>) = (vec![], vec![]);
    let Some(v) = json::parse(src) else { return (commits, repos) };
    for e in v.arr() {
        let at = parse_rfc3339(e.get("created_at").text()).unwrap_or(0);
        let repo = e.get("repo").get("name").text().to_string();
        if repo.is_empty() { continue; }
        if !repos.iter().any(|r| r.name == repo) {
            repos.push(RepoItem { url: format!("https://github.com/{repo}"), name: repo.clone(), at });
        }
        if e.get("type").text() == "PushEvent" && local_day(at, tz) == today_local_day {
            let p = e.get("payload");
            let list = p.get("commits").arr();
            if list.is_empty() {
                let n = p.get("distinct_size").num().or(p.get("size").num()).unwrap_or(1.0) as u32;
                commits.push(CommitItem { repo: repo.clone(), message: String::new(), at, count: n.max(1) });
            } else {
                for c in list.iter().rev() {
                    let msg = c.get("message").text().lines().next().unwrap_or("").to_string();
                    commits.push(CommitItem { repo: repo.clone(), message: msg, at, count: 1 });
                }
            }
        }
    }
    (commits, repos)
}

impl Heat {
    /// Hari berurutan berakhir pada `today` (boleh hari ini belum ada kontribusi), dihitung mundur.
    pub fn streak(&self, today: &str) -> u32 {
        let mut by: Vec<&Day> = self.days.iter().filter(|d| d.date.as_str() <= today).collect();
        by.sort_by(|a, b| b.date.cmp(&a.date));
        let mut n = 0;
        for (i, d) in by.iter().enumerate() {
            if d.level > 0 || d.count > 0 { n += 1; } else if i == 0 { continue; } else { break; }
        }
        n
    }
    pub fn max_count(&self) -> u32 { self.days.iter().map(|d| d.count).max().unwrap_or(0) }

    /// Sel heatmap kolom-per-pekan (Minggu di baris 0) untuk `weeks` pekan terakhir; nilai 0..4, -1 = kosong.
    /// Indeks = kolom * 7 + baris, siap dipakai Repeater.
    pub fn cells(&self, weeks: usize) -> Vec<i32> {
        let mut out = vec![-1i32; weeks * 7];
        let Some(last) = self.days.iter().map(|d| d.date.as_str()).max() else { return out };
        let day_no = |s: &str| -> Option<i64> {
            let n = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
            Some(days_from_civil(n(0, 4)?, n(5, 7)?, n(8, 10)?))
        };
        let Some(last_no) = day_no(last) else { return out };
        let last_col_start = last_no - ((weekday(last_no) + 1) % 7) as i64; // Minggu pekan terakhir
        let max = self.max_count();
        for d in &self.days {
            let Some(no) = day_no(&d.date) else { continue };
            let col_from_end = (last_col_start - (no - ((weekday(no) + 1) % 7) as i64)) / 7;
            if col_from_end < 0 || col_from_end as usize >= weeks { continue; }
            let col = weeks - 1 - col_from_end as usize;
            let row = (weekday(no) + 1) % 7;
            out[col * 7 + row] = if d.level > 0 || max == 0 { d.level as i32 } else { level_from_count(d.count, max) as i32 };
        }
        out
    }
}

/// Susun `Snapshot` dari hasil mentah.
pub fn snapshot(user: &str, heat: Heat, commits: Vec<CommitItem>, repos: Vec<RepoItem>, tz: f64, now_utc: i64) -> Snapshot {
    let today = date_key(local_day(now_utc, tz));
    let today_commits = commits.iter().map(|c| c.count).sum();
    let streak = heat.streak(&today);
    Snapshot { user: user.to_string(), heat, commits, repos, today_commits, streak }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVENTS: &str = r#"[
      {"type":"PushEvent","repo":{"name":"avoo/gravi-island"},"created_at":"2026-11-23T10:00:00Z","payload":{"commits":[{"message":"fix: bug\n\nbody"},{"message":"feat: widget"}]}},
      {"type":"WatchEvent","repo":{"name":"rust-lang/rust"},"created_at":"2026-11-23T09:00:00Z","payload":{}},
      {"type":"PushEvent","repo":{"name":"avoo/other"},"created_at":"2026-11-20T09:00:00Z","payload":{"commits":[{"message":"lama"}]}},
      {"type":"PushEvent","repo":{"name":"avoo/nocommits"},"created_at":"2026-11-23T11:00:00Z","payload":{"size":3}}
    ]"#;

    #[test]
    fn events_today_and_repos() {
        let today = local_day(parse_rfc3339("2026-11-23T12:00:00Z").unwrap(), 7.0);
        let (c, r) = parse_events(EVENTS, 7.0, today);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].message, "feat: widget");
        assert_eq!(c[1].message, "fix: bug");
        assert_eq!((c[2].repo.as_str(), c[2].count, c[2].message.as_str()), ("avoo/nocommits", 3, ""));
        assert_eq!(r.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["avoo/gravi-island", "rust-lang/rust", "avoo/other", "avoo/nocommits"]);
    }

    #[test]
    fn graphql_parse_and_cells() {
        let g = r#"{"data":{"viewer":{"login":"avoo","contributionsCollection":{"contributionCalendar":{"totalContributions":9,"weeks":[
          {"contributionDays":[{"contributionCount":0,"date":"2026-11-15","contributionLevel":"NONE"},{"contributionCount":2,"date":"2026-11-16","contributionLevel":"FIRST_QUARTILE"}]},
          {"contributionDays":[{"contributionCount":7,"date":"2026-11-22","contributionLevel":"FOURTH_QUARTILE"},{"contributionCount":0,"date":"2026-11-23","contributionLevel":"NONE"}]}]}},
          "repositories":{"nodes":[{"nameWithOwner":"avoo/gravi-island","pushedAt":"2026-11-23T10:00:00Z","url":"https://github.com/avoo/gravi-island"}]}}}}"#;
        let (login, heat, repos) = parse_graphql(g).unwrap();
        assert_eq!((login.as_str(), heat.total, heat.days.len(), repos.len()), ("avoo", 9, 4, 1));
        let cells = heat.cells(3);
        assert_eq!(cells.len(), 21);
        // 2026-11-22 adalah Minggu -> baris 0 di kolom terakhir; 2026-11-23 Senin -> baris 1
        assert_eq!(cells[2 * 7], 4);
        assert_eq!(cells[2 * 7 + 1], 0);
        // 2026-11-15 Minggu pekan sebelumnya -> kolom 1 baris 0; 16 Nov Senin -> baris 1
        assert_eq!(cells[7 + 1], 1);
        assert_eq!(cells[0], -1);
        assert!(parse_graphql("{\"errors\":[]}").is_none());
    }

    #[test]
    fn html_contributions() {
        let html = r#"<h2 class="f4">1,234 contributions in the last year</h2>
          <td tabindex="-1" data-ix="0" aria-selected="false" data-date="2026-11-21" id="c1" data-level="0" class="ContributionCalendar-day"></td>
          <td data-date="2026-11-22" id="c2" data-level="3" class="ContributionCalendar-day"></td>"#;
        let h = parse_contrib_html(html);
        assert_eq!(h.total, 1234);
        assert_eq!(h.days.len(), 2);
        assert_eq!(h.days[1].level, 3);
        assert_eq!(h.streak("2026-11-22"), 1);
    }

    #[test]
    fn streak_skips_empty_today() {
        let mk = |d: &str, l: u8| Day { date: d.into(), count: l as u32, level: l };
        let h = Heat { days: vec![mk("2026-11-20", 1), mk("2026-11-21", 2), mk("2026-11-22", 1), mk("2026-11-23", 0)], total: 4 };
        assert_eq!(h.streak("2026-11-23"), 3);
        let broken = Heat { days: vec![mk("2026-11-20", 1), mk("2026-11-21", 0), mk("2026-11-22", 1)], total: 2 };
        assert_eq!(broken.streak("2026-11-22"), 1);
    }

    #[test]
    fn helpers() {
        assert!(valid_user("avoo-creator") && !valid_user("a b") && !valid_user(""));
        assert!(graphql_body().contains("contributionCalendar"));
        assert!(json::parse(&graphql_body()).is_some());
    }
}
