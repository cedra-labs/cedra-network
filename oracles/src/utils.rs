use hex;

pub fn get_adjusted_price_u64(price: f64, decimals: u8) -> u64 {
    let scale = 10u64.pow(decimals as u32) as f64; // cast decimals to u32
    (price * scale).round() as u64
}

pub fn decode_hex_string(s: &str) -> String {
    if let Some(stripped) = s.strip_prefix("0x") {
        if let Ok(bytes) = hex::decode(stripped) {
            if let Ok(decoded) = String::from_utf8(bytes) {
                return decoded;
            }
        }
    }
    s.to_string() // fallback: return as-is if not hex or invalid
}