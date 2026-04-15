pub struct Histogram {
    buckets: Vec<u64>,
    min: f64,
    max: f64,
    bucket_count: usize,
}

impl Histogram {
    pub fn new(min: f64, max: f64, bucket_count: usize) -> Self {
        Self {
            buckets: vec![0; bucket_count],
            min,
            max,
            bucket_count,
        }
    }

    pub fn record(&mut self, value: f64) {
        let range = self.max - self.min;
        let bucket = ((value - self.min) / range * self.bucket_count as f64) as usize;
        let bucket = bucket.min(self.bucket_count - 1);
        self.buckets[bucket] += 1;
    }

    pub fn get_count(&self, bucket: usize) -> u64 {
        self.buckets.get(bucket).copied().unwrap_or(0)
    }

    pub fn total_count(&self) -> u64 {
        self.buckets.iter().sum()
    }
}
