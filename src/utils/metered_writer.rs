pub struct MeteredWriter<W> {
    inner: W,
    bytes_written: u64,
}

impl<W: std::io::Write> MeteredWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            bytes_written: 0,
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.bytes_written += written as u64;
        Ok(written)
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}
