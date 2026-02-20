use std::path::Path;

/// Generates a seeder file at the given path.
///
/// # Arguments
/// * `name` - Name of the seeder (e.g. "UserSeeder")
/// * `target_dir` - Directory where the seeder file will be created
///
/// # Returns
/// The path to the created file, or error if file already exists.
pub async fn make_seeder(name: &str, target_dir: &Path) -> anyhow::Result<std::path::PathBuf> {
    // 1. Sanitize name (ensure ends with Seeder)
    let struct_name = if name.ends_with("Seeder") {
        name.to_string()
    } else {
        format!("{}Seeder", name)
    };

    // 2. Determine file name (snake_case)
    let file_name = camel_to_snake(&struct_name);

    // 3. Target Path
    let target = target_dir.join(format!("{}.rs", file_name));

    if target.exists() {
        anyhow::bail!("Seeder already exists: {:?}", target);
    }

    // 4. Content
    let content = format!(
        r#"use async_trait::async_trait;
use core_db::seeder::Seeder;

pub struct {struct_name};

#[async_trait]
impl Seeder for {struct_name} {{
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {{
        tracing::info!("Seeding {struct_name}...");
        // Logic here
        // let conn = core_db::common::sql::DbConn::pool(db);
        Ok(())
    }}
}}
"#,
        struct_name = struct_name
    );

    // 5. Ensure directory exists
    if !target_dir.exists() {
        tokio::fs::create_dir_all(target_dir).await?;
    }

    // 6. Write
    tokio::fs::write(&target, content).await?;

    Ok(target)
}

fn camel_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}
