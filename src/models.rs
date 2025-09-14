use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "UPPERCASE")]
pub enum Project {
    DWALL,
    LSAR,
    UP2B,
    FLUXY,
}

impl Project {
    /// 项目的仓库地址
    pub fn repository(&self) -> &'static str {
        match self {
            Project::DWALL => "https://github.com/dwall-rs/dwall",
            Project::LSAR => "https://github.com/alley-rs/lsar",
            Project::UP2B => "https://github.com/up2b/up2b",
            Project::FLUXY => "https://github.com/alley-rs/fluxy",
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Visit {
    pub id: i64,
    pub project_name: Project,
    pub ip_address: String,
    pub country: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProjectStats {
    pub project_name: Project,
    pub total_visits: i64,
    pub unique_visitors: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CountryStats {
    pub country: Option<String>,
    pub visit_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDetailedStats {
    pub project_name: Project,
    pub repository: String,
    pub total_visits: i64,
    pub unique_visitors: i64,
    pub country_stats: Vec<CountryStats>,
    pub recent_visits: Vec<Visit>,
}
