use chrono::{Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionDay {
    pub date: String,
    pub count: u32,
    pub level: u8,
    pub weekday: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoItem {
    pub name: String,
    pub full_name: String,
    pub language: Option<String>,
    pub language_color: Option<String>,
    pub updated_at: String,
    pub url: String,
    pub stars: u32,
    pub is_private: bool,
    pub is_fork: bool,
    pub default_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitItem {
    pub sha: String,
    pub message: String,
    pub repo: String,
    pub branch: String,
    pub committed_at: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSummary {
    pub username: String,
    pub range_months: u32,
    pub total_contributions: u32,
    pub today_contributions: u32,
    pub current_streak: u32,
    pub days: Vec<ContributionDay>,
    pub repos: Vec<RepoItem>,
    pub commits: Vec<CommitItem>,
}

fn count_to_level(count: u32) -> u8 {
    match count {
        0 => 0,
        1..=2 => 1,
        3..=5 => 2,
        6..=9 => 3,
        _ => 4,
    }
}

fn is_bot_name(s: &str) -> bool {
    let low = s.to_lowercase();
    low.contains("[bot]") || low.contains("dependabot") || low.contains("github-actions")
}

#[tauri::command]
pub async fn fetch_github_summary(range_months: Option<u32>) -> Result<GitHubSummary, String> {
    let months = match range_months.unwrap_or(3) {
        3 | 6 | 12 => range_months.unwrap_or(3),
        _ => 3,
    };

    let token = crate::secrets::read_secret_internal("github_token")
        .ok_or_else(|| "GitHub Personal Access Token belum diatur di Pengaturan.".to_string())?;

    let now = Utc::now();
    let days_back = match months {
        3 => 90,
        6 => 180,
        _ => 364,
    };
    let from = (now - Duration::days(days_back))
        .format("%Y-%m-%dT00:00:00Z")
        .to_string();
    let to = now.format("%Y-%m-%dT23:59:59Z").to_string();

    let query = r#"
    query($from: DateTime!, $to: DateTime!) {
      viewer {
        login
        name
        contributionsCollection(from: $from, to: $to) {
          contributionCalendar {
            totalContributions
            weeks {
              contributionDays {
                date
                contributionCount
                weekday
              }
            }
          }
        }
        repositories(
          first: 15,
          ownerAffiliations: [OWNER, COLLABORATOR],
          isArchived: false,
          orderBy: {field: PUSHED_AT, direction: DESC}
        ) {
          nodes {
            name
            nameWithOwner
            url
            pushedAt
            stargazerCount
            isPrivate
            isFork
            owner {
              login
            }
            primaryLanguage {
              name
              color
            }
            defaultBranchRef {
              name
              target {
                ... on Commit {
                  history(first: 8) {
                    nodes {
                      abbreviatedOid
                      messageHeadline
                      committedDate
                      url
                      author {
                        name
                        user {
                          login
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    "#;

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token.trim()))
        .header("User-Agent", "nobi-island/0.1.0")
        .json(&json!({
            "query": query,
            "variables": { "from": from, "to": to }
        }))
        .send()
        .await
        .map_err(|e| format!("Gagal menghubungi GitHub API: {}", e))?;

    let status = resp.status();
    if status.as_u16() == 401 {
        return Err("Token GitHub tidak valid atau sudah kedaluwarsa. Perbarui PAT di Pengaturan.".into());
    }
    if status.as_u16() == 403 {
        return Err("Akses GitHub ditolak (HTTP 403). Pastikan PAT memiliki scope read:user dan repo.".into());
    }
    if !status.is_success() {
        return Err(format!("GitHub API error: HTTP {}", status.as_u16()));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let viewer = &body["data"]["viewer"];

    if viewer.is_null() {
        if let Some(err_msg) = body["errors"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|e| e["message"].as_str())
        {
            return Err(format!("GitHub GraphQL: {}", err_msg));
        }
        return Err("Data akun GitHub kosong. Periksa izin PAT di Pengaturan.".into());
    }

    let username = viewer["login"].as_str().unwrap_or("unknown").to_string();
    let viewer_login_low = username.to_lowercase();
    let viewer_name_low = viewer["name"].as_str().unwrap_or("").trim().to_lowercase();

    let cal = &viewer["contributionsCollection"]["contributionCalendar"];
    let total_contributions = cal["totalContributions"].as_u64().unwrap_or(0) as u32;

    let mut days = Vec::new();
    if let Some(weeks) = cal["weeks"].as_array() {
        for w in weeks {
            if let Some(cdays) = w["contributionDays"].as_array() {
                for d in cdays {
                    let count = d["contributionCount"].as_u64().unwrap_or(0) as u32;
                    let weekday = d["weekday"].as_u64().unwrap_or(0).min(6) as u8;
                    days.push(ContributionDay {
                        date: d["date"].as_str().unwrap_or("").to_string(),
                        count,
                        level: count_to_level(count),
                        weekday,
                    });
                }
            }
        }
    }

    let today_str = Local::now().format("%Y-%m-%d").to_string();
    let today_contributions = days
        .iter()
        .find(|d| d.date == today_str)
        .or_else(|| days.last())
        .map(|d| d.count)
        .unwrap_or(0);

    let mut current_streak: u32 = 0;
    for (idx, d) in days.iter().rev().enumerate() {
        if d.count > 0 {
            current_streak += 1;
        } else if idx == 0 && d.date == today_str {
            continue;
        } else {
            break;
        }
    }

    let mut repos = Vec::new();
    let mut commits = Vec::new();
    let mut seen_commit_keys: HashSet<String> = HashSet::new();

    if let Some(nodes) = viewer["repositories"]["nodes"].as_array() {
        for r in nodes {
            if r.is_null() {
                continue;
            }
            let repo_name = r["name"].as_str().unwrap_or("").to_string();
            let full_name = r["nameWithOwner"].as_str().unwrap_or("").to_string();
            if repo_name.is_empty() {
                continue;
            }
            let owner_login = r["owner"]["login"].as_str().unwrap_or("").to_lowercase();
            let is_fork = r["isFork"].as_bool().unwrap_or(false);
            let is_private = r["isPrivate"].as_bool().unwrap_or(false);
            let pushed_at = r["pushedAt"].as_str().unwrap_or("").to_string();
            let default_branch = r["defaultBranchRef"]["name"].as_str().map(|s| s.to_string());
            let branch_label = default_branch.clone().unwrap_or_else(|| "main".into());

            let mut user_commit_count_in_repo = 0usize;
            if let Some(hist) = r["defaultBranchRef"]["target"]["history"]["nodes"].as_array() {
                for c in hist {
                    let sha = c["abbreviatedOid"].as_str().unwrap_or("").to_string();
                    let msg = c["messageHeadline"].as_str().unwrap_or("").to_string();
                    if sha.is_empty() || msg.is_empty() {
                        continue;
                    }

                    let author_login = c["author"]["user"]["login"]
                        .as_str()
                        .unwrap_or("")
                        .to_lowercase();
                    let author_name = c["author"]["name"]
                        .as_str()
                        .unwrap_or("")
                        .trim()
                        .to_lowercase();

                    if is_bot_name(&author_login) || is_bot_name(&author_name) {
                        continue;
                    }

                    let matches_user = if !author_login.is_empty() {
                        author_login == viewer_login_low
                    } else {
                        (!viewer_name_low.is_empty() && author_name == viewer_name_low)
                            || author_name == viewer_login_low
                    };

                    if !matches_user {
                        continue;
                    }

                    user_commit_count_in_repo += 1;
                    let key = format!("{}#{}", full_name, sha);
                    if seen_commit_keys.insert(key) {
                        commits.push(CommitItem {
                            sha,
                            message: msg,
                            repo: repo_name.clone(),
                            branch: branch_label.clone(),
                            committed_at: c["committedDate"].as_str().unwrap_or("").to_string(),
                            url: c["url"].as_str().unwrap_or("").to_string(),
                        });
                    }
                }
            }

            let is_owned = owner_login == viewer_login_low;
            if is_fork && user_commit_count_in_repo == 0 {
                continue;
            }
            if !is_owned && user_commit_count_in_repo == 0 {
                continue;
            }

            repos.push(RepoItem {
                name: repo_name,
                full_name,
                language: r["primaryLanguage"]["name"].as_str().map(|s| s.to_string()),
                language_color: r["primaryLanguage"]["color"].as_str().map(|s| s.to_string()),
                updated_at: pushed_at,
                url: r["url"].as_str().unwrap_or("").to_string(),
                stars: r["stargazerCount"].as_u64().unwrap_or(0) as u32,
                is_private,
                is_fork,
                default_branch,
            });
        }
    }

    repos.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    repos.truncate(10);

    commits.sort_by(|a, b| b.committed_at.cmp(&a.committed_at));
    commits.truncate(12);

    Ok(GitHubSummary {
        username,
        range_months: months,
        total_contributions,
        today_contributions,
        current_streak,
        days,
        repos,
        commits,
    })
}
