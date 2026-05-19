use rusqlite::{Connection, Result};
use std::io::Write;
use std::path::Path;

/// 현재 앱이 지원하는 스키마 버전.
/// 스키마 변경 시 이 값을 올리고 MIGRATIONS 배열에 SQL을 추가한다.
/// 중요: 스키마 버전을 올릴 때는 반드시 Cargo.toml의 version(app_version)도 함께 올려야 한다.
/// app_version이 바뀌지 않으면 릴리즈 노트 모달이 표시되지 않는다.
pub const SCHEMA_VERSION: u32 = 5;

/// 인덱스 i: 버전 i → i+1 로 올리는 SQL.
/// [0] v0→v1: 버전 도입 이전 DB를 v1으로 승격. 스키마는 IF NOT EXISTS로 생성되어 있으므로 SQL 없음.
/// [1] v1→v2: Area에 prompt 컬럼 추가 + AI 대화 테이블(ChatSession, ChatMessage) 추가.
const MIGRATIONS: &[&str] = &[
    "", // v0 → v1
    // v1 → v2: AI 기능 추가
    "ALTER TABLE Area ADD COLUMN prompt TEXT;
     CREATE TABLE IF NOT EXISTS ChatSession (
         id         INTEGER PRIMARY KEY AUTOINCREMENT,
         title      TEXT NOT NULL DEFAULT '새 대화',
         model      TEXT NOT NULL DEFAULT 'anthropic/claude-haiku-4-5',
         created_at TEXT NOT NULL DEFAULT (datetime('now')),
         updated_at TEXT NOT NULL DEFAULT (datetime('now'))
     );
     CREATE TABLE IF NOT EXISTS ChatMessage (
         id                INTEGER PRIMARY KEY AUTOINCREMENT,
         session_id        INTEGER NOT NULL,
         role              TEXT    NOT NULL,
         content           TEXT    NOT NULL,
         model             TEXT,
         prompt_tokens     INTEGER NOT NULL DEFAULT 0,
         completion_tokens INTEGER NOT NULL DEFAULT 0,
         total_tokens      INTEGER NOT NULL DEFAULT 0,
         created_at        TEXT    NOT NULL DEFAULT (datetime('now')),
         FOREIGN KEY (session_id) REFERENCES ChatSession (id) ON DELETE CASCADE
     );
     CREATE INDEX IF NOT EXISTS idx_chat_message_session ON ChatMessage (session_id, created_at);",
    // v2 → v3: 학생 특성 태그 컬럼 추가
    "ALTER TABLE Student ADD COLUMN tags TEXT;",
    // v3 → v4: 영역 역할 컬럼 + 학생 행동 프로필 컬럼 추가
    "ALTER TABLE Area ADD COLUMN role TEXT NOT NULL DEFAULT 'common';
     ALTER TABLE Student ADD COLUMN behavior TEXT;",
    // v4 → v5: 영역별 행동 항목 커스텀 설정 컬럼 추가
    "ALTER TABLE Area ADD COLUMN behavior_items TEXT;",
];

// ── 내부 헬퍼 ────────────────────────────────────────────────

fn get_version(conn: &Connection) -> Result<u32> {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
}

/// 현재 버전에서 SCHEMA_VERSION까지 마이그레이션을 단계별로 실행한다.
/// - 각 단계는 rusqlite Transaction으로 감싸 실패 시 자동 ROLLBACK된다.
/// - foreign_keys는 트랜잭션 외부에서만 변경 가능하므로, IIFE 종료 후 복구한다.
/// - 각 단계 커밋 전 PRAGMA foreign_key_check로 무결성을 검증한다.
pub fn migrate(conn: &mut Connection, from: u32) -> Result<()> {
    // foreign_keys 변경은 트랜잭션 외부에서만 유효 (SQLite 공식 권고)
    conn.execute_batch("PRAGMA foreign_keys = OFF;")?;

    // IIFE로 마이그레이션 실행 — 성공·실패 모두 이후 foreign_keys = ON 복구 보장
    let result: Result<()> = (|| {
        for v in from..SCHEMA_VERSION {
            let idx = v as usize;
            let sql = MIGRATIONS.get(idx).copied().ok_or_else(|| {
                rusqlite::Error::InvalidParameterName(
                    format!("마이그레이션 스크립트 누락: v{v} → v{}", v + 1),
                )
            })?;

            // Transaction: 스코프 이탈(에러 포함) 시 자동 ROLLBACK
            let tx = conn.transaction()?;

            if !sql.is_empty() {
                tx.execute_batch(sql)?;
            }

            // user_version을 pragma_update API로 설정 (format! 없이 안전하게)
            tx.pragma_update(None, "user_version", v + 1)?;

            // 커밋 전 외래키 무결성 검증 — 위반 행이 하나라도 있으면 롤백
            {
                let mut stmt = tx.prepare("PRAGMA foreign_key_check;")?;
                if stmt.exists([])? {
                    return Err(rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                        Some(format!(
                            "v{v} → v{} 마이그레이션 후 외래키 무결성 위반",
                            v + 1
                        )),
                    ));
                }
            }

            tx.commit()?;
        }
        Ok(())
    })();

    // 트랜잭션이 모두 닫힌 후 복구 — 열린 트랜잭션이 없으므로 PRAGMA가 반드시 적용됨
    // 복구 실패 시 conn이 foreign_keys = OFF 상태로 남으므로 에러로 처리
    let fk_result = conn.execute_batch("PRAGMA foreign_keys = ON;");

    // 마이그레이션 에러 우선, 복구 에러는 마이그레이션 성공 시에만 반환
    result.and(fk_result)
}

