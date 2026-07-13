use crate::state::{DbState, GlobalConfigState};
use crate::types::CellPdfNote;
use base64::Engine;
use rusqlite::OptionalExtension;
use serde::Serialize;
use tauri::State;

const DEFAULT_MODEL: &str = "anthropic/claude-haiku-4-5";
const APP_REFERER: &str = "https://github.com/school-record-app";
const APP_TITLE: &str = "School Record App";

const DEFAULT_SYSTEM_PROMPT: &str = r#"# 1. 역할(Role)

- 교과목 세부능력 및 특기사항(세특), 행동 발달 및 종합의견(행발), 창의적 체험활동(창체), 자유학기 활동 등 다양한 영역을 포괄하며 학생 개별 특성과 역량을 객관적이고 신뢰성 있게 기록함.
- 초등학교부터 고등학교까지 학년별 특성과 교과목별 교육 목표에 맞추어 체계적이고 맞춤형 문장 작성.
- 글자 수 및 바이트 수를 정밀하게 관리하여 요청된 분량 내에서 일관되고 논리적인 흐름 유지.
- 교사 시점 관찰 기록체로 객관적이고 정중한 어조 사용, 중복 표현 최소화 및 긍정적·발전 가능성 강조.
- 작성 후 피드백을 체계적으로 반영하여 최종 완성도 제고, 학생별 특성에 따른 개별화 내용 충분히 반영.

# 2. 작성 규칙(Writing Rules)

- 학생 직접 지칭 금지 ('학생', '그', '그녀' 등)
- 모든 문장 명사형 종결(~함, ~모습, ~역량 발휘함)
- 긍정적 표현 우선, 부정적 내용 포함 시 개선 의지 및 성장 가능성 병기
- 다양한 어휘 활용으로 반복 및 중복 표현 방지
- 교과 수업, 수행 평가, 탐구, 토론, 실험 등의 구체적 활동 내용 반영
- 글자 수 및 바이트 수 요청 기준 엄격 준수
- 객관적이고 전문적 어투 유지 및 편견 최소화
- 학생 개별 특성과 상황별 맞춤형 표현 권장

# 3. 작성 절차(Process)

1. 작성 항목 선택 (세특, 행발, 창체, 자유학기 등)
2. 글자 수 및 바이트 수 설정
3. 학생 특성 및 활동 키워드 수집 및 세분화
4. 입력 정보를 기반으로 초안 작성
5. 글자 수 및 바이트 수 안내, 분량 조절 방법 설명
6. 교사 및 관련자 피드백 수집 및 체계적 반영
7. 수정 및 보완 후 최종본 완성 및 확인
8. 최종 글자 수·바이트 수 재확인 및 제공

# 4. 톤과 스타일(Tone & Style)

- 객관적이고 교사 시점 관찰 기록체로 작성
- 긍정적이며 발전 가능성을 강조하는 문체 유지
- 간결하고 명확하며 풍부한 어휘 활용
- 논리적 문장 흐름과 자연스러운 연결어 사용
- 문장 끝맺음 항상 명사형 통일성 확보
- 부정적 내용도 개선 의지·성장 가능성과 함께 서술
- 편견 배제와 사실 근거 중심 기록 강조

# 5. 자연스러운 연결어 및 문장 흐름

- 시간 및 순서: 먼저, 이후에, 동시에, 나중에, 결국
- 원인과 결과: 때문에, 따라서, 이로 인해, 그 결과
- 대조 및 반전: 그러나, 반면에, 다만, 그와 달리
- 예시 및 부연 설명: 예를 들어, 즉, 다시 말해, 특히
- 강조 및 부각: 특히, 무엇보다도, 더욱이, 확실히
- 추가 및 확장: 또한, 게다가, 나아가, 아울러
- 비교 및 유사: 마찬가지로, 이와 같이, 비슷하게
- 요약 및 결론: 결국, 따라서, 결론적으로, 요약하면

# 6. 문장 패턴 예시

- 수업에 능동적으로 참여하며 학습 태도가 뛰어남.
- 탐구 과제를 체계적으로 수행하여 과제 해결 역량을 발휘함.
- 모둠 활동에서 의견 조율과 공동 수행을 주도함.
- 발표 준비 및 실행 과정에서 표현력이 두각을 나타냄.
- 책임감을 가지고 과제를 완수하며 발전 가능성 제시함.
- 자기주도적 학습 태도를 바탕으로 꾸준한 성장 의지를 보임.
- 협력 과정에서 타인의 의견을 존중하고 조율하는 능력이 탁월함.
- 실험 및 조사 활동에서 높은 집중력과 체계적 접근법 활용함.
- 논리적인 사고와 창의적 문제 분석 및 대응 능력을 발휘함.
- 교과 학습 내용과 연계한 심화 탐구 활동 적극적으로 수행함.

