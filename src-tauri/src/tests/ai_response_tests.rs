use crate::commands::ai::extract_response_text;
use serde_json::json;

#[test]
fn test_plain_string_content() {
    let resp = json!({ "choices": [{ "message": { "content": "  활동에 적극 참여함.  " } }] });
    assert_eq!(extract_response_text(&resp).unwrap(), "활동에 적극 참여함.");
}

#[test]
fn test_content_block_array() {
    let resp = json!({ "choices": [{ "message": { "content": [
        { "type": "text", "text": "첫 문장. " },
        { "type": "text", "text": "둘째 문장." }
    ] } }] });
    assert_eq!(extract_response_text(&resp).unwrap(), "첫 문장. 둘째 문장.");
}

#[test]
fn test_top_level_error_in_200_response() {
    let resp = json!({ "error": { "code": 402, "message": "Insufficient credits" } });
    let err = extract_response_text(&resp).unwrap_err();
    assert!(err.contains("Insufficient credits"), "{err}");
    assert!(!err.contains("응답 파싱 실패"), "{err}");
}

#[test]
fn test_choice_level_error() {
    let resp = json!({ "choices": [{ "error": { "message": "upstream provider error" }, "message": { "content": null } }] });
    let err = extract_response_text(&resp).unwrap_err();
    assert!(err.contains("upstream provider error"), "{err}");
}

#[test]
fn test_null_content_reports_finish_reason_and_snippet() {
    let resp = json!({ "choices": [{ "finish_reason": "length", "message": { "content": null, "reasoning": "생각 중..." } }] });
    let err = extract_response_text(&resp).unwrap_err();
    assert!(err.contains("finish_reason: length"), "{err}");
    assert!(err.contains("reasoning"), "{err}");
}

#[test]
fn test_empty_string_content_is_error() {
    let resp = json!({ "choices": [{ "finish_reason": "stop", "message": { "content": "   " } }] });
    assert!(extract_response_text(&resp).is_err());
}

#[test]
fn test_long_response_snippet_is_truncated() {
    let long = "가".repeat(1000);
    let resp = json!({ "choices": [{ "message": { "content": null, "reasoning": long } }] });
    let err = extract_response_text(&resp).unwrap_err();
    assert!(err.ends_with('…'), "{err}");
    assert!(err.chars().count() < 400, "len={}", err.chars().count());
}