// ── 공개 API ─────────────────────────────────────────────────

/// 새 DB 파일 생성 후 스키마 초기화 및 버전 기록
pub fn create_new(db_path: &Path) -> Result<Connection> {
    let mut conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    {
        let tx = conn.transaction()?;
        tx.execute_batch(include_str!("schema.sql"))?;
        tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        tx.commit()?;
    }
    Ok(conn)
}

/// 기존 DB 파일 열기 — 버전 검사만 수행 (마이그레이션은 migrate_schema 커맨드에서 별도 실행)
pub fn open_existing(db_path: &Path) -> Result<Connection, OpenError> {
    let conn = Connection::open(db_path).map_err(OpenError::Db)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;").map_err(OpenError::Db)?;

    let db_version = get_version(&conn).map_err(OpenError::Db)?;

    if db_version > SCHEMA_VERSION {
        return Err(OpenError::TooNew { db_version, app_version: SCHEMA_VERSION });
    }

    Ok(conn)
}

/// 앱 전역 설정 DB (프로젝트와 무관하게 %APPDATA% 에 보관)
/// API 키, AI 모델, 시스템 프롬프트, 참고 자료 설정 등 저장
pub fn open_global_config(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS APP_CONFIGS (
            config_key   TEXT PRIMARY KEY,
            config_value TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

/// DB 파일을 SQL 텍스트 덤프로 백업한다.
/// - UTF-8 BOM 포함 → Windows 메모장에서 한글 정상 표시
/// - 단일 파일에 덮어쓰기 방식 (날짜 없음)
pub fn dump_to_sql_file(src: &Path, dst: &Path) -> Result<(), String> {
    let conn = Connection::open(src).map_err(|e| e.to_string())?;

    let file = std::fs::File::create(dst).map_err(|e| e.to_string())?;
    let mut out = std::io::BufWriter::new(file);

    // UTF-8 BOM — Windows 메모장이 UTF-8로 인식하게 함
    out.write_all(b"\xEF\xBB\xBF").map_err(|e| e.to_string())?;

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(out, "-- School Record App 2.0 백업").map_err(|e| e.to_string())?;
    writeln!(out, "-- 생성 시각: {now}").map_err(|e| e.to_string())?;
    writeln!(out).map_err(|e| e.to_string())?;
    writeln!(out, "PRAGMA foreign_keys = OFF;").map_err(|e| e.to_string())?;
    writeln!(out, "BEGIN TRANSACTION;").map_err(|e| e.to_string())?;

    // 테이블 목록(DDL 포함) 수집
    let mut stmt = conn
        .prepare(
            "SELECT name, sql FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
             ORDER BY rowid",
        )
        .map_err(|e| e.to_string())?;

    let tables: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (table, ddl) in &tables {
        writeln!(out, "\n-- 테이블: {table}").map_err(|e| e.to_string())?;
        writeln!(out, "DROP TABLE IF EXISTS \"{table}\";").map_err(|e| e.to_string())?;
        writeln!(out, "{ddl};").map_err(|e| e.to_string())?;

        let mut data_stmt = conn
            .prepare(&format!("SELECT * FROM \"{table}\""))
            .map_err(|e| e.to_string())?;

        let col_count = data_stmt.column_count();
        let col_names: Vec<String> = (0..col_count)
            .map(|i| data_stmt.column_name(i).unwrap_or("col").to_string())
            .collect();
        let cols = col_names
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");

        let mut rows = data_stmt.query([]).map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let vals: Vec<String> = (0..col_count)
                .map(|i| {
                    use rusqlite::types::ValueRef;
                    match row.get_ref(i) {
                        Ok(ValueRef::Null) => "NULL".into(),
                        Ok(ValueRef::Integer(n)) => n.to_string(),
                        Ok(ValueRef::Real(f)) => format!("{f}"),
                        Ok(ValueRef::Text(b)) => {
                            let s = std::str::from_utf8(b).unwrap_or("");
                            format!("'{}'", s.replace('\'', "''"))
                        }
                        Ok(ValueRef::Blob(b)) => {
                            let hex: String =
                                b.iter().map(|x| format!("{x:02X}")).collect();
                            format!("X'{hex}'")
                        }
                        Err(_) => "NULL".into(),
                    }
                })
                .collect();

            writeln!(out, "INSERT INTO \"{table}\" ({cols}) VALUES ({});", vals.join(", "))
                .map_err(|e| e.to_string())?;
        }
    }

    writeln!(out, "\nCOMMIT;").map_err(|e| e.to_string())?;
    writeln!(out, "PRAGMA foreign_keys = ON;").map_err(|e| e.to_string())?;

    out.flush().map_err(|e| e.to_string())?;
    Ok(())
}

// ── 오류 타입 ─────────────────────────────────────────────────

#[derive(Debug)]
pub enum OpenError {
    Db(rusqlite::Error),
    /// DB 파일이 현재 앱보다 상위 버전
    TooNew { db_version: u32, app_version: u32 },
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::Db(e) => write!(f, "데이터베이스 오류: {e}"),
            OpenError::TooNew { db_version, app_version } => write!(
                f,
                "이 파일은 더 최신 버전의 앱에서 생성되었습니다. \
                 앱을 업데이트해주세요. (파일 버전: v{db_version}, 현재 앱: v{app_version})"
            ),
        }
    }
}
