pub struct MeteredReader<R> {
    inner: R,
    bytes_read: u64,
}

impl<R: std::io::Read> MeteredReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            bytes_read: 0,
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;
        self.bytes_read += read as u64;
        Ok(read)
    }

    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}
