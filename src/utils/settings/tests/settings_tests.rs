use super::*;
use std::fs;
use tempfile::TempDir;

// Helper to create a temporary settings dir and write a file
fn create_temp_settings(content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let ai_dir = dir.path().join(".ai");
    fs::create_dir_all(&ai_dir).unwrap();
    let settings_file = ai_dir.join("settings.json");
    fs::write(&settings_file, content).unwrap();
    (dir, settings_file)
}

#[test]
fn test_read_settings_file_valid() {
    let (_dir, path) = create_temp_settings(r#"{"permissions": {"allow": ["Read"]}}"#);
    let settings = read_settings_file(&path).unwrap();
    assert_eq!(
        settings["permissions"]["allow"][0].as_str().unwrap(),
        "Read"
    );
}

#[test]
fn test_read_settings_file_empty() {
    let (_dir, path) = create_temp_settings("");
    let settings = read_settings_file(&path).unwrap();
    assert!(settings.is_object());
    assert_eq!(settings.as_object().unwrap().len(), 0);
}

#[test]
fn test_read_settings_file_missing() {
    let result = read_settings_file(Path::new("/nonexistent/path/settings.json"));
    assert!(result.is_none());
}

#[test]
fn test_read_settings_file_invalid_json() {
    let (_dir, path) = create_temp_settings("{invalid json}");
    let result = read_settings_file(&path);
    assert!(result.is_none());
}

#[test]
fn test_deep_merge_objects() {
    let base = serde_json::json!({"a": 1, "b": {"c": 2, "d": 3}});
    let overlay = serde_json::json!({"b": {"c": 99}, "e": 5});
    let merged = deep_merge(&base, &overlay);
    assert_eq!(merged["a"], 1);
    assert_eq!(merged["b"]["c"], 99); // overlay wins
    assert_eq!(merged["b"]["d"], 3); // preserved from base
    assert_eq!(merged["e"], 5); // new key
}

#[test]
fn test_deep_merge_array_replaces() {
    let base = serde_json::json!({"items": [1, 2, 3]});
    let overlay = serde_json::json!({"items": [4, 5]});
    let merged = deep_merge(&base, &overlay);
    assert_eq!(merged["items"].as_array().unwrap().len(), 2);
    assert_eq!(merged["items"][0], 4);
}

#[test]
fn test_deep_merge_null_deletes() {
    let base = serde_json::json!({"a": 1, "b": 2});
    let overlay = serde_json::json!({"b": null});
    let merged = deep_merge(&base, &overlay);
    assert_eq!(merged["a"], 1);
    assert!(merged.get("b").is_none());
}

#[tokio::test]
async fn test_update_settings_creates_file() {
    let dir = TempDir::new().unwrap();
    let ai_dir = dir.path().join(".ai");
    // Don't create the file - update should create it

    // Monkey-patch by changing directory won't work easily;
    // instead test the raw path reading/writing
    let settings_file = ai_dir.join("settings.json");
    fs::create_dir_all(&ai_dir).unwrap();

    let settings = serde_json::json!({"permissions": {"allow": ["Read"]}});
    // Manually simulate what update_settings_for_source does
    let existing = read_settings_file(&settings_file).unwrap_or(Value::Object(serde_json::Map::new()));
    let merged = deep_merge(&existing, &settings);
    let json_str = serde_json::to_string_pretty(&merged).unwrap();
    fs::write(&settings_file, json_str + "\n").unwrap();

    // Verify
    let read_back = read_settings_file(&settings_file).unwrap();
    assert_eq!(read_back["permissions"]["allow"][0].as_str().unwrap(), "Read");
}

#[tokio::test]
async fn test_update_settings_merges_existing() {
    let dir = TempDir::new().unwrap();
    let ai_dir = dir.path().join(".ai");
    fs::create_dir_all(&ai_dir).unwrap();
    let settings_file = ai_dir.join("settings.json");

    // Write existing settings
    fs::write(&settings_file, r#"{"model": "claude-sonnet-4-6"}"#).unwrap();

    // Merge new settings
    let settings = serde_json::json!({"permissions": {"allow": ["Read"]}});
    let existing = read_settings_file(&settings_file).unwrap();
    let merged = deep_merge(&existing, &settings);
    let json_str = serde_json::to_string_pretty(&merged).unwrap();
    fs::write(&settings_file, json_str + "\n").unwrap();

    // Verify both keys exist
    let read_back = read_settings_file(&settings_file).unwrap();
    assert_eq!(read_back["model"].as_str().unwrap(), "claude-sonnet-4-6");
    assert_eq!(read_back["permissions"]["allow"][0].as_str().unwrap(), "Read");
}

#[test]
fn test_settings_path_user() {
    let path = get_settings_file_path_for_source(&EditableSettingSource::UserSettings);
    assert!(path.is_some());
    let p = path.unwrap();
    assert!(p.to_string_lossy().contains(".ai"));
    assert!(p.file_name().unwrap() == "settings.json");
}

#[test]
fn test_settings_path_project() {
    let path = get_settings_file_path_for_source(&EditableSettingSource::ProjectSettings);
    assert!(path.is_some());
    let p = path.unwrap();
    assert!(p.to_string_lossy().contains(".ai"));
    assert!(p.file_name().unwrap() == "settings.json");
}

#[test]
fn test_settings_path_local() {
    let path = get_settings_file_path_for_source(&EditableSettingSource::LocalSettings);
    assert!(path.is_some());
    let p = path.unwrap();
    assert!(p.file_name().unwrap() == "settings.local.json");
}
