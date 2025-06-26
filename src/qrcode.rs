use image::{DynamicImage};
use rqrr::PreparedImage;
use std::path::Path;
use url::Url;
use crate::database::TotpEntry;

pub fn read_totp_qr(image: DynamicImage) -> anyhow::Result<TotpEntry> {
    // Load and convert image to grayscale
    let gray_img = image.to_luma8();

    // Prepare image for QR detection
    let mut prepared = PreparedImage::prepare(gray_img);

    // Detect QR grids
    let grids = prepared.detect_grids();

    if grids.is_empty() {
        anyhow::bail!("No QR codes found in the image");
    }

    // Decode first QR code
    let (_, content) = grids[0].decode()?;

    // Check if it's a TOTP URL
    if content.starts_with("otpauth://totp/") {
        // otpauth://totp/name:user?secret=secret&issuer=issuer
        let url = Url::parse(content.as_str())?;
        let name = url.path_segments().and_then(|mut segments| segments.next_back()).unwrap_or("unknown").to_string();
        let name = name.split(':').collect::<Vec<&str>>()[0];
        
        let query_pairs = url.query_pairs();
        let mut secret = String::new();
        let mut issuer = None;
        for (key, value) in query_pairs {
            match key.as_ref() {
                "secret" => secret = value.to_string(),
                "issuer" => issuer = Some(value.to_string()),
                _ => {}
            }
        }
        if secret.is_empty() {
            anyhow::bail!("The TOTP URL does not contain a valid secret");
        }
        let created_at = chrono::Utc::now().to_rfc3339(); // Use current timestamp
        Ok(TotpEntry {
            id: None,
            name: name.to_string(),
            secret,
            issuer,
            created_at, // You can set this to the current timestamp if needed
        })
    } else {
        anyhow::bail!("❌ The QR code does not contain a valid TOTP URL")
    }
}

pub fn read_totp_qr_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<TotpEntry> {
    // Load image from file
    let image = image::open(path)?;

    // Read TOTP entry from the QR code in the image
    read_totp_qr(image)
}
