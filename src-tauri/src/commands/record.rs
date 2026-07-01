use crate::commands::crypto::resolve_data_key;
use crate::crypto::{maybe_decrypt, maybe_encrypt};
use crate::state::{CryptoStateHandle, DbState};
use crate::types::{
    ActivityItem, ActivityPreviewItem, AreaByteSummary, AreaGridData, BulkImportResult,
    HistoryEntry, ImportRecordInput, PreviewImportItem, RecordCell, StudentAreaPreview,
    StudentByteRow, StudentItem,
};
use rusqlite::{Connection, OptionalExtension};
use std::collections::HashMap;
use tauri::State;

pub fn get_area_grid_impl(
    conn: &Connection,
    area_id: i64,
    key: Option<[u8; 32]>,
) -> Result<AreaGridData, String> {
    let mut stmt = conn
        .prepare(
            "SELECT act.id, act.name, act.prompt, act.date_info
             FROM Activity act
             JOIN AreaActivity aa ON act.id = aa.activity_id
             WHERE aa.area_id = ?1
             ORDER BY aa.sort_order ASC, act.name ASC",
        )
        .map_err(|e| e.to_string())?;

    let activities = stmt
        .query_map(rusqlite::params![area_id], |row| {
            Ok(ActivityItem {
                id: row.get(0)?,
                name: row.get(1)?,
                prompt: row.get(2)?,
                date_info: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.grade, s.class_num, s.number, s.name, s.tags, s.behavior
             FROM Student s
             JOIN AreaStudent as_ ON s.id = as_.student_id
             WHERE as_.area_id = ?1
             ORDER BY s.grade, s.class_num, s.number",
        )
        .map_err(|e| e.to_string())?;

    let raw_students = stmt
        .query_map(rusqlite::params![area_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut students = Vec::with_capacity(raw_students.len());
    for (id, grade, class_num, number, name, tags_raw, behavior) in raw_students {
        let tags = match tags_raw {
            Some(s) if !s.is_empty() => {
                serde_json::from_str::<Vec<String>>(&s).unwrap_or_default()
            }
            _ => vec![],
        };
        students.push(StudentItem {
            id,
            grade,
            class_num,
            number,
            name: maybe_decrypt(name, key)?,
            tags,
            behavior,
        });
    }

    let activity_ids: Vec<i64> = activities.iter().map(|a| a.id).collect();
    let student_ids: Vec<i64> = students.iter().map(|s| s.id).collect();
    let records = if activity_ids.is_empty() || student_ids.is_empty() {
        vec![]
    } else {
        let act_placeholders = activity_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let stu_placeholders = student_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", activity_ids.len() + i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT activity_id, student_id, content
             FROM ActivityRecord
             WHERE activity_id IN ({})
               AND student_id IN ({})",
            act_placeholders, stu_placeholders
        );
        let params: Vec<i64> = activity_ids
            .iter()
            .chain(student_ids.iter())
            .copied()
            .collect();
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let raw_rows = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut rows = Vec::with_capacity(raw_rows.len());
        for (activity_id, student_id, content) in raw_rows {
            rows.push(RecordCell {
                activity_id,
                student_id,
                content: maybe_decrypt(content, key)?,
            });
        }
        rows
    };

    Ok(AreaGridData {
        activities,
        students,
        records,
    })
}

#[tauri::command]
pub fn get_area_grid(
    area_id: i64,
    state: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<AreaGridData, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let key = resolve_data_key(conn, &crypto)?;
    get_area_grid_impl(conn, area_id, key)
}

pub fn upsert_record_impl(
    conn: &Connection,
    activity_id: i64,
    student_id: i64,
    content: &str,
    key: Option<[u8; 32]>,
) -> Result<(), String> {
    let stored = maybe_encrypt(content, key)?;
    conn.execute(
        "INSERT INTO ActivityRecord (activity_id, student_id, content, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(activity_id, student_id) DO UPDATE SET
           content = excluded.content,
           updated_at = excluded.updated_at",
        rusqlite::params![activity_id, student_id, stored],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn upsert_record(
    activity_id: i64,
    student_id: i64,
    content: String,
    state: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let key = resolve_data_key(conn, &crypto)?;
    upsert_record_impl(conn, activity_id, student_id, &content, key)
}

pub fn get_record_history_impl(
    conn: &Connection,
    activity_id: i64,
    student_id: i64,
    limit: i64,
    offset: i64,
    key: Option<[u8; 32]>,
) -> Result<Vec<HistoryEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.content, h.changed_at, h.note
             FROM ActivityRecordHistory h
             JOIN ActivityRecord r ON r.id = h.activity_record_id
             WHERE r.activity_id = ?1 AND r.student_id = ?2
             ORDER BY h.changed_at DESC
             LIMIT ?3 OFFSET ?4",
        )
        .map_err(|e| e.to_string())?;

    let raw = stmt
        .query_map(
            rusqlite::params![activity_id, student_id, limit, offset],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::with_capacity(raw.len());
    for (id, content, changed_at, note) in raw {
        entries.push(HistoryEntry {
            id,
            content: maybe_decrypt(content, key)?,
            changed_at,
            note,
        });
    }
    Ok(entries)
}

#[tauri::command]
pub fn get_record_history(
    activity_id: i64,
    student_id: i64,
    limit: i64,
    offset: i64,
    state: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<Vec<HistoryEntry>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let key = resolve_data_key(conn, &crypto)?;
    get_record_history_impl(conn, activity_id, student_id, limit, offset, key)
}

pub fn save_snapshot_internal(
    conn: &Connection,
    activity_id: i64,
    student_id: i64,
    note: Option<&str>,
) -> Result<(), String> {
    let inserted = conn
        .execute(
            "INSERT INTO ActivityRecordHistory (activity_record_id, content, changed_at, note)
             SELECT r.id, r.content, r.updated_at, ?3
             FROM ActivityRecord r
             WHERE r.activity_id = ?1 AND r.student_id = ?2
               AND NOT EXISTS (
                   SELECT 1 FROM ActivityRecordHistory h
                   WHERE h.activity_record_id = r.id
                     AND h.changed_at = r.updated_at
               )",
            rusqlite::params![activity_id, student_id, note],
        )
        .map_err(|e| e.to_string())?;

    if inserted == 0 {
        conn.execute(
            "UPDATE ActivityRecordHistory SET note = ?3
             WHERE id = (
                 SELECT h.id FROM ActivityRecordHistory h
                 JOIN ActivityRecord r ON r.id = h.activity_record_id
                 WHERE r.activity_id = ?1 AND r.student_id = ?2
                   AND h.changed_at = r.updated_at
                 LIMIT 1
             )",
            rusqlite::params![activity_id, student_id, note],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_history_snapshot(
    activity_id: i64,
    student_id: i64,
    note: Option<String>,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    save_snapshot_internal(conn, activity_id, student_id, note.as_deref())
}

pub fn bulk_import_records_impl(
    conn: &Connection,
    records: &[ImportRecordInput],
    key: Option<[u8; 32]>,
) -> Result<BulkImportResult, String> {
    let mut students_created: i64 = 0;
    let mut students_updated: i64 = 0;
    let mut records_saved: i64 = 0;
    let mut student_cache: HashMap<(i64, i64, i64), i64> = HashMap::new();

    for r in records.iter() {
        let cache_key = (r.grade, r.class_num, r.number);

        if !student_cache.contains_key(&cache_key) {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM Student WHERE grade=?1 AND class_num=?2 AND number=?3",
                    rusqlite::params![r.grade, r.class_num, r.number],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|e| e.to_string())?
                > 0;

            if exists {
                if let Some(ref n) = r.name {
                    if !n.is_empty() {
                        let existing_name_raw: String = conn
                            .query_row(
                                "SELECT name FROM Student WHERE grade=?1 AND class_num=?2 AND number=?3",
                                rusqlite::params![r.grade, r.class_num, r.number],
                                |row| row.get(0),
                            )
                            .map_err(|e| e.to_string())?;
                        let existing_name = maybe_decrypt(existing_name_raw, key)?;
                        if existing_name.trim().is_empty() {
                            let stored_name = maybe_encrypt(n, key)?;
                            conn.execute(
                                "UPDATE Student SET name = ?1 WHERE grade=?2 AND class_num=?3 AND number=?4",
                                rusqlite::params![stored_name, r.grade, r.class_num, r.number],
                            )
                            .map_err(|e| e.to_string())?;
                        }
                    }
                }
                students_updated += 1;
            } else {
                let name = r.name.as_deref().unwrap_or("이름 없음");
                let stored_name = maybe_encrypt(name, key)?;
                conn.execute(
                    "INSERT INTO Student (grade, class_num, number, name) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![r.grade, r.class_num, r.number, stored_name],
                )
                .map_err(|e| e.to_string())?;
                students_created += 1;
            }

            let student_id: i64 = conn
                .query_row(
                    "SELECT id FROM Student WHERE grade=?1 AND class_num=?2 AND number=?3",
                    rusqlite::params![r.grade, r.class_num, r.number],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;

            student_cache.insert(cache_key, student_id);
        }

        let &student_id = student_cache
            .get(&cache_key)
            .ok_or_else(|| "캐시 오류".to_string())?;
        let stored_content = maybe_encrypt(&r.content, key)?;

        conn.execute(
            "INSERT INTO ActivityRecord (activity_id, student_id, content, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))
             ON CONFLICT(activity_id, student_id) DO UPDATE SET
               content = excluded.content,
               updated_at = excluded.updated_at",
            rusqlite::params![r.activity_id, student_id, stored_content],
        )
        .map_err(|e| e.to_string())?;

        if !r.content.is_empty() {
            conn.execute(
                "INSERT INTO ActivityRecordHistory (activity_record_id, content, changed_at, note)
                 SELECT r.id, r.content, r.updated_at, 'import'
                 FROM ActivityRecord r
                 WHERE r.activity_id = ?1 AND r.student_id = ?2
                   AND NOT EXISTS (
                       SELECT 1 FROM ActivityRecordHistory h
                       WHERE h.activity_record_id = r.id
                         AND h.changed_at = r.updated_at
                   )",
                rusqlite::params![r.activity_id, student_id],
            )
            .map_err(|e| e.to_string())?;
        }
        records_saved += 1;
    }

    Ok(BulkImportResult {
        students_created,
        students_updated,
        records_saved,
    })
}

#[tauri::command]
pub fn bulk_import_records(
    records: Vec<ImportRecordInput>,
    state: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<BulkImportResult, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let key = resolve_data_key(conn, &crypto)?;

    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
    match bulk_import_records_impl(conn, &records, key) {
        Ok(r) => {
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
            Ok(r)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

pub fn preview_import_records_impl(
    conn: &Connection,
    records: &[ImportRecordInput],
    key: Option<[u8; 32]>,
) -> Result<Vec<PreviewImportItem>, String> {
    let mut result = Vec::new();
    let mut student_cache: HashMap<(i64, i64, i64), Option<(i64, String)>> = HashMap::new();
    let mut activity_cache: HashMap<i64, String> = HashMap::new();

    for r in records.iter() {
        let activity_name = if let Some(name) = activity_cache.get(&r.activity_id) {
            name.clone()
        } else {
            let name: Option<String> = conn
                .query_row(
                    "SELECT name FROM Activity WHERE id = ?1",
                    rusqlite::params![r.activity_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            let name = name.unwrap_or_else(|| format!("활동 #{}", r.activity_id));
            activity_cache.insert(r.activity_id, name.clone());
            name
        };

        let cache_key = (r.grade, r.class_num, r.number);
        let student_info = if let Some(cached) = student_cache.get(&cache_key) {
            cached.clone()
        } else {
            let info: Option<(i64, String)> = conn
                .query_row(
                    "SELECT id, name FROM Student WHERE grade=?1 AND class_num=?2 AND number=?3",
                    rusqlite::params![r.grade, r.class_num, r.number],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            let info = info
                .map(|(id, enc_name)| maybe_decrypt(enc_name, key).map(|name| (id, name)))
                .transpose()?;
            student_cache.insert(cache_key, info.clone());
            info
        };

        let (student_name, existing_content) = match student_info {
            Some((student_id, name)) => {
                let content: Option<String> = conn
                    .query_row(
                        "SELECT content FROM ActivityRecord WHERE activity_id=?1 AND student_id=?2",
                        rusqlite::params![r.activity_id, student_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|e| e.to_string())?;
                let plain_content = match content {
                    Some(c) => maybe_decrypt(c, key)?,
                    None => String::new(),
                };
                (name, plain_content)
            }
            None => {
                let name = r.name.as_deref().unwrap_or("이름 없음").to_string();
                (name, String::new())
            }
        };

        result.push(PreviewImportItem {
            grade: r.grade,
            class_num: r.class_num,
            number: r.number,
            student_name,
            activity_id: r.activity_id,
            activity_name,
            new_content: r.content.clone(),
            existing_content,
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn preview_import_records(
    records: Vec<ImportRecordInput>,
    state: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<Vec<PreviewImportItem>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let key = resolve_data_key(conn, &crypto)?;
    preview_import_records_impl(conn, &records, key)
}

// UTF-8 바이트 수 계산 (엔터 \n → \r\n 으로 처리, 프론트엔드와 동일 로직)
fn byte_length(s: &str) -> i64 {
    let normalized = s.replace('\r', "").replace('\n', "\r\n");
    normalized.len() as i64
}

#[tauri::command]
pub fn get_student_full_preview(
    student_id: i64,
    state: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<Vec<StudentAreaPreview>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let key = resolve_data_key(conn, &crypto)?;

    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.name, a.byte_limit,
                    act.id, act.name,
                    COALESCE(ar.content, '') AS content
             FROM Area a
             JOIN AreaStudent as_ ON a.id = as_.area_id AND as_.student_id = ?1
             JOIN AreaActivity aa ON a.id = aa.area_id
             JOIN Activity act ON aa.activity_id = act.id
             LEFT JOIN ActivityRecord ar ON ar.activity_id = act.id AND ar.student_id = ?1
             ORDER BY a.id, aa.sort_order ASC, act.name ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![student_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut areas: Vec<StudentAreaPreview> = Vec::new();
    let mut area_index: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();

    for (area_id, area_name, byte_limit, _act_id, act_name, content_raw) in rows {
        let content = maybe_decrypt(content_raw, key)?;
        let idx = if let Some(&i) = area_index.get(&area_id) {
            i
        } else {
            let i = areas.len();
            areas.push(StudentAreaPreview {
                area_id,
                area_name,
                byte_limit,
                total_bytes: 0,
                activities: Vec::new(),
            });
            area_index.insert(area_id, i);
            i
        };
        let b = byte_length(&content);
        areas[idx].total_bytes += b;
        areas[idx].activities.push(ActivityPreviewItem {
            activity_name: act_name,
            content,
        });
    }

    Ok(areas)
}

#[tauri::command]
pub fn get_all_areas_byte_summary(
    state: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<Vec<StudentByteRow>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let key = resolve_data_key(conn, &crypto)?;

    // 학생 × 영역 × 활동 레코드를 한 번에 조회
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.grade, s.class_num, s.number, s.name,
                    a.id, a.name, a.byte_limit,
                    COALESCE(ar.content, '') AS content
             FROM Student s
             JOIN AreaStudent as_ ON s.id = as_.student_id
             JOIN Area a ON a.id = as_.area_id
             JOIN AreaActivity aa ON aa.area_id = a.id
             JOIN Activity act ON act.id = aa.activity_id
             LEFT JOIN ActivityRecord ar ON ar.activity_id = act.id AND ar.student_id = s.id
             ORDER BY s.grade, s.class_num, s.number, a.id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut students: Vec<StudentByteRow> = Vec::new();
    let mut student_index: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    // (student_id, area_id) → index in student.areas
    let mut area_index: std::collections::HashMap<(i64, i64), usize> = std::collections::HashMap::new();

    for (s_id, grade, class_num, number, name_raw, a_id, a_name, byte_limit, content_raw) in rows {
        let name = maybe_decrypt(name_raw, key)?;
        let content = maybe_decrypt(content_raw, key)?;
        let b = byte_length(&content);

        let s_idx = if let Some(&i) = student_index.get(&s_id) {
            i
        } else {
            let i = students.len();
            students.push(StudentByteRow {
                student_id: s_id,
                grade,
                class_num,
                number,
                name,
                areas: Vec::new(),
            });
            student_index.insert(s_id, i);
            i
        };

        let area_key = (s_id, a_id);
        if let Some(&a_idx) = area_index.get(&area_key) {
            students[s_idx].areas[a_idx].total_bytes += b;
        } else {
            let a_idx = students[s_idx].areas.len();
            students[s_idx].areas.push(AreaByteSummary {
                area_id: a_id,
                area_name: a_name,
                byte_limit,
                total_bytes: b,
            });
            area_index.insert(area_key, a_idx);
        }
    }

    Ok(students)
}
