use crate::commands::updater::install_kind_for;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sr_updater_test_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_windows_with_uninstaller_is_installed() {
    let dir = temp_dir("installed");
    std::fs::write(dir.join("uninstall.exe"), b"").unwrap();
    assert_eq!(install_kind_for(&dir, true), "installed");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_windows_without_uninstaller_is_portable() {
    let dir = temp_dir("portable");
    assert_eq!(install_kind_for(&dir, true), "portable");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_uninstall_directory_does_not_count() {
    // 같은 이름의 폴더는 설치 흔적이 아니다
    let dir = temp_dir("dirname");
    std::fs::create_dir_all(dir.join("uninstall.exe")).unwrap();
    assert_eq!(install_kind_for(&dir, true), "portable");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_non_windows_is_always_installed() {
    let dir = temp_dir("mac");
    assert_eq!(install_kind_for(&dir, false), "installed");
    let _ = std::fs::remove_dir_all(&dir);
}