# 7. 단어 변환 및 어휘 팁

- '적극적' → '능동적', '주도적', '진취적'
- '협력' → '조율', '협업', '공동 수행'
- '우수' → '탁월', '뛰어남', '두각'
- '발표력' → '표현력', '의사 전달 능력'
- '성장 가능성' → '발전 가능성', '향상 가능성'
- '문제 해결' → '과제 해결', '문제 분석 및 대응'

# 8. 글쓰기 팁

- 3단 구성 활용: 학습 태도 → 활동 성과 → 성장 가능성
- 문장 길이 조절과 연결어 활용으로 자연스러운 리듬감 형성
- 구체적 활동과 결과를 서술해 설득력 강화
- 긍정적 발전 방향 항상 포함
- 어휘 다양화로 반복 방지
- 학생 특성과 교과 특성 반영
- 명사형 종결 유지
- 문장 길이는 20~30자 내외 권장, 너무 길거나 짧은 문장은 간결화 또는 연결

# 9. 교과별 세부 작성법 예시

## 국어
- 문학 작품 해석과 감상에서 심도 있는 사고를 보임.
- 발표와 토론에서 논리적인 의견 개진 및 의사소통 능력을 발휘함.

## 수학
- 문제 해결 과정에서 체계적 사고와 창의적 접근 능력을 발휘함.
- 수학적 개념 이해 및 응용 능력이 뛰어남.

## 과학
- 실험 설계 및 수행에서 정확성과 집중력이 탁월함.
- 과학적 탐구 과정에서 논리적 분석과 종합 능력이 두드러짐.

## 사회
- 다양한 사회 현상과 이슈에 대한 비판적 사고력을 발휘함.
- 탐구 활동과 토론을 통해 문제 해결 방안을 제시함.

# 10. 행동 발달 및 종합의견 작성 팁

- 협력과 소통 능력 강조.
- 자기주도적 학습 태도 및 책임감 구체적 서술.
- 문제 해결 과정에서의 태도 및 의지 표현.
- 성장 가능성과 발전 방향 명확히 기술.
- 교사 관찰 시 편견 배제 및 객관적 기록법 준수.

# 11. 창의적 체험활동 및 자유학기 활동 작성법

- 자율활동 참여도와 태도 구체적 기술.
- 동아리 활동에서 역할과 성과 명확히 표현.
- 진로 탐색 과정에서 자기주도성 강조.
- 스포츠클럽 활동에서 협력과 성취 표현.

# 12. 부정적 내용 긍정적 전환법

- 단점이나 부족한 점 기술 시 반드시 개선 의지와 노력 병기.
- 다양한 부정적 사례별 긍정 전환 예시 포함 (예: 집중력 부족, 시간 관리 미흡, 과제 제출 지연 등).
- 부정적 요소도 성장 가능성 일부로 자연스럽게 표현.

# 13. 글자 수 및 바이트 수 관리법

- 한글과 영문, 특수문자 혼용 시 바이트 수 차이 주의.
- 글자 수와 바이트 수 직접 계산 방법 간략 안내 포함.
- 분량 초과 시 불필요 중복어휘 제거, 문장 간결화로 조절 권장.

# 14. 피드백 반영 프로세스

- 작성 후 교사 및 관련자 피드백 수집 절차 명확화.
- 피드백 내용 분류 및 우선순위 선정.
- 수정 작업 및 재확인 단계 체계적 시행.
- 최종 완성본 확인 및 기록 보관.

# 15. 자주 쓰이는 생활기록부 용어 사전

- 수행평가, 탐구 활동, 모둠 활동, 발표력, 협력, 리더십, 자기주도 학습, 문제 해결력, 성장 가능성, 책임감, 태도, 성취도, 의견 조율, 심화 학습, 표현력, 분석력, 종합적 사고

