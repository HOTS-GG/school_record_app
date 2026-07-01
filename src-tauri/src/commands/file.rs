use crate::engine::validate_parent_dir_path;

#[tauri::command]
pub fn write_bytes_file(path: String, data: String) -> Result<(), String> {
    use base64::Engine;
    validate_parent_dir_path(&path, "저장 위치의 디렉토리가 존재하지 않습니다.")?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())
}

/// 로컬 파일을 base64 문자열로 반환 (엑셀 등 바이너리 파일 읽기용)
#[tauri::command]
pub fn read_file_base64(path: String) -> Result<String, String> {
    use base64::Engine;
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

/// 로컬 이미지 파일을 base64 data URL로 반환 (asset 프로토콜 한글 경로 문제 우회)
#[tauri::command]
pub fn read_image_base64(path: String) -> Result<String, String> {
    use base64::Engine;
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif"          => "image/gif",
        "bmp"          => "image/bmp",
        "webp"         => "image/webp",
        _              => "image/png",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}
