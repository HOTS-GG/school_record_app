use crate::commands::area::{
    create_area_impl, default_prompt_for, delete_area_impl, get_areas_impl, update_area_impl,
};
use super::{insert_activity, insert_area, insert_student, setup_test_db};

/// 기본 영역 6종에 모두 지침이 정의되어 있어야 한다.
/// (전역 프롬프트가 영역별 지침을 떠안지 않는 구조의 전제)
#[test]
fn test_every_default_area_has_prompt() {
    for name in [
        "자율활동",
        "진로활동",
        "행동특성및종합의견",
        "창체동아리",
        "자율동아리",
        "과목별세부능력특기사항",
    ] {
        let p = default_prompt_for(name);
        assert!(p.is_some(), "{name} 영역의 기본 지침이 없음");
        assert!(
            p.unwrap().len() > 50,
            "{name} 영역의 기본 지침이 지나치게 짧음"
        );
    }
    assert!(default_prompt_for("존재하지않는영역").is_none());
}

#[test]
fn test_create_default_area_auto_fills_prompt() {
    let conn = setup_test_db();
    // 기본 영역 이름 + 빈 지침 → 권장 지침이 기본값으로 들어간다 (공백만 있어도 비어 있는 것으로 본다)
    create_area_impl(&conn, "자율활동", 1500, None, "homeroom").unwrap();
    create_area_impl(&conn, "진로활동", 1500, Some("   "), "homeroom").unwrap();
    for name in ["자율활동", "진로활동"] {
        let prompt: String = conn
            .query_row("SELECT prompt FROM Area WHERE name = ?1", [name], |r| r.get(0))
            .unwrap();
        assert_eq!(prompt, default_prompt_for(name).unwrap(), "{name} 권장 지침이 자동으로 채워져야 함");
    }
}

