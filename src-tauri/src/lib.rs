use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const INITIAL_MIGRATION: &str =
    include_str!("../../prisma/migrations/20260611000000_init/migration.sql");

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

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法定位应用数据目录: {error}"))?;
    std::fs::create_dir_all(&data_dir)
        .map_err(|error| format!("无法创建应用数据目录: {error}"))?;
    Ok(data_dir.join("items.db"))
}

fn open_database(app: &AppHandle) -> Result<Connection, String> {
    let connection =
        Connection::open(database_path(app)?).map_err(|error| format!("无法打开数据库: {error}"))?;
    connection
        .execute_batch(INITIAL_MIGRATION)
        .map_err(|error| format!("无法初始化数据库: {error}"))?;
    Ok(connection)
}

fn normalize_input(input: ItemInput) -> Result<(String, Option<String>), String> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("名称不能为空".to_string());
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
             FROM items
             ORDER BY id DESC",
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
            "UPDATE items
             SET name = ?1, description = ?2, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![name, description, id],
        )
        .map_err(|error| error.to_string())?;
    if affected == 0 {
        return Err("记录不存在或已被删除".to_string());
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
        return Err("记录不存在或已被删除".to_string());
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            open_database(app.handle()).map_err(std::io::Error::other)?;
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
            delete_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
