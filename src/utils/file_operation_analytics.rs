pub fn track_file_read(_path: &str, _bytes: u64) {}

pub fn track_file_write(_path: &str, _bytes: u64) {}

pub fn track_file_delete(_path: &str) {}

pub fn get_file_operation_stats() -> FileOperationStats {
    FileOperationStats {
        total_reads: 0,
        total_writes: 0,
        total_deletes: 0,
        bytes_read: 0,
        bytes_written: 0,
    }
}

#[derive(Clone, Debug, Default)]
pub struct FileOperationStats {
    pub total_reads: u64,
    pub total_writes: u64,
    pub total_deletes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
}
