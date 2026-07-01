// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod crypto;
mod db;
mod engine;
mod state;
mod types;
#[cfg(test)]
mod tests;

use commands::*;
use state::{CryptoState, CryptoStateHandle, DbPathState, DbState, GlobalConfigState, ReplaceCache};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 윈도우 아이콘 설정 (개발 모드에서도 반영)
            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon().cloned() {
                    let _ = window.set_icon(icon);
                }
            }

            // 앱 전역 설정 DB 초기화 (%APPDATA%\school-record-app\config.db)
            // API 키, AI 모델 등 프로젝트와 무관한 설정 저장
            let app_data_dir = app.path().app_data_dir()
                .expect("앱 데이터 디렉토리를 찾을 수 없습니다.");
            std::fs::create_dir_all(&app_data_dir)
                .expect("앱 데이터 디렉토리 생성 실패");
            let config_path = app_data_dir.join("config.db");
            let global_conn = db::open_global_config(&config_path)
                .expect("전역 설정 DB 초기화 실패");
            app.manage(GlobalConfigState(Mutex::new(global_conn)));

            // 프로젝트 DB 상태
            app.manage(DbState(Mutex::new(None)));
            app.manage(DbPathState(Mutex::new(None)));
            app.manage(Mutex::new(ReplaceCache {
                ruleset_version: 0,
                entries: HashMap::new(),
            }));
            app.manage(CryptoStateHandle::new(CryptoState { key: None }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 프로젝트
            new_project,
            open_project,
            backup_project,
            migrate_schema,
            // 영역
            get_areas,
            create_area,
            update_area,
            delete_area,
            seed_default_areas,
            seed_areas_by_role,
            set_area_behavior_items,
            // 활동
            get_activities,
            set_area_activities,
            create_activity,
            update_activity,
            delete_activity,
            set_activity_areas,
            // 학생
            get_students,
            create_student,
            update_student,
            delete_student,
            bulk_upsert_students,
            get_area_students,
            set_area_students,
            get_student_tags,
            set_student_tags,
            get_student_behavior,
            set_student_behavior,
            // 기록
            get_area_grid,
            upsert_record,
            get_record_history,
            save_history_snapshot,
            bulk_import_records,
            preview_import_records,
            get_student_full_preview,
            get_all_areas_byte_summary,
            write_bytes_file,
            read_file_base64,
            read_image_base64,
            // 스냅샷
            create_snapshot,
            get_snapshots,
            restore_snapshot,
            // 치환
            get_replace_rules,
            create_replace_rule,
            update_replace_rule,
            delete_replace_rule,
            seed_default_replace_rules,
            preview_replace,
            apply_replace,
            // 유의어
            get_synonym_groups,
            create_synonym_group,
            delete_synonym_group,
            add_synonym_word,
            delete_synonym_word,
            seed_default_synonyms,
            // 맞춤법 검사
            check_spelling,
            check_speller_online,
            get_records_for_spell,
            // 점검
            get_all_records_for_inspect,
            // 설정 (프로젝트 DB)
            get_config,
            set_config,
            delete_config,
            get_global_config,
            set_global_config,
            delete_global_config,
            check_and_update_app_version,
            // 암호화
            get_encryption_status,
            unlock_encryption,
            enable_encryption,
            disable_encryption,
            change_encryption_password,
            // AI 생성
            ai_generate_record,
            diagnose_api_key,
            test_api_key,
            test_stored_api_key,
            sync_openrouter_models,
            // AI 대화
            ai_chat,
            create_chat_session,
            get_chat_sessions,
            get_chat_messages,
            save_chat_message,
            update_chat_session,
            delete_chat_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
