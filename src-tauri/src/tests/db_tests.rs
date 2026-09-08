use crate::commands::project::migrate_schema_impl;
use crate::db;
use rusqlite::Connection;

fn temp_path(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let mut p = std::env::temp_dir();
    p.push(format!("school_test_{}_{}.db", label, nanos));
    p
}

/// 버전 도입 이전(v0) 시절의 스키마.
/// 마이그레이션이 추가한 컬럼·테이블은 일부러 제외되어 있어야 한다.
/// 최신 schema.sql로 만든 DB에 user_version만 0으로 위조하면
/// ALTER TABLE이 기존 컬럼과 충돌하므로 마이그레이션 체인을 검증할 수 없다.
const LEGACY_V0_SCHEMA: &str = "
CREATE TABLE Student (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    grade INTEGER NOT NULL, class_num INTEGER NOT NULL, number INTEGER NOT NULL,
    name TEXT NOT NULL,
    UNIQUE (grade, class_num, number)
);
CREATE TABLE Area (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    byte_limit INTEGER NOT NULL CHECK (byte_limit > 0)
);
CREATE TABLE Activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE AreaActivity (
    area_id INTEGER NOT NULL, activity_id INTEGER NOT NULL,
    PRIMARY KEY (area_id, activity_id),
    FOREIGN KEY (area_id) REFERENCES Area (id) ON DELETE CASCADE,
    FOREIGN KEY (activity_id) REFERENCES Activity (id) ON DELETE CASCADE
);
CREATE TABLE AreaStudent (
    area_id INTEGER NOT NULL, student_id INTEGER NOT NULL,
    PRIMARY KEY (area_id, student_id),
    FOREIGN KEY (area_id) REFERENCES Area (id) ON DELETE CASCADE,
    FOREIGN KEY (student_id) REFERENCES Student (id) ON DELETE CASCADE
);
CREATE TABLE ActivityRecord (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL, student_id INTEGER NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (activity_id, student_id),
    FOREIGN KEY (activity_id) REFERENCES Activity (id) ON DELETE CASCADE,
    FOREIGN KEY (student_id) REFERENCES Student (id) ON DELETE CASCADE
);
CREATE TABLE ActivityRecordHistory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_record_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    changed_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (activity_record_id) REFERENCES ActivityRecord (id) ON DELETE CASCADE
);
CREATE TABLE Snapshot (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    memo TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE ReplaceRule (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    old_text TEXT NOT NULL, new_text TEXT NOT NULL,
    is_regex INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 0 CHECK (priority >= 0),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (old_text, new_text)
);
CREATE TABLE SynonymGroup (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE SynonymItem (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL, word TEXT NOT NULL,
    UNIQUE (group_id, word),
    FOREIGN KEY (group_id) REFERENCES SynonymGroup (id) ON DELETE CASCADE
);
CREATE TABLE APP_CONFIGS (
    config_key TEXT PRIMARY KEY,
    config_value TEXT NOT NULL
);
";

/// v0 스키마로 DB 파일을 만들고 user_version = 0으로 둔다.
fn create_legacy_v0_db(path: &std::path::Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(LEGACY_V0_SCHEMA).unwrap();
    conn.pragma_update(None, "user_version", 0u32).unwrap();
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    cols.iter().any(|c| c == column)
}

fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        rusqlite::params![name],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

#[test]
fn test_create_new_creates_required_tables() {
    let path = temp_path("create_tables");
    let conn = db::create_new(&path).unwrap();

    let required = [
        "Student",
        "Area",
        "Activity",
        "ActivityRecord",
        "ActivityRecordHistory",
        "Snapshot",
        "ReplaceRule",
        "SynonymGroup",
        "SynonymItem",
        "APP_CONFIGS",
    ];
    for table in &required {
        assert!(table_exists(&conn, table), "테이블 없음: {table}");
    }

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_new_sets_schema_version() {
    let path = temp_path("schema_version");
    let conn = db::create_new(&path).unwrap();

    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, db::SCHEMA_VERSION);

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_open_existing_same_version_ok() {
    let path = temp_path("open_existing");
    {
        let conn = db::create_new(&path).unwrap();
        drop(conn);
    }

    let result = db::open_existing(&path);
    assert!(result.is_ok(), "open_existing 실패: {:?}", result.err());

    let conn = result.unwrap();
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, db::SCHEMA_VERSION);

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_open_too_new_returns_error() {
    let path = temp_path("too_new");
    {
        let conn = db::create_new(&path).unwrap();
        // user_version을 앱 버전보다 높게 수동 설정
        conn.pragma_update(None, "user_version", db::SCHEMA_VERSION + 1)
            .unwrap();
        drop(conn);
    }

    let result = db::open_existing(&path);
    assert!(
        matches!(result, Err(db::OpenError::TooNew { .. })),
        "TooNew 에러 예상, 실제: {:?}",
        result.map(|_| ())
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_open_existing_does_not_auto_migrate() {
    // open_existing은 마이그레이션을 수행하지 않아야 함
    let path = temp_path("no_auto_migrate");
    {
        let conn = db::create_new(&path).unwrap();
        conn.pragma_update(None, "user_version", 0u32).unwrap();
        drop(conn);
    }

    let conn = db::open_existing(&path).unwrap();
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 0, "open_existing가 마이그레이션을 수행하면 안 됨");

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_migrate_schema_upgrades_to_current_version() {
    // v0 스키마에서 시작해 마이그레이션 체인 전체가 통과해야 함
    let path = temp_path("migrate_upgrade");
    create_legacy_v0_db(&path);

    let mut conn = db::open_existing(&path).unwrap();
    migrate_schema_impl(&mut conn).unwrap();

    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, db::SCHEMA_VERSION);

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_migrate_v0_adds_all_columns_and_tables() {
    // 각 마이그레이션이 추가하기로 한 컬럼·테이블이 실제로 생겼는지 검증
    let path = temp_path("migrate_shape");
    create_legacy_v0_db(&path);

    let mut conn = db::open_existing(&path).unwrap();
    migrate_schema_impl(&mut conn).unwrap();

    // 컬럼 추가 (v2~v9, v12)
    for (table, column) in [
        ("Area", "prompt"),            // v2
        ("Student", "tags"),           // v3
        ("Area", "role"),              // v4
        ("Student", "behavior"),       // v4
        ("Area", "behavior_items"),    // v5
        ("AreaActivity", "sort_order"),// v7
        ("ReplaceRule", "note"),       // v8
        ("Activity", "prompt"),        // v9
        ("Activity", "date_info"),     // v9
        ("CellPdfNote", "enabled"),    // v12
        ("CellPdfNote", "id"),         // v11 (복합 PK → id PK 재구성)
    ] {
        assert!(
            column_exists(&conn, table, column),
            "마이그레이션 후 {table}.{column} 컬럼이 없음"
        );
    }

    // 테이블 추가 (v2, v6, v10, v13)
    for t in [
        "ChatSession", "ChatMessage",   // v2
        "OcrSession", "OcrResult",      // v6
        "CellPdfNote",                  // v10
        "StudentAreaBehavior",          // v13
    ] {
        assert!(table_exists(&conn, t), "마이그레이션 후 {t} 테이블이 없음");
    }

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_migrate_v0_preserves_existing_data() {
    // 마이그레이션이 기존 데이터를 잃지 않아야 함
    let path = temp_path("migrate_data");
    create_legacy_v0_db(&path);

    {
        let conn = Connection::open(&path).unwrap();
        conn.execute(
            "INSERT INTO Student (grade, class_num, number, name) VALUES (2, 5, 1, '홍길동')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Area (name, byte_limit) VALUES ('자율활동', 1500)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO Activity (name) VALUES ('진로탐색')", [])
            .unwrap();
        conn.execute(
            "INSERT INTO ActivityRecord (activity_id, student_id, content) VALUES (1, 1, '기존 기록')",
            [],
        )
        .unwrap();
    }

    let mut conn = db::open_existing(&path).unwrap();
    migrate_schema_impl(&mut conn).unwrap();

    let name: String = conn
        .query_row("SELECT name FROM Student WHERE id = 1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "홍길동");

    let content: String = conn
        .query_row("SELECT content FROM ActivityRecord WHERE id = 1", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(content, "기존 기록");

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_migrate_schema_is_noop_when_already_current() {
    // 이미 최신 버전이면 migrate_schema_impl은 아무것도 하지 않음
    let path = temp_path("migrate_noop");
    let mut conn = db::create_new(&path).unwrap();
    migrate_schema_impl(&mut conn).unwrap();

    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, db::SCHEMA_VERSION);

    drop(conn);
    let _ = std::fs::remove_file(&path);
}
