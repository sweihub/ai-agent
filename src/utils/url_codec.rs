pub struct UrlCodec;

impl UrlCodec {
    pub fn encode(input: &str) -> String {
        let mut result = String::new();
        for c in input.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }

    pub fn decode(input: &str) -> Result<String, std::string::FromUtf8Error> {
        let mut result = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                let byte = u8::from_str_radix(&hex, 16).unwrap();
                result.push(byte);
            } else {
                result.push(c as u8);
            }
        }

        String::from_utf8(result)
    }
}
