mod base32;
mod database;
mod totp;
mod kv;

use std::collections::HashMap;
use std::env;
use database::TOTPDatabase;
use crate::base32::base32_decode;
use crate::database::TOTPEntry;
use crate::kv::get_cloudflare_kv;
use crate::totp::TOTP;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = TOTPDatabase::new("totp.db")?;

    let args = std::env::args().collect::<Vec<_>>();

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

            let entry = TOTPEntry {
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
                    let totp = TOTP::new(secret);
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
                    let totp = TOTP::new(secret);
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
                    let totp = TOTP::new(secret);
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
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
    println!("  generate                         Generate codes for all entries");
    println!("  delete <name>                    Delete an entry");
    println!("  loop [name]                      Continuous refresh mode");
    println!("  copy <name>                      Copy TOTP code to clipboard");
    println!("  sync                             Sync to Cloudflare KV");
    println!("  load                             Load from Cloudflare KV");
    println!();
    println!("Environment Variables (for Cloudflare KV):");
    println!("  CF_ACCOUNT_ID                    Cloudflare account ID");
    println!("  CF_NAMESPACE_ID                  KV namespace ID");
    println!("  CF_API_TOKEN                     API token");
    println!();
    println!("Examples:");
    println!("  totp-console add github 0123456789ABCDEF GitHub");
    println!("  totp-console get github");
    println!("  totp-console loop");
    println!("  totp-console loop github");
}