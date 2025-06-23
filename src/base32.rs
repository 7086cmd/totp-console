// Base32 decoder
pub fn base32_decode(input: &str) -> anyhow::Result<Vec<u8>> {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let input = input.to_uppercase().replace('=', "");

    if input.chars().any(|c| !alphabet.contains(c)) {
        anyhow::bail!("Invalid base32 character");
    }

    let mut result = Vec::new();
    let mut buffer = 0u64;
    let mut bits = 0;

    for c in input.chars() {
        let value = alphabet.find(c).unwrap() as u64;
        buffer = (buffer << 5) | value;
        bits += 5;

        if bits >= 8 {
            result.push((buffer >> (bits - 8)) as u8);
            bits -= 8;
        }
    }

    Ok(result)
}
