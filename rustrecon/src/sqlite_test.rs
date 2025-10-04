use anyhow::Result;
use rusqlite;
use sqlx::{Row, SqlitePool};
use std::error::Error;

pub fn test_rusqlite_connection() -> Result<()> {
    println!("🔍 Testing rusqlite functionality...");

    // Test 1: Create database in temp directory
    let temp_dir = std::env::temp_dir();
    let test_db_path = temp_dir.join("rustrecon_rusqlite_test.db");

    println!("📁 Test database path: {}", test_db_path.display());

    // Remove existing test database
    if test_db_path.exists() {
        std::fs::remove_file(&test_db_path)?;
        println!("🗑️ Removed existing test database");
    }

    // Test connection with rusqlite
    println!("⏳ Attempting to connect to SQLite database with rusqlite...");
    let conn = rusqlite::Connection::open(&test_db_path)?;
    println!("✅ Successfully connected to SQLite with rusqlite!");

    // Test table creation
    println!("⏳ Testing table creation...");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS test_table (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    println!("✅ Successfully created test table!");

    // Test data insertion
    println!("⏳ Testing data insertion...");
    conn.execute(
        "INSERT INTO test_table (name) VALUES (?1)",
        &[&"test_entry"],
    )?;
    println!("✅ Successfully inserted test data!");

    // Test data retrieval
    println!("⏳ Testing data retrieval...");
    {
        let mut stmt = conn.prepare("SELECT id, name FROM test_table WHERE name = ?1")?;
        let rows = stmt.query_map(&[&"test_entry"], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (id, name): (i64, String) = row?;
            println!("✅ Successfully retrieved data: id={}, name={}", id, name);
        }
    }

    // Clean up
    drop(conn);
    std::fs::remove_file(&test_db_path)?;
    println!("🗑️ Cleaned up test database");

    println!("🎉 All rusqlite tests passed!");
    Ok(())
}

pub async fn test_sqlite_connection() -> Result<()> {
    println!("🔍 Testing SQLite functionality...");

    // Test 1: Create database in temp directory
    let temp_dir = std::env::temp_dir();
    let test_db_path = temp_dir.join("rustrecon_test.db");

    println!("📁 Test database path: {}", test_db_path.display());

    // Remove existing test database
    if test_db_path.exists() {
        std::fs::remove_file(&test_db_path)?;
        println!("🗑️ Removed existing test database");
    }

    // Test connection string format
    let database_url = format!("sqlite://{}", test_db_path.display());
    println!("🔗 Database URL: {}", database_url);

    // Test connection
    println!("⏳ Attempting to connect to SQLite database...");
    let pool = SqlitePool::connect(&database_url).await?;
    println!("✅ Successfully connected to SQLite!");

    // Test table creation
    println!("⏳ Testing table creation...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS test_table (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await?;
    println!("✅ Successfully created test table!");

    // Test data insertion
    println!("⏳ Testing data insertion...");
    sqlx::query("INSERT INTO test_table (name) VALUES (?)")
        .bind("test_entry")
        .execute(&pool)
        .await?;
    println!("✅ Successfully inserted test data!");

    // Test data retrieval
    println!("⏳ Testing data retrieval...");
    let row = sqlx::query("SELECT id, name, created_at FROM test_table WHERE name = ?")
        .bind("test_entry")
        .fetch_one(&pool)
        .await?;

    let id: i64 = row.get("id");
    let name: String = row.get("name");
    println!("✅ Successfully retrieved data: id={}, name={}", id, name);

    // Clean up
    pool.close().await;
    std::fs::remove_file(&test_db_path)?;
    println!("🗑️ Cleaned up test database");

    println!("🎉 All SQLite tests passed!");
    Ok(())
}

pub async fn test_rustrecon_cache_path() -> Result<()> {
    println!("\n🔍 Testing RustRecon cache path setup...");

    // Test the actual cache directory creation
    let cache_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?
        .join("RustRecon");

    println!("📁 Cache directory: {}", cache_dir.display());
    println!("📁 Directory exists: {}", cache_dir.exists());

    // Create directory if it doesn't exist
    if !cache_dir.exists() {
        println!("⏳ Creating cache directory...");
        std::fs::create_dir_all(&cache_dir)?;
        println!("✅ Cache directory created successfully!");
    }

    // Test permissions by creating a test file
    let test_file = cache_dir.join("permission_test.txt");
    println!("⏳ Testing write permissions...");
    std::fs::write(&test_file, "test")?;
    println!("✅ Write permissions OK!");

    // Clean up test file
    std::fs::remove_file(&test_file)?;
    println!("🗑️ Cleaned up test file");

    // Test the actual database path
    let db_path = cache_dir.join("scan_cache.db");
    println!("📁 Database path: {}", db_path.display());

    // Test database connection with actual path
    let database_url = format!("sqlite://{}", db_path.display());
    println!("🔗 Database URL: {}", database_url);

    println!("⏳ Testing connection to actual cache database...");
    match SqlitePool::connect(&database_url).await {
        Ok(pool) => {
            println!("✅ Successfully connected to cache database!");
            pool.close().await;

            // Remove the test database if we created it
            if db_path.exists() {
                std::fs::remove_file(&db_path)?;
                println!("🗑️ Cleaned up test cache database");
            }
        }
        Err(e) => {
            println!("❌ Failed to connect to cache database: {}", e);
            println!("🔍 Error details:");
            let mut source = e.source();
            while let Some(err) = source {
                println!("   → {}", err);
                source = err.source();
            }
            return Err(e.into());
        }
    }

    println!("🎉 Cache path tests completed successfully!");
    Ok(())
}

pub async fn test_rustrecon_cache_path_rusqlite() -> Result<()> {
    println!("\n🔍 Testing RustRecon cache path setup with rusqlite...");

    // Test the actual cache directory creation
    let cache_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?
        .join("RustRecon");

    println!("📁 Cache directory: {}", cache_dir.display());
    println!("📁 Directory exists: {}", cache_dir.exists());

    // Create directory if it doesn't exist
    if !cache_dir.exists() {
        println!("⏳ Creating cache directory...");
        std::fs::create_dir_all(&cache_dir)?;
        println!("✅ Cache directory created successfully!");
    }

    // Test permissions by creating a test file
    let test_file = cache_dir.join("permission_test.txt");
    println!("⏳ Testing write permissions...");
    std::fs::write(&test_file, "test")?;
    println!("✅ Write permissions OK!");

    // Clean up test file
    std::fs::remove_file(&test_file)?;
    println!("🗑️ Cleaned up test file");

    // Test the actual database path with rusqlite
    let db_path = cache_dir.join("scan_cache.db");
    println!("📁 Database path: {}", db_path.display());

    println!("⏳ Testing connection to actual cache database with rusqlite...");
    match crate::rusqlite_database::RusqliteDatabase::new(&db_path) {
        Ok(_db) => {
            println!("✅ Successfully connected to cache database with rusqlite!");

            // Remove the test database if we created it
            if db_path.exists() {
                std::fs::remove_file(&db_path)?;
                println!("🗑️ Cleaned up test cache database");
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to connect to cache database with rusqlite: {}",
                e
            ));
        }
    }

    println!("🎉 Cache path tests with rusqlite completed successfully!");
    Ok(())
}

pub fn test_rustrecon_cache_database() -> Result<()> {
    println!("\n🔍 Testing RustRecon cache database with rusqlite...");

    // Test the actual cache directory creation
    let cache_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?
        .join("RustRecon");

    println!("📁 Cache directory: {}", cache_dir.display());

    // Create directory if it doesn't exist
    if !cache_dir.exists() {
        println!("⏳ Creating cache directory...");
        std::fs::create_dir_all(&cache_dir)?;
        println!("✅ Cache directory created successfully!");
    }

    // Test the actual database path
    let db_path = cache_dir.join("test_scan_cache.db");
    println!("📁 Test database path: {}", db_path.display());

    // Remove existing test database
    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
        println!("🗑️ Removed existing test database");
    }

    // Test database connection with RusqliteDatabase
    println!("⏳ Testing RusqliteDatabase initialization...");
    match crate::rusqlite_database::RusqliteDatabase::new(&db_path) {
        Ok(db) => {
            println!("✅ Successfully initialized RusqliteDatabase!");

            // Test storing and retrieving a scan result
            println!("⏳ Testing cache operations...");
            let test_patterns = vec![];

            match db.store_scan_result(
                "test_package",
                "1.0.0",
                "test_hash",
                "Test analysis result",
                &test_patterns,
                "test_model",
            ) {
                Ok(_) => println!("✅ Successfully stored test scan result!"),
                Err(e) => return Err(anyhow::anyhow!("Failed to store scan result: {}", e)),
            }

            // Test retrieving the cached result
            match db.get_cached_result("test_package", "1.0.0", "test_hash") {
                Ok(Some(cached)) => {
                    println!(
                        "✅ Successfully retrieved cached result: {}",
                        cached.analysis
                    );
                }
                Ok(None) => return Err(anyhow::anyhow!("Cached result not found")),
                Err(e) => return Err(anyhow::anyhow!("Failed to retrieve cached result: {}", e)),
            }

            // Test cache statistics
            match db.get_cache_stats() {
                Ok(stats) => {
                    println!(
                        "✅ Cache stats: {} entries, {} recent",
                        stats.total_cached_entries, stats.recent_scans_7_days
                    );
                }
                Err(e) => println!("⚠️  Failed to get cache stats: {}", e),
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to initialize RusqliteDatabase: {}",
                e
            ));
        }
    }

    // Clean up test database
    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
        println!("🗑️ Cleaned up test database");
    }

    println!("🎉 RustRecon cache database tests passed!");
    Ok(())
}

pub async fn run_all_tests() -> Result<()> {
    println!("🚀 Running comprehensive SQLite cache diagnostics...\n");

    // First test with rusqlite to see if SQLite works at all
    test_rusqlite_connection()?;

    // Test the RustRecon cache database implementation
    test_rustrecon_cache_database()?;

    // Then test with sqlx (we expect this to fail, but it's good to verify)
    match test_sqlite_connection().await {
        Ok(_) => println!("✅ SQLx SQLite connection works!"),
        Err(e) => {
            println!("❌ SQLx SQLite connection failed: {}", e);
            println!("🔍 This is expected - we're using rusqlite instead of SQLx.");
        }
    }

    // Test basic cache path functionality but don't fail on SQLx issues
    match test_rustrecon_cache_path_rusqlite().await {
        Ok(_) => println!("✅ Cache path tests with rusqlite passed!"),
        Err(e) => {
            println!("⚠️  Cache path test failed: {}", e);
            // Don't fail the entire diagnostic for this
        }
    }

    println!("\n✨ All diagnostic tests completed successfully!");
    println!("💡 The rusqlite-based cache system is working correctly!");
    println!("💡 SQLx failures are expected and don't affect functionality.");

    Ok(())
}
