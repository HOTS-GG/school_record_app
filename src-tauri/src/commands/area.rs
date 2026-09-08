use crate::state::{DbState, unique_err};
use crate::types::{ActivityItem, AreaItem};
use rusqlite::Connection;
use std::collections::HashMap;
use tauri::State;

// 영역별 기본 지침 — 영역 생성 시 Area.prompt에 함께 심는다.
// 전역 프롬프트는 어느 영역에나 해당하는 공통 규칙만 담고,
// 영역의 성격(무엇을 어떻게 쓰는 영역인가)은 여기서 책임진다.
const AREA_PROMPTS: &[(&str, &str)] = &[
    (
        "자율활동",
        "이 영역은 자율활동이다. 학급·학교 단위의 자치 활동과 행사 참여를 기록한다.\n\
         - 학급 임원 활동, 학급 규칙 제정, 학급 행사 기획·운영에서 맡은 역할과 기여를 구체적으로 서술한다.\n\
         - 학교 행사(체육대회, 축제, 캠페인 등)는 단순 참가 사실이 아니라 준비 과정과 태도가 드러나게 쓴다.\n\
         - 공동체 구성원으로서의 협력, 배려, 책임 이행이 드러나도록 한다.\n\
         - 결과보다 과정에서 보인 태도와 변화를 중심에 둔다.",
    ),
    (
        "진로활동",
        "이 영역은 진로활동이다. 진로 탐색과 설계 과정을 기록한다.\n\
         - 진로 검사, 직업인 특강, 학과 체험, 진로 상담 등에서 무엇을 알게 되었고 생각이 어떻게 바뀌었는지 서술한다.\n\
         - 희망 진로와 연결된 자기주도적 탐색(자료 조사, 독서, 후속 활동)을 구체적으로 드러낸다.\n\
         - 특정 대학·학과명을 단정적으로 명시하지 않고 관심 분야 수준으로 표현한다.\n\
         - 진로 인식의 변화와 앞으로의 계획이 드러나도록 한다.",
    ),
    (
        "행동특성및종합의견",
        "이 영역은 행동특성 및 종합의견이다. 1년간 관찰한 인성과 학교생활 전반을 종합해 기록한다.\n\
         - 특정 활동 하나가 아니라 여러 장면에서 일관되게 관찰된 성향과 태도를 서술한다.\n\
         - 학습 태도, 대인관계, 생활 습관, 성장 변화를 균형 있게 담는다.\n\
         - 추천서에 준하는 글이므로 강점이 구체적 근거와 함께 드러나게 한다.\n\
         - 부족했던 점은 개선 과정과 함께 서술하고, 단정적 평가나 낙인이 될 표현은 쓰지 않는다.",
    ),
    (
        "창체동아리",
        "이 영역은 창의적 체험활동 중 정규 동아리다. 분량이 짧으므로 핵심만 압축한다.\n\
         - 동아리에서 맡은 역할과 실제 수행한 활동을 우선 서술한다.\n\
         - 활동 주제나 산출물이 있으면 구체적으로 명시한다.\n\
         - 부원 간 협업이나 후배 지도 등 관계적 측면은 여유가 있을 때만 덧붙인다.\n\
         - 수식어를 줄이고 사실 중심으로 쓴다.",
    ),
    (
        "자율동아리",
        "이 영역은 자율동아리다. 학생이 스스로 조직하고 운영한 활동을 기록한다. 분량이 짧으므로 핵심만 압축한다.\n\
         - 동아리 결성 동기와 자발성이 드러나게 한다.\n\
         - 스스로 세운 활동 계획과 실제 수행 내용을 구체적으로 서술한다.\n\
         - 운영 과정에서의 주도성과 책임 이행을 중심에 둔다.\n\
         - 수식어를 줄이고 사실 중심으로 쓴다.",
    ),
    (
        "과목별세부능력특기사항",
        "이 영역은 교과 세부능력 및 특기사항이다. 해당 교과 수업에서 관찰한 학업 역량을 기록한다.\n\
         - 발표, 토론, 실험, 탐구, 수행평가, 과제 등 수업 중 구체적 장면을 근거로 삼는다.\n\
         - 단순한 성실성·태도보다 교과 고유의 사고력(개념 이해, 문제 해결, 자료 해석, 논증 등)이 드러나게 한다.\n\
         - 학생이 스스로 확장한 심화 탐구나 후속 학습이 있으면 반드시 포함한다.\n\
         - 성적, 등수, 점수는 직접 언급하지 않는다.",
    ),
];

/// 영역 이름에 대응하는 기본 지침 (없으면 None)
pub fn default_prompt_for(name: &str) -> Option<&'static str> {
    AREA_PROMPTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, p)| *p)
}

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
                    act.id AS act_id, act.name AS act_name,
                    act.prompt AS act_prompt, act.date_info AS act_date_info
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
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (area_id, area_name, byte_limit, prompt, role, behavior_items, act_id, act_name, act_prompt, act_date_info) =
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
            areas[idx].activities.push(ActivityItem { id, name, prompt: act_prompt, date_info: act_date_info });
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
    // 지침을 비워 두고 기본 영역 이름으로 만들면 권장 지침이 기본값으로 들어간다.
    // 교사가 직접 쓴 지침은 그대로 존중한다.
    let prompt = match prompt.map(str::trim) {
        Some(p) if !p.is_empty() => Some(p),
        _ => default_prompt_for(name.trim()),
    };
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

/// 영역 이름에 대응하는 권장 지침 — 추가 모달에서 미리 보여주기 위한 조회용
#[tauri::command]
pub fn get_area_default_prompt(name: String) -> Option<String> {
    default_prompt_for(name.trim()).map(str::to_string)
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
        // 영역별 기본 지침을 함께 심는다 — 전역 프롬프트가 모든 영역의 지침을 떠안지 않도록
        let prompt = default_prompt_for(name);
        let result = conn.execute(
            "INSERT OR IGNORE INTO Area (name, byte_limit, role, prompt) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![name, byte_limit, role, prompt],
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
