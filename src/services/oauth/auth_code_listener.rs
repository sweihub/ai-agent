// Source: /data/home/swei/claudecode/openclaudecode/src/services/oauth/auth-code-listener.ts
//! Local HTTP server that listens for OAuth authorization code redirects.
//!
//! When the user authorizes in their browser, the OAuth provider redirects to:
//! http://localhost:[port]/callback?code=AUTH_CODE&state=STATE
//!
//! This server captures that redirect and extracts the auth code.
//! Note: This is NOT an OAuth server - it's just a redirect capture mechanism.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use std::sync::Arc;

/// Shared state between the HTTP accept loop and the authorization waiter.
struct ListenerState {
    expected_state: tokio::sync::Mutex<Option<String>>,
}

/// Shared sender for auth codes — used by both the HTTP callback handler
/// and external callers (manual flow).
#[derive(Clone)]
struct CodeSender {
    tx: Arc<tokio::sync::Mutex<Option<mpsc::UnboundedSender<String>>>>,
}

impl CodeSender {
    async fn send(&self, code: String) -> anyhow::Result<()> {
        let tx = self.tx.lock().await;
        if let Some(tx) = tx.as_ref() {
            tx.send(code)
                .map_err(|e| anyhow::anyhow!("Failed to send auth code: {e}"))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Auth code sender not initialized"))
        }
    }
}

/// Temporary localhost HTTP server that listens for OAuth authorization code redirects.
///
/// The server binds to localhost on a random ephemeral port (or a specified one)
/// and waits for the OAuth provider to redirect the browser back with an
/// authorization code and state parameter.
pub struct AuthCodeListener {
    state: Arc<ListenerState>,
    /// Join handle for the accept loop task.
    accept_task: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    shutdown_tx: tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    callback_path: String,
    port: std::sync::Mutex<Option<u16>>,
    code_sender: CodeSender,
}

impl AuthCodeListener {
    pub fn new(callback_path: &str) -> Self {
        Self {
            state: Arc::new(ListenerState {
                expected_state: tokio::sync::Mutex::new(None),
            }),
            accept_task: tokio::sync::Mutex::new(None),
            shutdown_tx: tokio::sync::Mutex::new(None),
            callback_path: callback_path.to_string(),
            port: std::sync::Mutex::new(None),
            code_sender: CodeSender {
                tx: Arc::new(tokio::sync::Mutex::new(None)),
            },
        }
    }

    /// Starts listening on a random available port on localhost.
    /// Returns the port number.
    pub async fn start(&self, port: Option<u16>) -> anyhow::Result<u16> {
        let addr = match port {
            Some(p) => format!("127.0.0.1:{p}"),
            None => "127.0.0.1:0".to_string(),
        };

        let listener = TcpListener::bind(&addr).await?;
        let actual_port = listener.local_addr()?.port();

        *self.port.lock().unwrap() = Some(actual_port);

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        let state = self.state.clone();
        let callback_path = self.callback_path.clone();
        let code_sender = self.code_sender.clone();
        let join_handle = tokio::spawn(async move {
            Self::accept_loop(listener, callback_path, state, code_sender, shutdown_rx).await;
        });

        // Store the join handle instead of the raw listener
        *self.listener.lock().await = Some(join_handle);

        Ok(actual_port)
    }

    async fn accept_loop(
        listener: TcpListener,
        callback_path: String,
        state: Arc<ListenerState>,
        code_sender: CodeSender,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) {
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let cb_path = callback_path.clone();
                            let st = state.clone();
                            let cs = code_sender.clone();
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(stream, cb_path, st, cs).await {
                                    log::warn!("OAuth callback connection error: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("OAuth callback accept error: {e}");
                            break;
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
    }

    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        callback_path: String,
        state: Arc<ListenerState>,
        code_sender: CodeSender,
    ) -> anyhow::Result<()> {
        let mut buffer = [0u8; 4096];
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            return Ok(());
        }

        let request_str = String::from_utf8_lossy(&buffer[..n]);
        let request = Self::parse_request(&request_str);

        let is_callback = request.path == callback_path
            || request.path.starts_with(&format!("{}?", &callback_path));

        if !is_callback {
            let response = Self::make_response(404, "Not Found", "text/plain", "Not Found");
            let _ = stream.write_all(response.as_bytes()).await;
            return Ok(());
        }

        let (code, state_param, error, error_description) = Self::parse_query_params(&request.query);

        if let Some(ref err) = error {
            let body = format!(
                "OAuth error: {err}: {}",
                error_description.as_deref().unwrap_or("")
            );
            let response = Self::make_response(400, "Bad Request", "text/plain", &body);
            let _ = stream.write_all(response.as_bytes()).await;
            return Ok(());
        }

        let auth_code = match code {
            Some(c) => c,
            None => {
                let response = Self::make_response(400, "Bad Request", "text/plain", "Authorization code not found");
                let _ = stream.write_all(response.as_bytes()).await;
                return Ok(());
            }
        };

        {
            let expected = state.expected_state.lock().await;
            if state_param.as_deref() != expected.as_deref() {
                let response = Self::make_response(400, "Bad Request", "text/plain", "Invalid state parameter");
                let _ = stream.write_all(response.as_bytes()).await;
                return Ok(());
            }
        }

        // Send auth code via the shared sender (both HTTP handler and manual flow use this)
        if let Err(e) = code_sender.send(auth_code).await {
            log::warn!("Failed to send auth code from HTTP callback: {e}");
        }

        // Redirect to success page
        use crate::services::oauth::constants::get_oauth_config;
        let oauth_config = get_oauth_config();
        let success_url = oauth_config.console_success_url.clone();

        let response = Self::make_redirect(302, &success_url);
        let _ = stream.write_all(response.as_bytes()).await;

        Ok(())
    }

