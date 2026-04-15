use std::os::unix::net::UnixDatagram;
use std::time::Duration;

pub struct UdsClient {
    socket: UnixDatagram,
}

impl UdsClient {
    pub fn new(socket_path: &str) -> Result<Self, String> {
        let socket =
            UnixDatagram::unbound().map_err(|e| format!("Failed to create socket: {}", e))?;

        socket
            .connect(socket_path)
            .map_err(|e| format!("Failed to connect: {}", e))?;

        Ok(Self { socket })
    }

    pub fn send(&self, data: &[u8]) -> Result<usize, String> {
        self.socket
            .send(data)
            .map_err(|e| format!("Failed to send: {}", e))
    }

    pub fn send_with_timeout(&self, data: &[u8], timeout: Duration) -> Result<usize, String> {
        self.socket
            .set_write_timeout(Some(timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;
        self.send(data)
    }

    pub fn receive(&self, buf: &mut [u8]) -> Result<usize, String> {
        self.socket
            .recv(buf)
            .map_err(|e| format!("Failed to receive: {}", e))
    }

    pub fn receive_with_timeout(&self, buf: &mut [u8], timeout: Duration) -> Result<usize, String> {
        self.socket
            .set_read_timeout(Some(timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;
        self.receive(buf)
    }
}

pub fn create_client(socket_path: &str) -> Result<UdsClient, String> {
    UdsClient::new(socket_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uds_client_creation() {
        let result = UdsClient::new("/tmp/test.sock");
        assert!(result.is_err());
    }
}
