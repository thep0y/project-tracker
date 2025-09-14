use std::env;

use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub fn init() {
    // 从环境变量获取日志级别，如果未设置则根据环境使用默认值
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "trace".to_string()
        } else {
            "warn".to_string()
        }
    });

    let env_filter = format!("{}={}", env!("CARGO_PKG_NAME").replace("-", "_"), log_level);

    if cfg!(debug_assertions) {
        // 开发环境：输出到stderr
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(false)
            .with_line_number(true)
            .with_file(false)
            .with_ansi(true)
            .init();
    } else {
        // 生产环境：输出到文件
        let file_appender = tracing_appender::rolling::RollingFileAppender::new(
            tracing_appender::rolling::Rotation::DAILY,
            "log",
            "server",
        );

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_ansi(false)
            .with_writer(file_appender.with_max_level(Level::WARN))
            .init();
    }

    tracing::info!("Logger initialized successfully");
}