    fn parse_request(request: &str) -> HttpRequest {
        let first_line = request.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();

        let method = if parts.len() >= 1 {
            parts[0].to_string()
        } else {
            "GET".to_string()
        };
        let full_path = if parts.len() >= 2 { parts[1] } else { "/" };

        let (path, query) = if let Some(idx) = full_path.find('?') {
            (&full_path[..idx], full_path[idx + 1..].to_string())
        } else {
            (full_path, String::new())
        };

        HttpRequest {
            method,
            path: path.to_string(),
            query,
        }
    }

    fn parse_query_params(
        query: &str,
    ) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
        let mut code: Option<String> = None;
        let mut state: Option<String> = None;
        let mut error: Option<String> = None;
        let mut error_description: Option<String> = None;

        for param in query.split('&') {
            let mut parts = param.splitn(2, '=');
            let key = urlencoding::decode(parts.next().unwrap_or("")).unwrap_or_default().into_owned();
            let value = parts.next().map(|v| urlencoding::decode(v).unwrap_or_default().into_owned());

            match key.as_str() {
                "code" => code = value,
                "state" => state = value,
                "error" => error = value,
                "error_description" => error_description = value,
                _ => {}
            }
        }

        (code, state, error, error_description)
    }

    fn make_response(status: u16, status_text: &str, content_type: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn make_redirect(status: u16, location: &str) -> String {
        format!(
            "HTTP/1.1 {status} Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
    }

    /// Returns the port number the listener is bound to.
    pub fn get_port(&self) -> u16 {
        *self.port.lock().unwrap().unwrap_or(0)
    }

    /// Checks if a callback has been received.
    pub async fn has_pending_response(&self) -> bool {
        false
    }

    /// Waits for an authorization code to be received from the shared sender.
    /// This works for both automatic (HTTP callback) and manual (user input) codes.
    pub async fn wait_for_authorization(
        &self,
        state: String,
    ) -> anyhow::Result<String> {
        *self.state.expected_state.lock().await = Some(state);

        let (tx, mut rx) = mpsc::unbounded_channel();
        *self.code_sender.tx.lock().await = Some(tx);

        // Wait for code with a 5-minute timeout
        let code = tokio::time::timeout(
            tokio::time::Duration::from_secs(300),
            rx.recv(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("OAuth authorization timed out after 5 minutes"))?
        .ok_or_else(|| anyhow::anyhow!("Auth code channel closed without receiving code"))?;

        Ok(code)
    }

    /// Handles the success redirect after a successful automatic OAuth flow.
    pub async fn handle_success_redirect(&self, _scopes: &[String]) {
        log::info!("OAuth automatic redirect");
    }

    /// Handles error redirect for failed automatic flows.
    pub async fn handle_error_redirect(&self) {
        log::warn!("OAuth automatic redirect error");
    }

    /// Closes the listener and releases all resources.
    pub async fn close(&self) {
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }
        *self.state.expected_state.lock().await = None;
        *self.code_sender.tx.lock().await = None;
        // Wait for the accept loop to finish
        if let Some(handle) = self.accept_task.lock().await.take() {
            let _ = handle.await;
        }
    }

    /// Send an auth code manually (for the manual flow).
    pub async fn send_manual_code(&self, code: String) -> anyhow::Result<()> {
        self.code_sender.send(code).await
    }
}

struct HttpRequest {
    method: String,
    path: String,
    query: String,
}

impl Drop for AuthCodeListener {
    fn drop(&mut self) {
        log::debug!("AuthCodeListener dropped (use async close() for graceful shutdown)");
    }
}
