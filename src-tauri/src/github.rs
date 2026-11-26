use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionDay {
    pub date: String,
    pub count: u32,
    pub level: u8,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitItem {
    pub sha: String,
    pub message: String,
    pub repo: String,
    pub committed_at: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSummary {
    pub username: String,
    pub range_months: u32,
    pub total_contributions: u32,
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
        3 => 91,
        6 => 182,
        _ => 365,
    };
    let from = (now - Duration::days(days_back)).to_rfc3339();
    let to = now.to_rfc3339();

    let query = r#"
    query($from: DateTime!, $to: DateTime!) {
      viewer {
        login
        contributionsCollection(from: $from, to: $to) {
          contributionCalendar {
            totalContributions
            weeks {
              contributionDays {
                date
                contributionCount
              }
            }
          }
        }
        repositories(first: 6, ownerAffiliations: [OWNER, COLLABORATOR], orderBy: {field: PUSHED_AT, direction: DESC}) {
          nodes {
            name
            nameWithOwner
            url
            pushedAt
            stargazerCount
            primaryLanguage {
              name
              color
            }
            defaultBranchRef {
              target {
                ... on Commit {
                  history(first: 3) {
                    nodes {
                      abbreviatedOid
                      messageHeadline
                      committedDate
                      url
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
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "nobi-island/0.1.0")
        .json(&json!({
            "query": query,
            "variables": { "from": from, "to": to }
        }))
        .send()
        .await
        .map_err(|e| format!("Gagal menghubungi GitHub API: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API error: HTTP {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if let Some(errors) = body.get("errors") {
        return Err(format!("GitHub GraphQL error: {}", errors));
    }

    let viewer = &body["data"]["viewer"];
    let username = viewer["login"].as_str().unwrap_or("unknown").to_string();
    let cal = &viewer["contributionsCollection"]["contributionCalendar"];
    let total_contributions = cal["totalContributions"].as_u64().unwrap_or(0) as u32;

    let mut days = Vec::new();
    if let Some(weeks) = cal["weeks"].as_array() {
        for w in weeks {
            if let Some(cdays) = w["contributionDays"].as_array() {
                for d in cdays {
                    let count = d["contributionCount"].as_u64().unwrap_or(0) as u32;
                    days.push(ContributionDay {
                        date: d["date"].as_str().unwrap_or("").to_string(),
                        count,
                        level: count_to_level(count),
                    });
                }
            }
        }
    }

    let mut repos = Vec::new();
    let mut commits = Vec::new();

    if let Some(nodes) = viewer["repositories"]["nodes"].as_array() {
        for r in nodes {
            let repo_name = r["name"].as_str().unwrap_or("").to_string();
            repos.push(RepoItem {
                name: repo_name.clone(),
                full_name: r["nameWithOwner"].as_str().unwrap_or("").to_string(),
                language: r["primaryLanguage"]["name"].as_str().map(|s| s.to_string()),
                language_color: r["primaryLanguage"]["color"].as_str().map(|s| s.to_string()),
                updated_at: r["pushedAt"].as_str().unwrap_or("").to_string(),
                url: r["url"].as_str().unwrap_or("").to_string(),
                stars: r["stargazerCount"].as_u64().unwrap_or(0) as u32,
            });

            if let Some(hist) = r["defaultBranchRef"]["target"]["history"]["nodes"].as_array() {
                for c in hist {
                    commits.push(CommitItem {
                        sha: c["abbreviatedOid"].as_str().unwrap_or("").to_string(),
                        message: c["messageHeadline"].as_str().unwrap_or("").to_string(),
                        repo: repo_name.clone(),
                        committed_at: c["committedDate"].as_str().unwrap_or("").to_string(),
                        url: c["url"].as_str().unwrap_or("").to_string(),
                    });
                }
            }
        }
    }

    commits.sort_by(|a, b| b.committed_at.cmp(&a.committed_at));
    commits.truncate(10);

    Ok(GitHubSummary {
        username,
        range_months: months,
        total_contributions,
        days,
        repos,
        commits,
    })
}
