use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

#[macro_use]
extern crate tracing;

mod database;
mod handlers;
mod log;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    log::init();

    // 初始化数据库连接池
    let pool = database::init_database().await.map_err(|e| {
        error!("数据库初始化失败: {:?}", e);
        e
    })?;

    // 构建路由
    let app = Router::new()
        .route("/api/track/{project_name}", post(handlers::track_visit))
        .route(
            "/api/stats/{project_name}",
            get(handlers::get_project_stats),
        )
        .route("/api/stats", get(handlers::get_all_stats))
        .route(
            "/api/stats/{project_name}/time",
            get(handlers::get_project_stats_by_time),
        )
        .route(
            "/api/stats/time",
            get(handlers::get_all_projects_stats_by_time),
        )
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        error!("服务器绑定失败: {:?}", e);
        e
    })?;
    info!("服务器运行在 http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
