pub struct HexCodec;

impl HexCodec {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn decode(input: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
        (0..input.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&input[i..i + 2], 16))
            .collect()
    }
}
