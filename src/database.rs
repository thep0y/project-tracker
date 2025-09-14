use crate::models::{CountryStats, Project, ProjectDetailedStats, ProjectStats, Visit};
use sqlx::{query, query_as, SqlitePool};

pub async fn init_database() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:project_tracker.db?mode=rwc")
        .await
        .map_err(|e| {
            error!("数据库连接失败: {:?}", e);
            e
        })?;
    info!("数据库连接成功");

    // 创建表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS visits (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_name TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            country TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("数据库表创建失败: {:?}", e);
        e
    })?;
    info!("数据库表创建成功");

    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_visits_project_name ON visits(project_name)")
        .execute(&pool)
        .await
        .map_err(|e| {
            error!("数据库索引`idx_visits_project_name`创建失败: {:?}", e);
            e
        })?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_visits_created_at ON visits(created_at)")
        .execute(&pool)
        .await
        .map_err(|e| {
            error!("数据库索引`idx_visits_created_at`创建失败: {:?}", e);
            e
        })?;

    info!("数据库索引创建成功");

    Ok(pool)
}

pub async fn insert_visit(
    pool: &SqlitePool,
    project_name: &Project,
    ip_address: &str,
    country: Option<&str>,
) -> Result<(), sqlx::Error> {
    query("INSERT INTO visits (project_name, ip_address, country) VALUES (?, ?, ?)")
        .bind(project_name)
        .bind(ip_address)
        .bind(country)
        .execute(pool)
        .await
        .map_err(|e| {
            error!("数据库插入失败: {:?}", e);
            e
        })?;

    Ok(())
}

pub async fn get_project_stats(
    pool: &SqlitePool,
    project_name: &Project,
) -> Result<ProjectStats, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            ? as project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        WHERE project_name = ?
        "#,
    )
    .bind(project_name)
    .bind(project_name)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!("数据库查询失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

pub async fn get_all_projects_stats(pool: &SqlitePool) -> Result<Vec<ProjectStats>, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        GROUP BY project_name
        ORDER BY total_visits DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("数据库查询失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

pub async fn get_country_stats(
    pool: &SqlitePool,
    project_name: &Project,
) -> Result<Vec<CountryStats>, sqlx::Error> {
    let stats = query_as::<_, CountryStats>(
        r#"
        SELECT 
            country,
            COUNT(*) as visit_count
        FROM visits 
        WHERE project_name = ?
        GROUP BY country
        ORDER BY visit_count DESC
        "#,
    )
    .bind(project_name)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("数据库查询失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

pub async fn get_recent_visits(
    pool: &SqlitePool,
    project_name: &Project,
    limit: i32,
) -> Result<Vec<Visit>, sqlx::Error> {
    let visits: Vec<Visit> = query_as::<_, Visit>(
        r#"
        SELECT * FROM visits 
        WHERE project_name = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(project_name)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("数据库查询失败: {:?}", e);
        e
    })?;

    Ok(visits)
}

pub async fn get_project_detailed_stats(
    pool: &SqlitePool,
    project_name: &Project,
) -> Result<ProjectDetailedStats, sqlx::Error> {
    let basic_stats = get_project_stats(pool, project_name).await?;
    let country_stats = get_country_stats(pool, project_name).await?;
    let recent_visits = get_recent_visits(pool, project_name, 10).await?;

    Ok(ProjectDetailedStats {
        repository: basic_stats.project_name.repository().to_string(),
        project_name: basic_stats.project_name,
        total_visits: basic_stats.total_visits,
        unique_visitors: basic_stats.unique_visitors,
        country_stats,
        recent_visits,
    })
}
