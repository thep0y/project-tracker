use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum Project {
    Dwall,
    Lsar,
    UP2B,
    Fluxy,
}

impl Project {
    /// 项目的仓库地址
    pub fn repository(&self) -> &'static str {
        match self {
            Project::Dwall => "https://github.com/dwall-rs/dwall",
            Project::Lsar => "https://github.com/alley-rs/lsar",
            Project::UP2B => "https://github.com/up2b/up2b",
            Project::Fluxy => "https://github.com/alley-rs/fluxy",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Project::Dwall => "https://raw.githubusercontent.com/dwall-rs/dwall/refs/heads/main/src-tauri/icons/icon.ico",
            Project::Lsar => "https://raw.githubusercontent.com/alley-rs/lsar/refs/heads/main/src-tauri/icons/icon.ico",
            Project::UP2B => "https://raw.githubusercontent.com/up2b/up2b/refs/heads/main/src-tauri/icons/icon.ico",
            Project::Fluxy => "https://raw.githubusercontent.com/alley-rs/fluxy/refs/heads/main/src-tauri/icons/icon.ico",}
    }

    pub fn description(&self) -> &'static str {
        match self {
            Project::Dwall => "在 Windows 中模拟 macOS 根据时间切换壁纸的程序",
            Project::Lsar => "聚合多个平台的直播解析程序，目前支持斗鱼、虎牙、抖音、B站、Bigo",
            Project::UP2B => "支持多个图床的图床管理程序",
            Project::Fluxy => "轻量、快速的文件传输工具",
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
    pub icon: String,
    pub description: String,
    pub total_visits: i64,
    pub unique_visitors: i64,
    pub country_stats: Vec<CountryStats>,
    pub recent_visits: Vec<Visit>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TimeQuery {
    Date {
        date: String,
    },
    Month {
        month: String,
    },
    Year {
        year: String,
    },
    Range {
        start_date: String,
        end_date: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryParams {
    pub time: Option<TimeQuery>,
}
