use tauri::State;
use crate::state::DbState;

const SPELLER_URL: &str = "http://164.125.7.61/speller/results";
const SPELLER_PING: &str = "http://164.125.7.61/speller/";

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SpellError {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub description: String,
    pub candidates: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct SpellResult {
    pub original: String,
    pub errors: Vec<SpellError>,
}

#[tauri::command]
pub async fn check_spelling(text: String) -> Result<SpellResult, String> {
    if text.trim().is_empty() {
        return Ok(SpellResult { original: text, errors: vec![] });
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    // 300어절 기준으로 분할 (안전하게 500자 단위)
    let chunks = split_text(&text, 500);
    let mut all_errors: Vec<SpellError> = Vec::new();
    let mut offset = 0usize;

    for chunk in &chunks {
        let normalized = chunk.replace('\n', "\r\n");
        let params = [("text1", normalized.as_str())];

        let res = client
            .post(SPELLER_URL)
            .form(&params)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("서버 연결 실패: {e}"))?;

        let html = res.text().await.map_err(|e| e.to_string())?;
        eprintln!("[speller] chunk({} chars) 응답 앞 300자: {}", chunk.chars().count(), &html.chars().take(300).collect::<String>());
        let errors = parse_speller_html(&html, offset);
        eprintln!("[speller] 파싱 오류 수: {}", errors.len());
        for e in &errors { eprintln!("[speller]  → '{}' candWord={:?}", e.text, e.candidates); }
        all_errors.extend(errors);
        offset += chunk.chars().count();
    }

    Ok(SpellResult { original: text, errors: all_errors })
}

fn parse_speller_html(html: &str, char_offset: usize) -> Vec<SpellError> {
    let Some(start) = html.find("data = [") else {
        eprintln!("[speller] 'data = [' 패턴 없음");
        return vec![];
    };
    let rest = &html[start + "data = [".len() - 1..];
    let Some(end) = rest.find("];") else {
        eprintln!("[speller] '];' 없음");
        return vec![];
    };
    let json_str = &rest[..end + 1];
    eprintln!("[speller] JSON 앞 200자: {}", &json_str.chars().take(200).collect::<String>());

    let Ok(arr) = serde_json::from_str::<serde_json::Value>(json_str) else {
        eprintln!("[speller] JSON 파싱 실패");
        return vec![];
    };
    let Some(err_info) = arr.as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.get("errInfo"))
        .and_then(|v| v.as_array()) else {
        eprintln!("[speller] errInfo 없음, arr={}", &json_str.chars().take(100).collect::<String>());
        return vec![];
    };
    eprintln!("[speller] errInfo 항목 수: {}", err_info.len());

    err_info.iter().filter_map(|e| {
        let cand_raw = e.get("candWord")?.as_str()?;
        if cand_raw.is_empty() { return None; }
        let candidates: Vec<String> = cand_raw.split('|').filter(|s| !s.is_empty()).map(String::from).collect();
        if candidates.is_empty() { return None; }

        let start = e.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as usize + char_offset;
        let end   = e.get("end").and_then(|v| v.as_u64()).unwrap_or(0) as usize + char_offset;
        let text  = e.get("orgStr").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = e.get("help").and_then(|v| v.as_str()).unwrap_or("").to_string();

        Some(SpellError { start, end, text, description, candidates })
    }).collect()
}

fn split_text(text: &str, max_chars: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut chunks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let end = (i + max_chars).min(chars.len());
        chunks.push(chars[i..end].iter().collect());
        i = end;
    }
    chunks
}

#[tauri::command]
pub async fn check_speller_online() -> bool {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build() {
        Ok(c) => c,
        Err(_) => return false,
    };
    client.get(SPELLER_PING).send().await.is_ok()
}

#[tauri::command]
pub fn get_records_for_spell(
    scope: String,
    area_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<crate::types::RecordCell>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or("DB가 열려있지 않습니다.")?;
    crate::engine::get_records_for_scope(
        conn,
        &scope,
        &area_id.map(|id| vec![id]).unwrap_or_default(),
        None,
    )
}
