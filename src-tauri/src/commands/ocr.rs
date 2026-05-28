use crate::state::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;
use tauri::State;

// ── 사이드카 프로세스 관리 ────────────────────────────────────────

pub struct OcrProcess {
    stdin:  std::io::BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    _child: Child,
}

impl OcrProcess {
    fn spawn(sidecar_script: &str) -> Result<Self, String> {
        let mut child = Command::new("python")
            .args(["-X", "utf8", sidecar_script])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("OCR 사이드카 실행 실패: {e}"))?;

        let stdin  = child.stdin.take()
            .ok_or("stdin 획득 실패")?;
        let stdout = child.stdout.take()
            .ok_or("stdout 획득 실패")?;

        let mut proc = OcrProcess {
            stdin:  std::io::BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            _child: child,
        };

        // 준비 메시지 대기 ({"status":"ready",...})
        let mut ready = String::new();
        proc.stdout
            .read_line(&mut ready)
            .map_err(|e| format!("사이드카 준비 메시지 읽기 실패: {e}"))?;

        let v: serde_json::Value = serde_json::from_str(ready.trim())
            .map_err(|e| format!("준비 메시지 파싱 실패: {e}"))?;

        if v.get("status").and_then(|s| s.as_str()) == Some("error") {
            return Err(v["error"].as_str().unwrap_or("알 수 없는 오류").to_string());
        }

        Ok(proc)
    }

    fn request(&mut self, image_path: &str) -> Result<serde_json::Value, String> {
        let req = serde_json::json!({"image_path": image_path});
        writeln!(self.stdin, "{req}")
            .map_err(|e| format!("사이드카 요청 전송 실패: {e}"))?;
        self.stdin.flush()
            .map_err(|e| format!("stdin flush 실패: {e}"))?;

        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .map_err(|e| format!("사이드카 응답 읽기 실패: {e}"))?;

        serde_json::from_str(line.trim())
            .map_err(|e| format!("응답 파싱 실패: {e}"))
    }
}

pub struct OcrState(pub Mutex<Option<OcrProcess>>);

fn sidecar_script_path() -> String {
    // 개발: 프로젝트 루트 기준 / 배포: 리소스 디렉토리
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("sidecar");
    path.push("ocr_sidecar.py");
    path.to_string_lossy().to_string()
}

fn ensure_sidecar(state: &Mutex<Option<OcrProcess>>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        let script = sidecar_script_path();
        *guard = Some(OcrProcess::spawn(&script)?);
    }
    Ok(())
}

// ── 타입 정의 ────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct OcrResultItem {
    pub text:             String,
    pub confidence:       Option<f64>,
    pub bbox:             Vec<i64>,
    pub text_type:        String,
    pub needs_correction: bool,
}

#[derive(Serialize)]
pub struct OcrResponse {
    pub session_id: i64,
    pub results:    Vec<OcrResultItem>,
}

#[derive(Serialize)]
pub struct OcrSessionRow {
    pub id:         i64,
    pub image_path: String,
    pub image_hash: String,
    pub created_at: String,
    pub result_count: i64,
}

// ── 커맨드 ───────────────────────────────────────────────────────

