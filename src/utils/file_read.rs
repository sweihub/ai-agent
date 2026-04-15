use std::path::Path;

pub async fn read_file(path: &Path) -> Result<String, String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| e.to_string())
}

pub fn read_file_sync(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

pub async fn read_file_in_range(path: &Path, start: u64, end: u64) -> Result<String, String> {
    use tokio::io::AsyncSeekExt;

    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| e.to_string())?;

    file.seek(std::io::SeekFrom::Start(start))
        .await
        .map_err(|e| e.to_string())?;

    let mut buffer = tokio::io::AsyncReadExt::take(&mut file, end - start);
    let mut content = String::new();
    tokio::io::AsyncReadExt::read_to_string(&mut buffer, &mut content)
        .await
        .map_err(|e| e.to_string())?;

    Ok(content)
}
