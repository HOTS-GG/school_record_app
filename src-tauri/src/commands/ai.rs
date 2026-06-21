use crate::state::{DbState, GlobalConfigState};
use rusqlite::OptionalExtension;
use serde::Serialize;
use tauri::State;

const DEFAULT_MODEL: &str = "anthropic/claude-haiku-4-5";
const APP_REFERER: &str = "https://github.com/school-record-app";
const APP_TITLE: &str = "School Record App";

const DEFAULT_SYSTEM_PROMPT: &str = "당신은 대한민국 고등학교 담당 교사입니다. 교사의 관찰 시점에서 학생의 생활기록부 세부능력특기사항을 작성합니다.\n\n\
작성 규칙:\n\
- 관찰자(교사) 시점에서 서술\n\
- \"~함\", \"~음\", \"~모습\" 등 명사형 종결어미 사용\n\
- 학생 이름 및 개인정보 직접 언급 금지\n\
- 현재형으로 작성\n\
- 긍정적 내용만 서술\n\
- 구체적인 활동 내용과 성장 과정 반영\n\
- 줄바꿈 없이 한 문단으로 출력\n\
- 앞뒤 미사어구 없이 생활기록부 내용만 출력\n\
- 바이트 제한이 있는 경우 반드시 준수";

#[derive(Serialize)]
pub struct AiGenerateResult {
    pub text: String,
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Serialize)]
pub struct KeyDiagnostic {
    pub stored: bool,
    pub length: usize,
    pub prefix: String,  // 앞 8자
    pub suffix: String,  // 뒤 4자
}

fn read_global_str(conn: &rusqlite::Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT config_value FROM APP_CONFIGS WHERE config_key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| e.to_string())
}

/// 저장된 API 키 상태 진단 (실제 키 값은 노출하지 않음)
#[tauri::command]
pub fn diagnose_api_key(global: State<'_, GlobalConfigState>) -> Result<KeyDiagnostic, String> {
    let gcfg = global.0.lock().unwrap();
    let raw = read_global_str(&gcfg, "claude_api_key")?
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if raw.is_empty() {
        return Ok(KeyDiagnostic { stored: false, length: 0, prefix: String::new(), suffix: String::new() });
    }

    let prefix = if raw.len() >= 8 { raw[..8].to_string() } else { raw.clone() };
    let suffix = if raw.len() >= 4 { raw[raw.len()-4..].to_string() } else { raw.clone() };

    Ok(KeyDiagnostic {
        stored: true,
        length: raw.len(),
        prefix,
        suffix,
    })
}

#[tauri::command]
pub async fn ai_generate_record(
    student_name: String,
    area_name: String,
    activity_name: String,
    current_content: String,
    byte_limit: Option<i64>,
    area_id: i64,
    requirements: Option<String>,
    state: State<'_, DbState>,
    global: State<'_, GlobalConfigState>,
) -> Result<AiGenerateResult, String> {
    let (api_key, model, system_prompt) = {
        let gcfg = global.0.lock().unwrap();

        let api_key = read_global_str(&gcfg, "claude_api_key")?
            .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect::<String>())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "API 키가 설정되지 않았습니다. 설정(Settings)에서 OpenRouter API 키를 입력해주세요.".to_string())?;

        let model = read_global_str(&gcfg, "ai_model")?
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        // 영역별 프롬프트는 프로젝트 DB에서 읽기
        let area_prompt: Option<String> = if area_id > 0 {
            let guard = state.0.lock().unwrap();
            if let Some(conn) = guard.as_ref() {
                conn.query_row(
                    "SELECT prompt FROM Area WHERE id = ?1",
                    rusqlite::params![area_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?
                .flatten()
            } else {
                None
            }
        } else {
            None
        };

        let system_prompt = if let Some(p) = area_prompt.filter(|s| !s.trim().is_empty()) {
            p
        } else {
            read_global_str(&gcfg, "ai_system_prompt")?
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_SYSTEM_PROMPT.to_string())
        };

        (api_key, model, system_prompt)
    };

    let byte_info = match byte_limit {
        Some(limit) => format!(
            "바이트 제한: {limit} bytes (UTF-8, 한글 1자=3바이트, 영문/숫자=1바이트, 줄바꿈=2바이트). 반드시 초과하지 말 것."
        ),
        None => "바이트 제한 없음".to_string(),
    };

    let existing_info = if current_content.trim().is_empty() {
        "없음".to_string()
    } else {
        current_content.clone()
    };

    let mut user_message = format!(
        "영역: {area_name}\n\
        활동명: {activity_name}\n\
        학생 정보: {student_name}\n\
        기존 내용: {existing_info}\n\
        {byte_info}"
    );

    if let Some(req) = requirements.as_deref().filter(|s| !s.trim().is_empty()) {
        user_message.push_str(&format!("\n추가 요구사항: {req}"));
    }

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user",   "content": user_message }
        ]
    });

    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", APP_REFERER)
        .header("X-Title", APP_TITLE)
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
        .unwrap_or(&model)
        .to_string();

    let prompt_tokens     = json["usage"]["prompt_tokens"].as_i64().unwrap_or(0);
    let completion_tokens = json["usage"]["completion_tokens"].as_i64().unwrap_or(0);
    let total_tokens      = json["usage"]["total_tokens"].as_i64().unwrap_or(0);

    Ok(AiGenerateResult {
        text,
        model: used_model,
        prompt_tokens,
        completion_tokens,
        total_tokens,
    })
}