/// 이미지 OCR 실행 → 세션/결과 DB 저장 → 결과 반환
#[tauri::command]
pub fn ocr_image(
    image_path: String,
    ocr:        State<'_, OcrState>,
    db:         State<'_, DbState>,
) -> Result<OcrResponse, String> {
    // 사이드카 기동
    ensure_sidecar(&ocr.0)?;

    // OCR 요청
    let resp = {
        let mut guard = ocr.0.lock().map_err(|e| e.to_string())?;
        let proc = guard.as_mut().ok_or("사이드카가 없습니다")?;
        match proc.request(&image_path) {
            Ok(v) => v,
            Err(e) => {
                // 프로세스 죽었을 가능성 → 다음 요청에서 재기동
                *guard = None;
                return Err(e);
            }
        }
    };

    if resp["success"].as_bool() != Some(true) {
        return Err(resp["error"].as_str().unwrap_or("OCR 오류").to_string());
    }

    // 이미지 해시 (SHA-256 앞 16자리)
    let image_hash = sha256_short(&image_path);

    // DB 저장
    let conn_guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = conn_guard
        .as_ref()
        .ok_or("프로젝트가 열려 있지 않습니다")?;

    conn.execute(
        "INSERT INTO OcrSession (image_path, image_hash) VALUES (?1, ?2)",
        params![image_path, image_hash],
    )
    .map_err(|e| e.to_string())?;

    let session_id = conn.last_insert_rowid();

    let items: Vec<OcrResultItem> = serde_json::from_value(
        resp["results"].clone(),
    )
    .map_err(|e| format!("결과 파싱 실패: {e}"))?;

    for item in &items {
        let bbox_str = serde_json::to_string(&item.bbox).unwrap_or_default();
        conn.execute(
            "INSERT INTO OcrResult (session_id, raw_text, confidence, bbox, text_type)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                session_id,
                item.text,
                item.confidence,
                bbox_str,
                item.text_type,
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(OcrResponse { session_id, results: items })
}

/// 사용자 교정값 저장
#[tauri::command]
pub fn save_ocr_correction(
    result_id:      i64,
    corrected_text: String,
    db:             State<'_, DbState>,
) -> Result<(), String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn  = guard.as_ref().ok_or("프로젝트가 열려 있지 않습니다")?;

    conn.execute(
        "UPDATE OcrResult
         SET corrected_text = ?1, corrected_at = datetime('now')
         WHERE id = ?2",
        params![corrected_text, result_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 최근 OCR 세션 목록 조회
#[tauri::command]
pub fn get_ocr_history(
    limit: i64,
    db:    State<'_, DbState>,
) -> Result<Vec<OcrSessionRow>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn  = guard.as_ref().ok_or("프로젝트가 열려 있지 않습니다")?;

    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.image_path, s.image_hash, s.created_at,
                    COUNT(r.id) AS result_count
             FROM OcrSession s
             LEFT JOIN OcrResult r ON r.session_id = s.id
             GROUP BY s.id
             ORDER BY s.created_at DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(OcrSessionRow {
                id:           row.get(0)?,
                image_path:   row.get(1)?,
                image_hash:   row.get(2)?,
                created_at:   row.get(3)?,
                result_count: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// 학습 데이터셋 내보내기 (교정값이 있는 항목만 JSON)
#[tauri::command]
pub fn export_ocr_dataset(
    output_path: String,
    db:          State<'_, DbState>,
) -> Result<usize, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn  = guard.as_ref().ok_or("프로젝트가 열려 있지 않습니다")?;

    let mut stmt = conn
        .prepare(
            "SELECT s.image_path, r.raw_text, r.corrected_text, r.bbox, r.text_type
             FROM OcrResult r
             JOIN OcrSession s ON s.id = r.session_id
             WHERE r.corrected_text IS NOT NULL",
        )
        .map_err(|e| e.to_string())?;

    #[derive(Serialize)]
    struct DatasetRow {
        image_path:     String,
        raw_text:       String,
        corrected_text: String,
        bbox:           serde_json::Value,
        text_type:      String,
    }

    let rows: Vec<DatasetRow> = stmt
        .query_map([], |row| {
            let bbox_str: String = row.get(3).unwrap_or_default();
            let bbox = serde_json::from_str(&bbox_str).unwrap_or(serde_json::Value::Null);
            Ok(DatasetRow {
                image_path:     row.get(0)?,
                raw_text:       row.get(1)?,
                corrected_text: row.get(2)?,
                bbox,
                text_type:      row.get(4).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let count = rows.len();
    let json  = serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?;
    std::fs::write(&output_path, json).map_err(|e| e.to_string())?;

    Ok(count)
}

// ── 헬퍼 ────────────────────────────────────────────────────────

fn sha256_short(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(s.as_bytes());
    format!("{:x}", hash)[..16].to_string()
}