# 16. **최종출력형태**:
최종 출력물은 앞뒤로 어떠한 미사어구 없이 생활기록부 내용 한문단(줄바꿈없음)."#;

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

    // 문자 단위로 잘라야 함 — 바이트 슬라이싱은 한글 등 멀티바이트 문자에서 panic → 앱 강제 종료
    let prefix: String = raw.chars().take(8).collect();
    let suffix: String = {
        let chars: Vec<char> = raw.chars().collect();
        let start = chars.len().saturating_sub(4);
        chars[start..].iter().collect()
    };

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
    activity_id: i64,
    student_id: i64,
    include_pdf: Option<bool>,
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

        // 전역 → 영역 → 활동 순서로 프롬프트 적층
        let global_prompt = read_global_str(&gcfg, "ai_system_prompt")?
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SYSTEM_PROMPT.to_string());

        let (area_prompt, activity_prompt, activity_date_info, pdf_summary): (Option<String>, Option<String>, Option<String>, Option<String>) = {
            let guard = state.0.lock().unwrap();
            if let Some(conn) = guard.as_ref() {
                let ap = if area_id > 0 {
                    conn.query_row(
                        "SELECT prompt FROM Area WHERE id = ?1",
                        rusqlite::params![area_id],
                        |row| row.get(0),
                    ).optional().map_err(|e| e.to_string())?.flatten()
                } else { None };

                let (actp, actd) = if activity_id > 0 {
                    conn.query_row(
                        "SELECT prompt, date_info FROM Activity WHERE id = ?1",
                        rusqlite::params![activity_id],
                        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?)),
                    ).optional().map_err(|e| e.to_string())?.unwrap_or((None, None))
                } else { (None, None) };

                let pdf = if include_pdf.unwrap_or(true) && activity_id > 0 && student_id > 0 {
                    let mut stmt = conn.prepare(
                        "SELECT file_name, ai_summary FROM CellPdfNote
                         WHERE activity_id = ?1 AND student_id = ?2 AND enabled = 1 ORDER BY id",
                    ).map_err(|e| e.to_string())?;
                    let summaries = stmt
                        .query_map(rusqlite::params![activity_id, student_id], |row| {
                            Ok((row.get::<_, Option<String>>(0)?, row.get::<_, String>(1)?))
                        })
                        .map_err(|e| e.to_string())?
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| e.to_string())?;
                    let joined = summaries.iter()
                        .filter(|(_, s)| !s.trim().is_empty())
                        .map(|(name, s)| match name {
                            Some(n) => format!("({n})\n{s}"),
                            None => s.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    if joined.is_empty() { None } else { Some(joined) }
                } else { None };

                (ap, actp, actd, pdf)
            } else {
                (None, None, None, None)
            }
        };

        // 비어있지 않은 레이어만 개행으로 합치기
        let mut layers = vec![global_prompt];
        if let Some(p) = area_prompt.filter(|s| !s.trim().is_empty()) {
            layers.push(format!("[영역 추가 지침]\n{p}"));
        }
        if let Some(p) = activity_prompt.filter(|s| !s.trim().is_empty()) {
            layers.push(format!("[활동 추가 지침]\n{p}"));
        }
        if let Some(d) = activity_date_info.filter(|s| !s.trim().is_empty()) {
            layers.push(format!("[활동 내용/일정 메모]\n{d}"));
        }
        if let Some(s) = pdf_summary.filter(|s| !s.trim().is_empty()) {
            layers.push(format!("[학생 PDF 분석 자료]\n{s}"));
        }
        let system_prompt = layers.join("\n\n");

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

    // 문자 단위로 잘라야 함 — 바이트 슬라이싱은 멀티바이트 문자에서 panic → 앱 강제 종료
    let prefix: String = key.chars().take(12).collect();

    if status.is_success() {
        let info = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            let limit = json["data"]["limit"].as_f64();
            let usage = json["data"]["usage"].as_f64().unwrap_or(0.0);
            let is_free = json["data"]["is_free_tier"].as_bool().unwrap_or(false);
            match limit {
                Some(l) => format!("✓ 연결 성공 (키: {}...) | 사용: ${:.4} / ${:.2}{}", prefix, usage, l, if is_free { " [무료]" } else { "" }),
                None    => format!("✓ 연결 성공 (키: {}...) | 사용: ${:.4}{}", prefix, usage, if is_free { " [무료 티어]" } else { "" }),
            }
        } else {
            format!("✓ 연결 성공 (키: {}...)", prefix)
        };
        Ok(info)
    } else {
        Err(format!("✗ 인증 실패 ({status}) | 키: {}... (길이: {})\n응답: {}", prefix, key.len(), text))
    }
}

// ── PDF 분석 커맨드 ───────────────────────────────────────────

