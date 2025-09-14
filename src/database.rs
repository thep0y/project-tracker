use sqlx::{query, query_as, SqlitePool};

use crate::models::{CountryStats, Project, ProjectDetailedStats, ProjectStats, Visit};

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

    let project_name = basic_stats.project_name;
    let repository = project_name.repository().to_owned();
    let icon = project_name.icon().to_owned();
    let description = project_name.description().to_owned();

    Ok(ProjectDetailedStats {
        project_name,
        repository,
        icon,
        description,
        total_visits: basic_stats.total_visits,
        unique_visitors: basic_stats.unique_visitors,
        country_stats,
        recent_visits,
    })
}

/// 根据特定日期查询项目统计（格式：YYYY-MM-DD）
pub async fn get_project_stats_by_date(
    pool: &SqlitePool,
    project_name: &Project,
    date: &str,
) -> Result<ProjectStats, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            ? as project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        WHERE project_name = ? 
        AND DATE(created_at) = ?
        "#,
    )
    .bind(project_name)
    .bind(project_name)
    .bind(date)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!("按日期查询数据库失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

/// 根据特定月份查询项目统计（格式：YYYY-MM）
pub async fn get_project_stats_by_month(
    pool: &SqlitePool,
    project_name: &Project,
    year_month: &str,
) -> Result<ProjectStats, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            ? as project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        WHERE project_name = ? 
        AND strftime('%Y-%m', created_at) = ?
        "#,
    )
    .bind(project_name)
    .bind(project_name)
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!("按月份查询数据库失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

/// 根据特定年份查询项目统计（格式：YYYY）
pub async fn get_project_stats_by_year(
    pool: &SqlitePool,
    project_name: &Project,
    year: &str,
) -> Result<ProjectStats, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            ? as project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        WHERE project_name = ? 
        AND strftime('%Y', created_at) = ?
        "#,
    )
    .bind(project_name)
    .bind(project_name)
    .bind(year)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!("按年份查询数据库失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

/// 获取所有项目在特定日期的统计（格式：YYYY-MM-DD）
pub async fn get_all_projects_stats_by_date(
    pool: &SqlitePool,
    date: &str,
) -> Result<Vec<ProjectStats>, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        WHERE DATE(created_at) = ?
        GROUP BY project_name
        ORDER BY total_visits DESC
        "#,
    )
    .bind(date)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("按日期查询所有项目数据库失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

/// 获取所有项目在特定月份的统计（格式：YYYY-MM）
pub async fn get_all_projects_stats_by_month(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<ProjectStats>, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        WHERE strftime('%Y-%m', created_at) = ?
        GROUP BY project_name
        ORDER BY total_visits DESC
        "#,
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("按月份查询所有项目数据库失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

/// 获取所有项目在特定年份的统计（格式：YYYY）
pub async fn get_all_projects_stats_by_year(
    pool: &SqlitePool,
    year: &str,
) -> Result<Vec<ProjectStats>, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        WHERE strftime('%Y', created_at) = ?
        GROUP BY project_name
        ORDER BY total_visits DESC
        "#,
    )
    .bind(year)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!("按年份查询所有项目数据库失败: {:?}", e);
        e
    })?;

    Ok(stats)
}

/// 根据日期范围查询项目统计（格式：YYYY-MM-DD）
pub async fn get_project_stats_by_date_range(
    pool: &SqlitePool,
    project_name: &Project,
    start_date: &str,
    end_date: &str,
) -> Result<ProjectStats, sqlx::Error> {
    let stats = query_as::<_, ProjectStats>(
        r#"
        SELECT 
            ? as project_name,
            COUNT(*) as total_visits,
            COUNT(DISTINCT ip_address) as unique_visitors
        FROM visits 
        WHERE project_name = ? 
        AND DATE(created_at) BETWEEN ? AND ?
        "#,
    )
    .bind(project_name)
    .bind(project_name)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!("按日期范围查询数据库失败: {:?}", e);
        e
    })?;

    Ok(stats)
}
