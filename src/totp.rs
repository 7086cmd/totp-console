use std::time::{SystemTime, UNIX_EPOCH};
use sha1::{Digest, Sha1};

// TOTP implementation
#[derive(Debug, Clone)]
pub(crate) struct Totp {
    secret: Vec<u8>,
    time_step: u64,
    digits: usize,
}

impl Totp {
    pub(crate) fn new(secret: Vec<u8>) -> Self {
        Self {
            secret,
            time_step: 30,
            digits: 6,
        }
    }

    pub(crate) fn generate(&self) -> anyhow::Result<String> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let counter = time / self.time_step;

        let counter_bytes = counter.to_be_bytes();

        // HMAC-SHA1
        let mut hasher = Sha1::new();
        let mut ipad = [0x36; 64];
        let mut opad = [0x5c; 64];

        let mut key = self.secret.clone();
        if key.len() > 64 {
            key = Sha1::digest(&key).to_vec();
        }
        key.resize(64, 0);

        for i in 0..64 {
            ipad[i] ^= key[i];
            opad[i] ^= key[i];
        }

        hasher.update(ipad);
        hasher.update(counter_bytes);
        let inner_hash = hasher.finalize_reset();

        hasher.update(opad);
        hasher.update(inner_hash);
        let hmac = hasher.finalize();

        let offset = (hmac[19] & 0xf) as usize;
        let code = ((hmac[offset] & 0x7f) as u32) << 24
            | (hmac[offset + 1] as u32) << 16
            | (hmac[offset + 2] as u32) << 8
            | (hmac[offset + 3] as u32);

        let otp = code % 10_u32.pow(self.digits as u32);
        Ok(format!("{:0width$}", otp, width = self.digits))
    }

    pub(crate) fn time_remaining(&self) -> u64 {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.time_step - (time % self.time_step)
    }

    #[cfg(test)]
    pub(crate) fn generate_at_time(&self, unix_time: u64) -> anyhow::Result<String> {
        let counter = unix_time / self.time_step;
        let counter_bytes = counter.to_be_bytes();

        // HMAC-SHA1
        let mut hasher = Sha1::new();
        let mut ipad = [0x36; 64];
        let mut opad = [0x5c; 64];

        let mut key = self.secret.clone();
        if key.len() > 64 {
            key = Sha1::digest(&key).to_vec();
        }
        key.resize(64, 0);

        for i in 0..64 {
            ipad[i] ^= key[i];
            opad[i] ^= key[i];
        }

        hasher.update(ipad);
        hasher.update(counter_bytes);
        let inner_hash = hasher.finalize_reset();

        hasher.update(opad);
        hasher.update(inner_hash);
        let hmac = hasher.finalize();

        let offset = (hmac[19] & 0xf) as usize;
        let code = ((hmac[offset] & 0x7f) as u32) << 24
            | (hmac[offset + 1] as u32) << 16
            | (hmac[offset + 2] as u32) << 8
            | (hmac[offset + 3] as u32);

        let otp = code % 10_u32.pow(self.digits as u32);
        Ok(format!("{:0width$}", otp, width = self.digits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_generation() {
        // Test vector from RFC 6238
        // Using secret "12345678901234567890" (ASCII)
        let secret = b"12345678901234567890".to_vec();
        let totp = Totp::new(secret);

        // Test at time 59 seconds (counter = 1)
        // Expected: 287082 (from RFC 6238 test vector)
        let code = totp.generate_at_time(59).unwrap();
        assert_eq!(code.len(), 6);
        assert_eq!(code, "287082");
    }

    #[test]
    fn test_totp_generation_multiple_times() {
        // Test multiple time steps
        let secret = b"12345678901234567890".to_vec();
        let totp = Totp::new(secret);

        // Test vector from RFC 6238
        let test_cases = vec![
            (59, "287082"),
            (1111111109, "081804"),
            (1111111111, "050471"),
            (1234567890, "005924"),
            (2000000000, "279037"),
            (20000000000, "353130"),
        ];

        for (time, expected) in test_cases {
            let code = totp.generate_at_time(time).unwrap();
            assert_eq!(code, expected, "Failed at time {}", time);
        }
    }

    #[test]
    fn test_time_remaining() {
        let secret = vec![1, 2, 3, 4, 5];
        let totp = Totp::new(secret);

        let remaining = totp.time_remaining();
        assert!(remaining > 0 && remaining <= 30);
    }

    #[test]
    fn test_totp_code_length() {
        let secret = vec![1, 2, 3, 4, 5];
        let totp = Totp::new(secret);

        let code = totp.generate().unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
}

#[cfg(test)]
mod base32_tests {
    use crate::base32::base32_decode;

    #[test]
    fn test_base32_decode_valid() {
        // Test valid base32 strings
        // "JBSWY3DPEBLW64TMMQ======" decodes to "Hello World"
        let result = base32_decode("JBSWY3DPEBLW64TMMQ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"Hello World");
    }

    #[test]
    fn test_base32_decode_invalid() {
        // Test with invalid characters
        let result = base32_decode("INVALID123!@#");
        assert!(result.is_err());
    }

    #[test]
    fn test_base32_decode_empty() {
        let result = base32_decode("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"");
    }
}