// ── 모델 동기화 필터 상수 ─────────────────────────────────────

/// 이미지·영상·음성 생성/처리 전용 모델 제외 키워드
const EXCLUDED_KEYWORDS: &[&str] = &[
    // 이미지 생성
    "dall-e", "stable-diffusion", "imagen", "image-alpha", "image-beta",
    "sdxl", "flux",
    // 영상 생성
    "video", "sora",
    // 음성·오디오
    "tts", "whisper", "audio",
    // 임베딩
    "embed",
    // 모더레이션
    "moderation",
    // 기타 비대화
    "realtime", "vision-only",
];

/// OpenRouter 전체 대화형 모델 동기화 (모든 제공사, 이미지·영상·음성 모델 제외)
#[tauri::command]
pub async fn sync_openrouter_models(
    global: State<'_, GlobalConfigState>,
) -> Result<usize, String> {
    let api_key = {
        let gcfg = global.0.lock().unwrap();
        read_global_str(&gcfg, "claude_api_key")?
            .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect::<String>())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "API 키가 설정되지 않았습니다. 설정에서 OpenRouter API 키를 입력해주세요.".to_string())?
    };

    let client = reqwest::Client::new();
    let resp = client
        .get("https://openrouter.ai/api/v1/models")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("HTTP-Referer", APP_REFERER)
        .header("X-Title", APP_TITLE)
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 오류 {status}: {text}"));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let empty = vec![];
    let models: Vec<serde_json::Value> = json["data"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str()?;
            let id_lower = id.to_lowercase();
            let name = m["name"].as_str().unwrap_or(id);

            // architecture.modality 필드로 출력 타입 확인 (text 출력만 허용)
            if let Some(modality) = m["architecture"]["modality"].as_str() {
                let output = modality.split("->").last().unwrap_or("").to_lowercase();
                if output.contains("image") || output.contains("video") || output.contains("audio") {
                    return None;
                }
            }

            // 제공사 prefix가 영문자·숫자·하이픈·점만 허용 (~ 등 특수문자 제외)
            let provider = id_lower.split('/').next().unwrap_or("");
            if !provider.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.') {
                return None;
            }

            // 키워드 기반 추가 필터
            if EXCLUDED_KEYWORDS.iter().any(|k| id_lower.contains(k)) {
                return None;
            }

            Some(serde_json::json!({ "id": id, "name": name }))
        })
        .collect();

    let count = models.len();
    let json_str = serde_json::to_string(&models).map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();

    {
        let gcfg = global.0.lock().unwrap();
        gcfg.execute(
            "INSERT OR REPLACE INTO APP_CONFIGS (config_key, config_value) VALUES (?1, ?2)",
            rusqlite::params!["synced_models", json_str],
        ).map_err(|e| e.to_string())?;
        gcfg.execute(
            "INSERT OR REPLACE INTO APP_CONFIGS (config_key, config_value) VALUES (?1, ?2)",
            rusqlite::params!["synced_models_at", now],
        ).map_err(|e| e.to_string())?;
    }

    Ok(count)
}

/// API 키 유효성 테스트 — 외부에서 키를 받아 테스트 (입력창에서 직접 테스트용)
#[tauri::command]
pub async fn test_api_key(api_key: String) -> Result<String, String> {
    call_openrouter_auth(api_key.trim().to_string()).await
}

/// 저장된 API 키로 테스트 — GlobalConfig에서 직접 읽어서 테스트
#[tauri::command]
pub async fn test_stored_api_key(global: State<'_, GlobalConfigState>) -> Result<String, String> {
    let key = {
        let gcfg = global.0.lock().unwrap();
        read_global_str(&gcfg, "claude_api_key")?
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "저장된 API 키가 없습니다. 설정에서 키를 입력해주세요.".to_string())?
    };
    call_openrouter_auth(key).await
}

async fn call_openrouter_auth(key: String) -> Result<String, String> {
    let key: String = key.chars().filter(|c| !c.is_whitespace()).collect();
    if key.is_empty() {
        return Err("API 키가 비어 있습니다.".to_string());
    }

    let client = reqwest::Client::new();

    let resp = client
        .get("https://openrouter.ai/api/v1/auth/key")
        .header("Authorization", format!("Bearer {key}"))
        .header("HTTP-Referer", APP_REFERER)
        .header("X-Title", APP_TITLE)
        .send()
        .await
        .map_err(|e| format!("네트워크 오류: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    if status.is_success() {
        let info = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            let limit = json["data"]["limit"].as_f64();
            let usage = json["data"]["usage"].as_f64().unwrap_or(0.0);
            let is_free = json["data"]["is_free_tier"].as_bool().unwrap_or(false);
            let prefix = &key[..key.len().min(12)];
            match limit {
                Some(l) => format!("✓ 연결 성공 (키: {}...) | 사용: ${:.4} / ${:.2}{}", prefix, usage, l, if is_free { " [무료]" } else { "" }),
                None    => format!("✓ 연결 성공 (키: {}...) | 사용: ${:.4}{}", prefix, usage, if is_free { " [무료 티어]" } else { "" }),
            }
        } else {
            format!("✓ 연결 성공 (키: {}...)", &key[..key.len().min(12)])
        };
        Ok(info)
    } else {
        let prefix = &key[..key.len().min(12)];
        Err(format!("✗ 인증 실패 ({status}) | 키: {}... (길이: {})\n응답: {}", prefix, key.len(), text))
    }
}
