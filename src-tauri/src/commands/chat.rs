use crate::state::{DbState, GlobalConfigState};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::State;

const DEFAULT_MODEL: &str = "anthropic/claude-haiku-4-5";

// ── API 메시지 ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct AiChatResult {
    pub text: String,
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

// ── DB 모델 ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ChatSessionInfo {
    pub id: i64,
    pub title: String,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ChatMessageInfo {
    pub id: i64,
    pub session_id: i64,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub created_at: String,
}

// ── 내부 헬퍼 ────────────────────────────────────────────────

fn get_config_str(conn: &rusqlite::Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT config_value FROM APP_CONFIGS WHERE config_key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

// ── 세션 CRUD ─────────────────────────────────────────────────

#[tauri::command]
pub fn create_chat_session(
    title: String,
    model: String,
    state: State<'_, DbState>,
) -> Result<ChatSessionInfo, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "INSERT INTO ChatSession (title, model) VALUES (?1, ?2)",
        rusqlite::params![title, model],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, title, model, created_at, updated_at FROM ChatSession WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(ChatSessionInfo {
                id: row.get(0)?,
                title: row.get(1)?,
                model: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chat_sessions(state: State<'_, DbState>) -> Result<Vec<ChatSessionInfo>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, model, created_at, updated_at
             FROM ChatSession
             ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map([], |row| {
            Ok(ChatSessionInfo {
                id: row.get(0)?,
                title: row.get(1)?,
                model: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

#[tauri::command]
pub fn get_chat_messages(
    session_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<ChatMessageInfo>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, role, content, model,
                    prompt_tokens, completion_tokens, total_tokens, created_at
             FROM ChatMessage
             WHERE session_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let messages = stmt
        .query_map(rusqlite::params![session_id], |row| {
            Ok(ChatMessageInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                model: row.get(4)?,
                prompt_tokens: row.get(5)?,
                completion_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(messages)
}

#[tauri::command]
pub fn save_chat_message(
    session_id: i64,
    role: String,
    content: String,
    model: Option<String>,
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
    state: State<'_, DbState>,
) -> Result<i64, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "INSERT INTO ChatMessage
            (session_id, role, content, model, prompt_tokens, completion_tokens, total_tokens)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            session_id,
            role,
            content,
            model,
            prompt_tokens,
            completion_tokens,
            total_tokens
        ],
    )
    .map_err(|e| e.to_string())?;

    // session updated_at 갱신
    conn.execute(
        "UPDATE ChatSession SET updated_at = datetime('now') WHERE id = ?1",
        rusqlite::params![session_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_chat_session(
    session_id: i64,
    title: Option<String>,
    model: Option<String>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    if let Some(t) = title {
        conn.execute(
            "UPDATE ChatSession SET title = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![t, session_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(m) = model {
        conn.execute(
            "UPDATE ChatSession SET model = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![m, session_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn delete_chat_session(
    session_id: i64,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "DELETE FROM ChatSession WHERE id = ?1",
        rusqlite::params![session_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ── AI 대화 ──────────────────────────────────────────────────

#[tauri::command]
pub async fn ai_chat(
    messages: Vec<ChatMessage>,
    system_prompt: Option<String>,
    model: Option<String>,
    state: State<'_, DbState>,
    global: State<'_, GlobalConfigState>,
) -> Result<AiChatResult, String> {
    let (api_key, resolved_model) = {
        let gcfg = global.0.lock().unwrap();

        let api_key = get_config_str(&gcfg, "claude_api_key")?
            .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect::<String>())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "API 키가 설정되지 않았습니다. 설정(Settings)에서 OpenRouter API 키를 입력해주세요.".to_string())?;

        // 파라미터 모델 우선, 없으면 전역 설정값, 없으면 기본값
        let resolved_model = model
            .filter(|s| !s.trim().is_empty())
            .or_else(|| get_config_str(&gcfg, "ai_model").ok().flatten())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        (api_key, resolved_model)
    };

    let mut api_messages: Vec<serde_json::Value> = Vec::new();

    // 학교 정책 제한 — 이미지·영상 생성 요청 거부
    const SCHOOL_POLICY: &str =
        "이미지, 영상, 그림 생성을 요청받으면 반드시 \
        \"학교 정책 상 이미지·영상 생성 기능은 지원되지 않습니다. \
        텍스트 기반 도움이 필요하시면 말씀해주세요.\"라고만 안내하세요.";

    let system_content = match system_prompt.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(sp) => format!("{}\n\n{}", sp, SCHOOL_POLICY),
        None     => SCHOOL_POLICY.to_string(),
    };
    api_messages.push(serde_json::json!({ "role": "system", "content": system_content }));

    for m in &messages {
        api_messages.push(serde_json::json!({ "role": m.role, "content": m.content }));
    }

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": resolved_model,
        "messages": api_messages
    });

    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://github.com/school-record-app")
        .header("X-Title", "School Record App")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("API 요청 실패: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 오류 {status}: {text}"));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let text = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "응답 파싱 실패".to_string())?
        .trim()
        .to_string();

    let used_model = json["model"]
        .as_str()
        .unwrap_or(&resolved_model)
        .to_string();

    let prompt_tokens     = json["usage"]["prompt_tokens"].as_i64().unwrap_or(0);
    let completion_tokens = json["usage"]["completion_tokens"].as_i64().unwrap_or(0);
    let total_tokens      = json["usage"]["total_tokens"].as_i64().unwrap_or(0);

    Ok(AiChatResult {
        text,
        model: used_model,
        prompt_tokens,
        completion_tokens,
        total_tokens,
    })
}
