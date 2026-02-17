use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::path::Path;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(app_data_dir: &Path) -> Result<DbPool, sqlx::Error> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("stremio_vmb.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    )
    .execute(&pool)
    .await?;

    run_migrations(&pool).await?;
    Ok(pool)
}

async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    let migrations: &[(&str, &str)] = &[
        ("1", include_str!("../migrations/001_create_tables.sql")),
        ("2", include_str!("../migrations/002_extend_metadata.sql")),
        ("3", include_str!("../migrations/003_create_discover_tables.sql")),
        ("4", include_str!("../migrations/004_default_media_path.sql")),
        ("5", include_str!("../migrations/005_media_series_name.sql")),
        ("6", include_str!("../migrations/006_ai_recommendations_cache.sql")),
        ("7", include_str!("../migrations/007_music_tables.sql")),
    ];

    for (version, sql) in migrations {
        let version_num: i64 = version.parse().unwrap();
        let already_applied: bool = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1"
        )
        .bind(version_num)
        .fetch_one(pool)
        .await? > 0;

        if already_applied {
            continue;
        }

        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(pool).await?;
            }
        }

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (?1)")
            .bind(version_num)
            .execute(pool)
            .await?;

        log::info!("Applied migration v{}", version);
    }

    Ok(())
}
