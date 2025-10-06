pub fn get_adjusted_price_u64(price: f64, decimals: u8) -> u64 {
    let scale = 10u64.pow(decimals as u32) as f64; // cast decimals to u32
    (price * scale).round() as u64
}