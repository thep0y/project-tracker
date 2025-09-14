use crate::models::TrackResponse;
use crate::{database, models::Project};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Json,
};
use reqwest;
use serde_json::json;
use sqlx::SqlitePool;

pub async fn track_visit(
    Path(project_name): Path<Project>,
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Json<TrackResponse>, axum::http::StatusCode> {
    // 获取客户端IP
    let ip_address = get_client_ip(&headers);

    // 获取国家信息
    let country = get_country_from_ip(&ip_address).await;

    // 插入访问记录
    match database::insert_visit(&pool, &project_name, &ip_address, country.as_deref()).await {
        Ok(_) => Ok(Json(TrackResponse {
            success: true,
            message: "Visit tracked successfully".to_string(),
        })),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_project_stats(
    Path(project_name): Path<Project>,
    State(pool): State<SqlitePool>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    match database::get_project_detailed_stats(&pool, &project_name).await {
        Ok(stats) => Ok(Json(json!(stats))),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_all_stats(
    State(pool): State<SqlitePool>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    match database::get_all_projects_stats(&pool).await {
        Ok(stats) => Ok(Json(json!({
            "projects": stats
        }))),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn get_client_ip(headers: &HeaderMap) -> String {
    // 尝试从各种可能的头部获取真实IP
    if let Some(ip) = headers.get("x-forwarded-for") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str
                .split(',')
                .next()
                .unwrap_or("unknown")
                .trim()
                .to_string();
        }
    }

    if let Some(ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str.to_string();
        }
    }

    "unknown".to_string()
}

async fn get_country_from_ip(ip: &str) -> Option<String> {
    if ip == "unknown" || ip.starts_with("127.") || ip.starts_with("192.168.") {
        return Some("Local".to_string());
    }

    // 使用免费的IP地理位置API
    let client = reqwest::Client::new();
    let url = format!("http://ip-api.com/json/{}", ip);

    match client.get(&url).send().await {
        Ok(response) => {
            if let Ok(data) = response.json::<serde_json::Value>().await {
                data.get("countryCode")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| Some("Unknown".to_string()))
            } else {
                Some("Unknown".to_string())
            }
        }
        Err(_) => Some("Unknown".to_string()),
    }
}
