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
}