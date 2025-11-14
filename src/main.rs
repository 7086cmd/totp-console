mod base32;
mod database;
mod totp;
mod kv;
mod qrcode;

use std::collections::HashMap;
use std::env;
use database::TotpDatabase;
use crate::base32::base32_decode;
use crate::database::TotpEntry;
use crate::kv::get_cloudflare_kv;
use crate::qrcode::{read_totp_qr_from_file};
use crate::totp::Totp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = TotpDatabase::new("totp.db")?;

    let args = env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    let mut clipboard = arboard::Clipboard::new()?;

    match args[1].as_str() {
        "add" => {
            if args.len() < 4 {
                eprintln!("❌ Usage: totp-console add <name> <secret> [issuer]");
                return Ok(());
            }

            let name = &args[2];
            let secret = &args[3];
            let issuer = args.get(4).cloned();

            // Validate secret
            if base32_decode(secret).is_err() {
                eprintln!("❌ Invalid base32 secret");
                return Ok(());
            }

            let entry = TotpEntry {
                id: None,
                name: name.clone(),
                secret: secret.clone(),
                issuer,
                created_at: String::new(),
            };

            match db.add_entry(&entry) {
                Ok(_) => println!("✅ Added TOTP entry: {}", name),
                Err(e) => eprintln!("❌ Failed to add entry: {}", e),
            }
        }
        "read" => {
            if args.len() != 3 {
                eprintln!("❌ Usage: totp-console read <image_path>");
                return Ok(());
            }
            let image_path = &args[2];
            match read_totp_qr_from_file(image_path) {
                Ok(entry) => {
                    match db.add_entry(&entry) {
                        Ok(_) => println!("✅ Added TOTP entry from image: {}", entry.name),
                        Err(e) => eprintln!("❌ Failed to add entry: {}", e),
                    }
                }
                Err(e) => eprintln!("❌ Error reading TOTP QR code: {}", e),
            }
        }
        "list" => {
            let entries = db.get_all_entries()?;

            if entries.is_empty() {
                println!("📭 No TOTP entries found");
                return Ok(());
            }

            println!("📋 TOTP Entries:");
            println!("================");

            for entry in entries {
                println!("🔑 {}", entry.name);
                if let Some(issuer) = entry.issuer {
                    println!("   Issuer: {}", issuer);
                }
                println!("   Created: {}", entry.created_at);
                println!();
            }
        }
        "get" => {
            if args.len() < 3 {
                eprintln!("❌ Usage: totp-console get <name>");
                return Ok(());
            }

            let name = &args[2];
            match db.get_entry_by_name(name)? {
                Some(entry) => {
                    let secret = base32_decode(&entry.secret)?;
                    let totp = Totp::new(secret);
                    let code = totp.generate()?;
                    let remaining = totp.time_remaining();

                    println!("🔑 {} | Code: {} | Expires in: {}s",
                             entry.name, code, remaining);
                }
                None => {
                    eprintln!("❌ Entry not found: {}", name);
                }
            }
        }
        "copy" => {
            if args.len() < 3 {
                eprintln!("❌ Usage: totp-console copy <name>");
                return Ok(());
            }

            let name = &args[2];
            match db.get_entry_by_name(name)? {
                Some(entry) => {
                    let secret = base32_decode(&entry.secret)?;
                    let totp = Totp::new(secret);
                    let code = totp.generate()?;
                    let remaining = totp.time_remaining();
                    
                    clipboard.set_text(code)?;
                    
                    println!("✅ Copied TOTP code for {}, valid for {} seconds",
                             entry.name, remaining);
                }
                None => {
                    eprintln!("❌ Entry not found: {}", name);
                }
            }
        }
        "delete" => {
            if args.len() < 3 {
                eprintln!("❌ Usage: totp-console delete <name>");
                return Ok(());
            }

            let name = &args[2];
            if db.delete_entry(name)? {
                println!("✅ Deleted entry: {}", name);
            } else {
                eprintln!("❌ Entry not found: {}", name);
            }
        }
        "loop" => {
            let entries = if args.len() >= 3 {
                // Single entry loop
                let name = &args[2];
                match db.get_entry_by_name(name)? {
                    Some(entry) => vec![entry],
                    None => {
                        eprintln!("❌ Entry not found: {}", name);
                        return Ok(());
                    }
                }
            } else {
                // All entries loop
                db.get_all_entries()?
            };

            if entries.is_empty() {
                println!("📭 No TOTP entries found");
                return Ok(());
            }

            println!("🔄 Live TOTP Mode (Press Ctrl+C to stop)");
            println!("=========================================");

            let mut last_codes: HashMap<String, String> = HashMap::new();

            loop {
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");

                println!("🔄 Live TOTP Codes - {}", chrono::Utc::now().format("%H:%M:%S"));
                println!("==========================================");

                for entry in &entries {
                    let secret = base32_decode(&entry.secret)?;
                    let totp = Totp::new(secret);
                    let code = totp.generate()?;
                    let remaining = totp.time_remaining();

                    let status = if last_codes.get(&entry.name) != Some(&code) {
                        "🆕"
                    } else {
                        "  "
                    };

                    let remaining_string = if env::var("NO_COLOR").is_ok() || remaining > 5u64 {
                        remaining.to_string() + "s"
                    } else {
                        format!("\x1b[31m{}s\x1b[0m", remaining) // Red color for low time
                    };

                    println!("{} 🔑 {:20} | {} | {}",
                             status, entry.name, code, remaining_string);

                    last_codes.insert(entry.name.clone(), code);
                }

                println!("\nPress Ctrl+C to exit live mode");
                tokio::time::sleep(std::time::Duration::from_millis(1_000)).await;
            }
        }
        "sync" => {
            match get_cloudflare_kv() {
                Some(kv) => {
                    let entries = db.get_all_entries()?;
                    kv.sync_to_kv(&entries).await?;
                }
                None => {
                    eprintln!("❌ Cloudflare KV not configured. Set CF_ACCOUNT_ID, CF_NAMESPACE_ID, and CF_API_TOKEN environment variables.");
                }
            }
        }

        "load" => {
            match get_cloudflare_kv() {
                Some(kv) => {
                    let entries = kv.load_from_kv().await?;
                    let mut added = 0;

                    for entry in entries {
                        match db.add_entry(&entry) {
                            Ok(_) => {
                                added += 1;
                                println!("✅ Added: {}", entry.name);
                            }
                            Err(_) => {
                                println!("⚠️  Skipped (already exists): {}", entry.name);
                            }
                        }
                    }

                    println!("📥 Loaded {} new entries from Cloudflare KV", added);
                }
                None => {
                    eprintln!("❌ Cloudflare KV not configured. Set CF_ACCOUNT_ID, CF_NAMESPACE_ID, and CF_API_TOKEN environment variables.");
                }
            }
        }
        "export" => {
            if args.len() < 3 {
                eprintln!("❌ Usage: totp-console export <file_path>");
                return Ok(());
            }

            let file_path = &args[2];
            let entries = db.get_all_entries()?;

            if entries.is_empty() {
                eprintln!("⚠️  No entries to export");
                return Ok(());
            }

            let json = serde_json::to_string_pretty(&entries)?;
            std::fs::write(file_path, json)?;

            println!("✅ Exported {} entries to {}", entries.len(), file_path);
        }
        "import" => {
            if args.len() < 3 {
                eprintln!("❌ Usage: totp-console import <file_path>");
                return Ok(());
            }

            let file_path = &args[2];
            let json = std::fs::read_to_string(file_path)?;
            let entries: Vec<TotpEntry> = serde_json::from_str(&json)?;

            let mut added = 0;
            let mut skipped = 0;

            for entry in entries {
                // Validate secret
                if base32_decode(&entry.secret).is_err() {
                    eprintln!("⚠️  Skipped {} (invalid secret)", entry.name);
                    skipped += 1;
                    continue;
                }

                match db.add_entry(&entry) {
                    Ok(_) => {
                        added += 1;
                    }
                    Err(_) => {
                        println!("⚠️  Skipped (already exists): {}", entry.name);
                        skipped += 1;
                    }
                }
            }

            println!("📥 Imported {} entries, skipped {}", added, skipped);
        }
        "search" => {
            if args.len() < 3 {
                eprintln!("❌ Usage: totp-console search <query>");
                return Ok(());
            }

            let query = &args[2];
            let entries = db.search_entries(query)?;

            if entries.is_empty() {
                println!("🔍 No entries found matching '{}'", query);
                return Ok(());
            }

            println!("🔍 Search Results for '{}':", query);
            println!("================");

            for entry in entries {
                println!("🔑 {}", entry.name);
                if let Some(issuer) = entry.issuer {
                    println!("   Issuer: {}", issuer);
                }
                println!("   Created: {}", entry.created_at);
                println!();
            }
        }
        "update" => {
            if args.len() < 3 {
                eprintln!("❌ Usage: totp-console update <name> [--secret <secret>] [--issuer <issuer>]");
                return Ok(());
            }

            let name = &args[2];
            let mut new_secret: Option<&str> = None;
            let mut new_issuer: Option<&str> = None;

            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "--secret" => {
                        if i + 1 < args.len() {
                            new_secret = Some(&args[i + 1]);
                            i += 2;
                        } else {
                            eprintln!("❌ --secret requires a value");
                            return Ok(());
                        }
                    }
                    "--issuer" => {
                        if i + 1 < args.len() {
                            new_issuer = Some(&args[i + 1]);
                            i += 2;
                        } else {
                            eprintln!("❌ --issuer requires a value");
                            return Ok(());
                        }
                    }
                    _ => {
                        eprintln!("❌ Unknown flag: {}", args[i]);
                        return Ok(());
                    }
                }
            }

            if new_secret.is_none() && new_issuer.is_none() {
                eprintln!("❌ Please specify at least one field to update (--secret or --issuer)");
                return Ok(());
            }

            // Validate new secret if provided
            if let Some(secret) = new_secret
                && base32_decode(secret).is_err() {
                    eprintln!("❌ Invalid base32 secret");
                    return Ok(());
                }

            if db.update_entry(name, new_secret, new_issuer)? {
                println!("✅ Updated entry: {}", name);
            } else {
                eprintln!("❌ Entry not found: {}", name);
            }
        }
        "info" => {
            let (count, oldest) = db.get_stats()?;

            println!("📊 Database Statistics");
            println!("=====================");
            println!("Total entries: {}", count);

            if let Some(oldest_date) = oldest {
                println!("Oldest entry: {}", oldest_date);
            }

            if count > 0 {
                println!();
                println!("Database file: totp.db");
            }
        }
        _ => {
            eprintln!("❌ Unknown command: {}", args[1]);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("🔐 TOTP Console Manager");
    println!("Usage: totp-console <command> [args]");
    println!();
    println!("Commands:");
    println!("  add <name> <secret> [issuer]     Add a new TOTP entry");
    println!("  list                             List all entries");
    println!("  get <name>                       Get TOTP code for specific entry");
    println!("  copy <name>                      Copy TOTP code to clipboard");
    println!("  delete <name>                    Delete an entry");
    println!("  update <name> [options]          Update an existing entry");
    println!("  search <query>                   Search entries by name or issuer");
    println!("  loop [name]                      Continuous refresh mode");
    println!("  info                             Show database statistics");
    println!("  read <image_path>                Read TOTP from QR code image");
    println!("  export <file_path>               Export entries to JSON file");
    println!("  import <file_path>               Import entries from JSON file");
    println!("  sync                             Sync to Cloudflare KV");
    println!("  load                             Load from Cloudflare KV");
    println!();
    println!("Update Options:");
    println!("  --secret <secret>                Update the secret key");
    println!("  --issuer <issuer>                Update the issuer");
    println!();
    println!("Cloudflare KV Configuration:");
    println!("  Create a `kv.json` file with the following structure:");
    println!("  {{");
    println!("    \"account_id\": \"your_account_id\",");
    println!("    \"namespace_id\": \"your_namespace_id\",");
    println!("    \"api_token\": \"your_api_token\"");
    println!("  }}");
    println!();
    println!("Environment Variables (for Cloudflare KV):");
    println!("  CF_ACCOUNT_ID                    Cloudflare account ID");
    println!("  CF_NAMESPACE_ID                  KV namespace ID");
    println!("  CF_API_TOKEN                     API token");
    println!();
    println!("Examples:");
    println!("  totp add github 0123456789ABCDEF GitHub");
    println!("  totp get github");
    println!("  totp update github --issuer \"GitHub Inc\"");
    println!("  totp search git");
    println!("  totp export backup.json");
    println!("  totp import backup.json");
    println!("  totp loop");
    println!("  totp loop github");
    println!("  totp sync");
    println!("  totp load");
}