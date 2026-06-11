use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "20260611000000_init",
        include_str!("../../prisma/migrations/20260611000000_init/migration.sql"),
    ),
    (
        "20260612000000_resource_bundles",
        include_str!("../../prisma/migrations/20260612000000_resource_bundles/migration.sql"),
    ),
];
const RESOURCE_BUNDLE_KEY: &str = "starter-pack";
const RESOURCE_BUNDLE_VERSION: &str = "1.0.0";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    id: i64,
    name: String,
    description: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemInput {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceBundleInfo {
    key: String,
    version: String,
    local_path: String,
    installed_files: usize,
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Cannot locate the application data directory: {error}"))?;
    fs::create_dir_all(&data_dir)
        .map_err(|error| format!("Cannot create the application data directory: {error}"))?;
    Ok(data_dir)
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("items.db"))
}

fn run_migrations(connection: &mut Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS _app_migrations (
                version TEXT NOT NULL PRIMARY KEY,
                applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );",
        )
        .map_err(|error| format!("Cannot create the migration history: {error}"))?;

    for (version, sql) in MIGRATIONS {
        let applied = connection
            .query_row(
                "SELECT 1 FROM _app_migrations WHERE version = ?1",
                [version],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| format!("Cannot inspect migration {version}: {error}"))?
            .is_some();
        if applied {
            continue;
        }

        let transaction = connection
            .transaction()
            .map_err(|error| format!("Cannot start migration {version}: {error}"))?;
        transaction
            .execute_batch(sql)
            .map_err(|error| format!("Cannot apply migration {version}: {error}"))?;
        transaction
            .execute(
                "INSERT INTO _app_migrations (version) VALUES (?1)",
                [version],
            )
            .map_err(|error| format!("Cannot record migration {version}: {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Cannot commit migration {version}: {error}"))?;
    }

    Ok(())
}

fn open_database(app: &AppHandle) -> Result<Connection, String> {
    let mut connection = Connection::open(database_path(app)?)
        .map_err(|error| format!("Cannot open the database: {error}"))?;
    run_migrations(&mut connection)?;
    Ok(connection)
}

fn bundled_resource_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let packaged = app
        .path()
        .resource_dir()
        .map_err(|error| format!("Cannot locate bundled resources: {error}"))?
        .join("resources")
        .join(RESOURCE_BUNDLE_KEY);
    if packaged.exists() {
        return Ok(packaged);
    }

    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join(RESOURCE_BUNDLE_KEY);
    if development.exists() {
        return Ok(development);
    }

    Err("The starter resource bundle is missing from the application package".to_string())
}

fn copy_missing_files(source: &Path, destination: &Path) -> Result<usize, String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("Cannot create resource directory: {error}"))?;
    let mut copied = 0;

    for entry in fs::read_dir(source).map_err(|error| format!("Cannot read resources: {error}"))? {
        let entry = entry.map_err(|error| format!("Cannot read a resource entry: {error}"))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copied += copy_missing_files(&source_path, &destination_path)?;
        } else if !destination_path.exists() {
            fs::copy(&source_path, &destination_path)
                .map_err(|error| format!("Cannot install resource {:?}: {error}", source_path))?;
            copied += 1;
        }
    }

    Ok(copied)
}

fn install_resource_bundle(app: &AppHandle, connection: &Connection) -> Result<usize, String> {
    let source = bundled_resource_dir(app)?;
    let destination = app_data_dir(app)?
        .join("resources")
        .join(RESOURCE_BUNDLE_KEY);
    let copied = copy_missing_files(&source, &destination)?;

    connection
        .execute(
            "INSERT INTO app_resource_bundles (key, version, installed_at)
             VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET
               version = excluded.version,
               installed_at = CURRENT_TIMESTAMP",
            params![RESOURCE_BUNDLE_KEY, RESOURCE_BUNDLE_VERSION],
        )
        .map_err(|error| format!("Cannot record the resource bundle version: {error}"))?;
    Ok(copied)
}

fn count_files(path: &Path) -> Result<usize, String> {
    let mut count = 0;
    for entry in
        fs::read_dir(path).map_err(|error| format!("Cannot read local resources: {error}"))?
    {
        let entry = entry.map_err(|error| format!("Cannot inspect a local resource: {error}"))?;
        if entry.path().is_dir() {
            count += count_files(&entry.path())?;
        } else {
            count += 1;
        }
    }
    Ok(count)
}

fn normalize_input(input: ItemInput) -> Result<(String, Option<String>), String> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("Name is required".to_string());
    }
    let description = input
        .description
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    Ok((name, description))
}

fn map_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[tauri::command]
fn list_items(app: AppHandle) -> Result<Vec<Item>, String> {
    let connection = open_database(&app)?;
    let mut statement = connection
        .prepare(
            "SELECT id, name, description, created_at, updated_at
             FROM items ORDER BY id DESC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], map_item)
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_item(app: AppHandle, input: ItemInput) -> Result<Item, String> {
    let (name, description) = normalize_input(input)?;
    let connection = open_database(&app)?;
    connection
        .execute(
            "INSERT INTO items (name, description) VALUES (?1, ?2)",
            params![name, description],
        )
        .map_err(|error| error.to_string())?;
    let id = connection.last_insert_rowid();
    connection
        .query_row(
            "SELECT id, name, description, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            map_item,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn update_item(app: AppHandle, id: i64, input: ItemInput) -> Result<Item, String> {
    let (name, description) = normalize_input(input)?;
    let connection = open_database(&app)?;
    let affected = connection
        .execute(
            "UPDATE items SET name = ?1, description = ?2,
             updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            params![name, description, id],
        )
        .map_err(|error| error.to_string())?;
    if affected == 0 {
        return Err("The item no longer exists".to_string());
    }
    connection
        .query_row(
            "SELECT id, name, description, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            map_item,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_item(app: AppHandle, id: i64) -> Result<(), String> {
    let connection = open_database(&app)?;
    let affected = connection
        .execute("DELETE FROM items WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    if affected == 0 {
        return Err("The item no longer exists".to_string());
    }
    Ok(())
}

#[tauri::command]
fn get_resource_bundle_info(app: AppHandle) -> Result<ResourceBundleInfo, String> {
    let local_path = app_data_dir(&app)?
        .join("resources")
        .join(RESOURCE_BUNDLE_KEY);
    Ok(ResourceBundleInfo {
        key: RESOURCE_BUNDLE_KEY.to_string(),
        version: RESOURCE_BUNDLE_VERSION.to_string(),
        installed_files: count_files(&local_path)?,
        local_path: local_path.to_string_lossy().into_owned(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let connection = open_database(app.handle()).map_err(std::io::Error::other)?;
            install_resource_bundle(app.handle(), &connection).map_err(std::io::Error::other)?;
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_items,
            create_item,
            update_item,
            delete_item,
            get_resource_bundle_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running ListNest");
}