#[test]
fn test_create_default_area_keeps_teacher_prompt() {
    let conn = setup_test_db();
    create_area_impl(&conn, "자율활동", 1500, Some("교사가 직접 쓴 지침"), "homeroom").unwrap();
    let prompt: String = conn
        .query_row("SELECT prompt FROM Area WHERE name = '자율활동'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(prompt, "교사가 직접 쓴 지침", "교사가 쓴 지침을 권장 지침으로 바꾸면 안 됨");
}

#[test]
fn test_create_custom_area_prompt_stays_none() {
    let conn = setup_test_db();
    create_area_impl(&conn, "내가만든영역", 500, None, "common").unwrap();
    let prompt: Option<String> = conn
        .query_row("SELECT prompt FROM Area WHERE name = '내가만든영역'", [], |r| r.get(0))
        .unwrap();
    assert!(prompt.is_none(), "기본 영역이 아니면 지침은 비어 있어야 함");
}

#[test]
fn test_create_area_returns_id() {
    let conn = setup_test_db();
    let id = create_area_impl(&conn, "국어", 500, None, "common").unwrap();
    assert!(id > 0);
}

#[test]
fn test_create_area_duplicate_name_error() {
    let conn = setup_test_db();
    create_area_impl(&conn, "국어", 500, None, "common").unwrap();
    let err = create_area_impl(&conn, "국어", 500, None, "common").unwrap_err();
    assert!(err.contains("이미 같은 이름의 영역"), "에러 메시지: {err}");
}

#[test]
fn test_get_areas_empty_db() {
    let conn = setup_test_db();
    let areas = get_areas_impl(&conn).unwrap();
    assert!(areas.is_empty());
}

#[test]
fn test_get_areas_single_no_activities() {
    let conn = setup_test_db();
    insert_area(&conn, "수학", 400);
    let areas = get_areas_impl(&conn).unwrap();
    assert_eq!(areas.len(), 1);
    assert_eq!(areas[0].name, "수학");
    assert!(areas[0].activities.is_empty());
}

#[test]
fn test_get_areas_with_activities() {
    let conn = setup_test_db();
    let area_id = insert_area(&conn, "과학", 600);
    let act_id = insert_activity(&conn, "실험보고서");

    conn.execute(
        "INSERT INTO AreaActivity (area_id, activity_id) VALUES (?1, ?2)",
        rusqlite::params![area_id, act_id],
    )
    .unwrap();

    let areas = get_areas_impl(&conn).unwrap();
    assert_eq!(areas.len(), 1);
    assert_eq!(areas[0].activities.len(), 1);
    assert_eq!(areas[0].activities[0].name, "실험보고서");
}

#[test]
fn test_update_area_name_and_limit() {
    let conn = setup_test_db();
    let id = create_area_impl(&conn, "영어", 300, None, "common").unwrap();
    update_area_impl(&conn, id, "영어(개정)", 600, None, "common").unwrap();

    let areas = get_areas_impl(&conn).unwrap();
    assert_eq!(areas[0].name, "영어(개정)");
    assert_eq!(areas[0].byte_limit, 600);
}

#[test]
fn test_update_area_duplicate_name_error() {
    let conn = setup_test_db();
    let id1 = create_area_impl(&conn, "체육", 200, None, "common").unwrap();
    let id2 = create_area_impl(&conn, "음악", 200, None, "common").unwrap();
    let _ = id1;
    let err = update_area_impl(&conn, id2, "체육", 200, None, "common").unwrap_err();
    assert!(err.contains("이미 같은 이름의 영역"), "에러 메시지: {err}");
}

#[test]
fn test_delete_area_removes_row() {
    let conn = setup_test_db();
    let id = create_area_impl(&conn, "미술", 250, None, "common").unwrap();
    delete_area_impl(&conn, id).unwrap();

    let areas = get_areas_impl(&conn).unwrap();
    assert!(areas.is_empty());
}

#[test]
fn test_delete_area_cascades_area_activity() {
    let conn = setup_test_db();
    let area_id = insert_area(&conn, "기술", 300);
    let act_id = insert_activity(&conn, "설계도");
    conn.execute(
        "INSERT INTO AreaActivity (area_id, activity_id) VALUES (?1, ?2)",
        rusqlite::params![area_id, act_id],
    )
    .unwrap();

    delete_area_impl(&conn, area_id).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM AreaActivity WHERE area_id=?1",
            rusqlite::params![area_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

// ── CHECK 제약 검증 ────────────────────────────────────────────

#[test]
fn test_create_area_byte_limit_zero_violates_check() {
    let conn = setup_test_db();
    let err = create_area_impl(&conn, "영역", 0, None, "common").unwrap_err();
    assert!(err.contains("CHECK constraint failed"), "byte_limit=0 CHECK 위반이어야 함: {err}");
}

#[test]
fn test_create_area_negative_byte_limit_violates_check() {
    let conn = setup_test_db();
    let err = create_area_impl(&conn, "영역", -100, None, "common").unwrap_err();
    assert!(err.contains("CHECK constraint failed"), "byte_limit=-100 CHECK 위반이어야 함: {err}");
}

#[test]
fn test_update_area_byte_limit_zero_violates_check() {
    let conn = setup_test_db();
    let id = create_area_impl(&conn, "영역", 500, None, "common").unwrap();
    let err = update_area_impl(&conn, id, "영역", 0, None, "common").unwrap_err();
    assert!(err.contains("CHECK constraint failed"), "update byte_limit=0 CHECK 위반이어야 함: {err}");
}

#[test]
fn test_delete_area_cascades_area_student() {
    let conn = setup_test_db();
    let area_id = insert_area(&conn, "가정", 300);
    let student_id = insert_student(&conn, 1, 1, 1, "홍길동");
    conn.execute(
        "INSERT INTO AreaStudent (area_id, student_id) VALUES (?1, ?2)",
        rusqlite::params![area_id, student_id],
    )
    .unwrap();

    delete_area_impl(&conn, area_id).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM AreaStudent WHERE area_id=?1",
            rusqlite::params![area_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}
