pub async fn generate_session_name(
    _messages: &[impl serde::Serialize],
    _signal: std::sync::mpsc::Sender<()>,
) -> Option<String> {
    None
}
