use crate::state::{DbState, unique_err};
use crate::types::{ActivityItem, AreaItem};
use rusqlite::Connection;
use std::collections::HashMap;
use tauri::State;

// 기본 영역 목록: (이름, byte_limit, role)
const DEFAULT_AREAS: &[(&str, i64, &str)] = &[
    ("자율활동", 1500, "homeroom"),
    ("진로활동", 1500, "homeroom"),
    ("창체동아리", 500, "subject"),
    ("자율동아리", 500, "subject"),
    ("과목별세부능력특기사항", 1500, "subject"),
    ("행동특성및종합의견", 1000, "homeroom"),
];

// 담임 전용 영역
const HOMEROOM_AREAS: &[(&str, i64, &str)] = &[
    ("자율활동", 1500, "homeroom"),
    ("진로활동", 1500, "homeroom"),
    ("행동특성및종합의견", 1000, "homeroom"),
];

// 교과교사(비담임) 전용 영역
const SUBJECT_AREAS: &[(&str, i64, &str)] = &[
    ("창체동아리", 500, "subject"),
    ("자율동아리", 500, "subject"),
    ("과목별세부능력특기사항", 1500, "subject"),
];

pub fn get_areas_impl(conn: &Connection) -> Result<Vec<AreaItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.name, a.byte_limit, a.prompt, a.role, a.behavior_items,
                    act.id AS act_id, act.name AS act_name
             FROM Area a
             LEFT JOIN AreaActivity aa ON a.id = aa.area_id
             LEFT JOIN Activity act ON aa.activity_id = act.id
             ORDER BY a.id, aa.sort_order ASC, act.name ASC",
        )
        .map_err(|e| e.to_string())?;

    let mut areas: Vec<AreaItem> = Vec::new();
    let mut index_map: HashMap<i64, usize> = HashMap::new();

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (area_id, area_name, byte_limit, prompt, role, behavior_items, act_id, act_name) =
            row.map_err(|e| e.to_string())?;

        let idx = if let Some(&i) = index_map.get(&area_id) {
            i
        } else {
            let i = areas.len();
            areas.push(AreaItem {
                id: area_id,
                name: area_name,
                byte_limit,
                prompt,
                role,
                behavior_items,
                activities: vec![],
            });
            index_map.insert(area_id, i);
            i
        };

        if let (Some(id), Some(name)) = (act_id, act_name) {
            areas[idx].activities.push(ActivityItem { id, name });
        }
    }

    Ok(areas)
}

#[tauri::command]
pub fn set_area_behavior_items(
    area_id: i64,
    items: Option<String>,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    conn.execute(
        "UPDATE Area SET behavior_items = ?1 WHERE id = ?2",
        rusqlite::params![items, area_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn create_area_impl(
    conn: &Connection,
    name: &str,
    byte_limit: i64,
    prompt: Option<&str>,
    role: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO Area (name, byte_limit, prompt, role) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, byte_limit, prompt, role],
    )
    .map_err(|e| unique_err(&e, &format!("이미 같은 이름의 영역이 있습니다: {name}")))?;

    Ok(conn.last_insert_rowid())
}

pub fn update_area_impl(
    conn: &Connection,
    id: i64,
    name: &str,
    byte_limit: i64,
    prompt: Option<&str>,
    role: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE Area SET name = ?1, byte_limit = ?2, prompt = ?3, role = ?4 WHERE id = ?5",
        rusqlite::params![name, byte_limit, prompt, role, id],
    )
    .map_err(|e| unique_err(&e, &format!("이미 같은 이름의 영역이 있습니다: {name}")))?;

    Ok(())
}

pub fn delete_area_impl(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM Area WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Tauri 커맨드 ────────────────────────────────────────────────

#[tauri::command]
pub fn get_areas(state: State<DbState>) -> Result<Vec<AreaItem>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    get_areas_impl(conn)
}

#[tauri::command]
pub fn create_area(
    name: String,
    byte_limit: i64,
    prompt: Option<String>,
    role: Option<String>,
    state: State<DbState>,
) -> Result<i64, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let r = role.as_deref().unwrap_or("common");
    create_area_impl(conn, &name, byte_limit, prompt.as_deref(), r)
}

#[tauri::command]
pub fn update_area(
    id: i64,
    name: String,
    byte_limit: i64,
    prompt: Option<String>,
    role: Option<String>,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let r = role.as_deref().unwrap_or("common");
    update_area_impl(conn, id, &name, byte_limit, prompt.as_deref(), r)
}

#[tauri::command]
pub fn delete_area(id: i64, state: State<DbState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    delete_area_impl(conn, id)
}

fn seed_areas_list(conn: &rusqlite::Connection, areas: &[(&str, i64, &str)]) -> Result<Vec<String>, String> {
    let mut added: Vec<String> = Vec::new();
    for &(name, byte_limit, role) in areas {
        let result = conn.execute(
            "INSERT OR IGNORE INTO Area (name, byte_limit, role) VALUES (?1, ?2, ?3)",
            rusqlite::params![name, byte_limit, role],
        );
        match result {
            Ok(1) => added.push(name.to_string()),
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(added)
}

/// 역할별 기본 영역 추가 (is_homeroom: true = 담임, false = 교과교사)
#[tauri::command]
pub fn seed_areas_by_role(is_homeroom: bool, state: State<DbState>) -> Result<Vec<String>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let areas = if is_homeroom { HOMEROOM_AREAS } else { SUBJECT_AREAS };
    seed_areas_list(conn, areas)
}

/// 기본 영역이 없을 때만 삽입 (이미 있으면 skip)
#[tauri::command]
pub fn seed_default_areas(state: State<DbState>) -> Result<Vec<String>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    seed_areas_list(conn, DEFAULT_AREAS)
}
