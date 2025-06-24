use std::env;
use crate::database::TOTPEntry;
use serde::{Deserialize, Serialize};

// Cloudflare KV integration
#[derive(Deserialize, Serialize, Debug)]
pub struct CloudflareKV {
    account_id: String,
    namespace_id: String,
    api_token: String,
}

impl CloudflareKV {
    fn new(account_id: String, namespace_id: String, api_token: String) -> Self {
        Self {
            account_id,
            namespace_id,
            api_token,
        }
    }

    pub(crate) async fn sync_to_kv(&self, entries: &[TOTPEntry]) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/totp_entries",
            self.account_id, self.namespace_id
        );

        let json_data = serde_json::to_string(entries)?;

        let response = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .body(json_data)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Synced {} entries to Cloudflare KV", entries.len());
        } else {
            eprintln!("❌ Failed to sync to Cloudflare KV: {}", response.status());
        }

        Ok(())
    }

    pub(crate) async fn load_from_kv(&self) -> anyhow::Result<Vec<TOTPEntry>> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/totp_entries",
            self.account_id, self.namespace_id
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if response.status().is_success() {
            let entries: Vec<TOTPEntry> = response.json().await?;
            println!("✅ Loaded {} entries from Cloudflare KV", entries.len());
            Ok(entries)
        } else {
            eprintln!("❌ Failed to load from Cloudflare KV: {}", response.status());
            Ok(Vec::new())
        }
    }
}

pub fn get_cloudflare_kv() -> Option<CloudflareKV> {
    // Read `kv.json`, and if it doesn't exist, use env vars.
    if let Ok(file_content) = std::fs::read_to_string("kv.json") {
        if let Ok(kv) = serde_json::from_str::<CloudflareKV>(&file_content) {
            return Some(kv);
        }
    }


    let account_id = env::var("CF_ACCOUNT_ID").ok()?;
    let namespace_id = env::var("CF_NAMESPACE_ID").ok()?;
    let api_token = env::var("CF_API_TOKEN").ok()?;

    Some(CloudflareKV::new(account_id, namespace_id, api_token))
}