use std::path::Path;

/// 설치 형태 판별 — 자동 업데이트는 설치형에서만 가능하다.
///
/// 업데이터는 새 NSIS 설치 파일을 받아 실행하는 방식이라, 무설치(portable) 실행 파일에서
/// 돌리면 기존 exe는 그대로 두고 설치형이 따로 깔린다. 그래서 무설치판에는 설치 대신
/// 새 버전을 내려받으라는 안내만 보여 준다.
///
/// Windows: NSIS 설치 폴더에는 실행 파일 옆에 `uninstall.exe`가 있다. 없으면 무설치판.
/// 그 외 OS(macOS .app)는 업데이터가 번들을 직접 교체하므로 항상 설치형으로 본다.
pub fn install_kind_for(exe_dir: &Path, is_windows: bool) -> &'static str {
    if !is_windows || exe_dir.join("uninstall.exe").is_file() {
        "installed"
    } else {
        "portable"
    }
}

/// 반환값: "installed" | "portable"
#[tauri::command]
pub fn get_install_kind() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| format!("실행 파일 경로를 확인할 수 없습니다: {e}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "실행 파일 폴더를 확인할 수 없습니다.".to_string())?;
    Ok(install_kind_for(dir, cfg!(target_os = "windows")).to_string())
}