/// PDF 파일 1개를 AI로 분석하여 요약 텍스트 반환 (내부 헬퍼)
async fn analyze_one_pdf(
    api_key: &str,
    model: &str,
    path: &str,
) -> Result<(String, String), String> {
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document.pdf")
        .to_string();
    let bytes = std::fs::read(path)
        .map_err(|e| format!("PDF 파일 읽기 실패 ({name}): {e}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let body = serde_json::json!({
        "model": model,
        "plugins": [
            {
                "id": "file-parser",
                "pdf": { "engine": "mistral-ocr" }
            }
        ],
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "이 PDF 파일의 내용을 분석하여 학생의 활동 특성과 주요 내용을 요약해주세요. 주요 활동 내용, 성과, 특이사항을 포함하여 3~5문장으로 간결하게 한국어로 작성해주세요."
                    },
                    {
                        "type": "file",
                        "file": {
                            "filename": name.clone(),
                            "file_data": format!("data:application/pdf;base64,{b64}")
                        }
                    }
                ]
            }
        ]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", APP_REFERER)
        .header("X-Title", APP_TITLE)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("API 요청 실패 ({name}): {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API 오류 {status} ({name}): {text}"));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let summary = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| format!("응답 파싱 실패 ({name})"))?
        .trim()
        .to_string();

    Ok((name, summary))
}

/// PDF 파일(복수 가능)을 AI로 개별 분석하고 파일당 1행씩 DB에 저장
#[tauri::command]
pub async fn analyze_cell_pdf(
    activity_id: i64,
    student_id: i64,
    file_paths: Vec<String>,
    state: State<'_, DbState>,
    global: State<'_, GlobalConfigState>,
) -> Result<Vec<CellPdfNote>, String> {
    if file_paths.is_empty() {
        return Err("선택된 PDF 파일이 없습니다.".to_string());
    }

    let (api_key, model) = {
        let gcfg = global.0.lock().unwrap();
        let key = read_global_str(&gcfg, "claude_api_key")?
            .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect::<String>())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "API 키가 설정되지 않았습니다.".to_string())?;
        let m = read_global_str(&gcfg, "ai_model")?
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        (key, m)
    };

    let mut saved: Vec<CellPdfNote> = Vec::with_capacity(file_paths.len());

    for path in &file_paths {
        let (name, summary) = analyze_one_pdf(&api_key, &model, path).await?;

        let guard = state.0.lock().unwrap();
        let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
        conn.execute(
            "INSERT INTO CellPdfNote (activity_id, student_id, file_name, ai_summary, updated_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            rusqlite::params![activity_id, student_id, name, summary],
        ).map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();
        let updated_at: String = conn.query_row(
            "SELECT updated_at FROM CellPdfNote WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;

        saved.push(CellPdfNote {
            id,
            activity_id,
            student_id,
            file_name: Some(name),
            ai_summary: summary,
            enabled: true,
            updated_at,
        });
    }

    Ok(saved)
}

/// 셀의 PDF 분석 메모 목록 조회
#[tauri::command]
pub fn get_cell_pdf_notes(
    activity_id: i64,
    student_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<CellPdfNote>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, activity_id, student_id, file_name, ai_summary, enabled, updated_at
         FROM CellPdfNote WHERE activity_id = ?1 AND student_id = ?2
         ORDER BY id",
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![activity_id, student_id], |row| Ok(CellPdfNote {
            id:          row.get(0)?,
            activity_id: row.get(1)?,
            student_id:  row.get(2)?,
            file_name:   row.get(3)?,
            ai_summary:  row.get(4)?,
            enabled:     row.get::<_, i64>(5)? != 0,
            updated_at:  row.get(6)?,
        }))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

/// 영역 내 모든 셀의 PDF 분석 메모 조회 (그리드 로드 시 일괄)
#[tauri::command]
pub fn get_area_pdf_notes(
    area_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<CellPdfNote>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    let mut stmt = conn.prepare(
        "SELECT n.id, n.activity_id, n.student_id, n.file_name, n.ai_summary, n.enabled, n.updated_at
         FROM CellPdfNote n
         JOIN AreaActivity aa ON aa.activity_id = n.activity_id
         WHERE aa.area_id = ?1
         ORDER BY n.id",
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![area_id], |row| Ok(CellPdfNote {
            id:          row.get(0)?,
            activity_id: row.get(1)?,
            student_id:  row.get(2)?,
            file_name:   row.get(3)?,
            ai_summary:  row.get(4)?,
            enabled:     row.get::<_, i64>(5)? != 0,
            updated_at:  row.get(6)?,
        }))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

/// PDF 분석 메모 사용 여부 토글 (AI 생성 시 포함 여부)
#[tauri::command]
pub fn set_cell_pdf_note_enabled(
    note_id: i64,
    enabled: bool,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    conn.execute(
        "UPDATE CellPdfNote SET enabled = ?2 WHERE id = ?1",
        rusqlite::params![note_id, enabled as i64],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// PDF 분석 메모 개별 삭제 (id 기준)
#[tauri::command]
pub fn delete_cell_pdf_note(
    note_id: i64,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard.as_ref().ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;
    conn.execute(
        "DELETE FROM CellPdfNote WHERE id = ?1",
        rusqlite::params![note_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
