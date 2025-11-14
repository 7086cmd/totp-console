# TOTP Console Manager

A powerful command-line TOTP (Time-based One-Time Password) manager for secure two-factor authentication. Generate, manage, and sync your TOTP codes directly from your terminal.

## Features

- Generate TOTP codes compatible with Google Authenticator, Authy, and other 2FA apps
- Secure local SQLite database storage
- QR code scanning support
- Live refresh mode with real-time code updates
- Clipboard integration for quick code copying
- Cloud synchronization via Cloudflare KV
- Import/Export functionality for backup and migration
- Search and filter entries
- Update existing entries
- Database statistics and information

## Installation

```bash
cargo install --path .
```

## Commands

### Core Commands

#### Add Entry
Add a new TOTP entry with a secret key:
```bash
totp-console add <name> <secret> [issuer]
```
Example:
```bash
totp-console add github JBSWY3DPEHPK3PXP GitHub
```

#### List Entries
Display all stored TOTP entries:
```bash
totp-console list
```

#### Get Code
Generate TOTP code for a specific entry:
```bash
totp-console get <name>
```
Example:
```bash
totp-console get github
```

#### Copy to Clipboard
Generate and copy TOTP code to clipboard:
```bash
totp-console copy <name>
```

#### Delete Entry
Remove a TOTP entry:
```bash
totp-console delete <name>
```

### Advanced Commands

#### Update Entry
Update an existing entry's secret or issuer:
```bash
totp-console update <name> [--secret <secret>] [--issuer <issuer>]
```
Examples:
```bash
totp-console update github --issuer "GitHub Inc"
totp-console update github --secret NEWSECRETKEY
```

#### Search Entries
Search for entries by name or issuer:
```bash
totp-console search <query>
```
Example:
```bash
totp-console search git
```

#### Live Mode
Continuous refresh mode showing real-time TOTP codes:
```bash
totp-console loop [name]
```
Examples:
```bash
totp-console loop          # Show all entries
totp-console loop github   # Show specific entry
```

#### Database Info
Display database statistics:
```bash
totp-console info
```

### QR Code Support

#### Read QR Code
Extract TOTP configuration from QR code image:
```bash
totp-console read <image_path>
```
Example:
```bash
totp-console read qrcode.png
```

### Import/Export

#### Export to JSON
Backup your TOTP entries to a JSON file:
```bash
totp-console export <file_path>
```
Example:
```bash
totp-console export backup.json
```

#### Import from JSON
Restore TOTP entries from a JSON file:
```bash
totp-console import <file_path>
```
Example:
```bash
totp-console import backup.json
```

### Cloud Sync (Cloudflare KV)

#### Sync to Cloud
Upload entries to Cloudflare KV:
```bash
totp-console sync
```

#### Load from Cloud
Download entries from Cloudflare KV:
```bash
totp-console load
```

## Cloudflare KV Configuration

To use cloud synchronization, create a KV namespace in your Cloudflare account and configure credentials using either:

### Option 1: Configuration File
Create a `kv.json` file:
```json
{
  "account_id": "your_account_id",
  "namespace_id": "your_namespace_id",
  "api_token": "your_api_token"
}
```

### Option 2: Environment Variables
```bash
export CF_ACCOUNT_ID=your_account_id
export CF_NAMESPACE_ID=your_namespace_id
export CF_API_TOKEN=your_api_token
```

## Examples

```bash
# Add a TOTP entry
totp-console add github JBSWY3DPEHPK3PXP GitHub

# Get current code
totp-console get github

# Update issuer
totp-console update github --issuer "GitHub Inc"

# Search entries
totp-console search git

# Export for backup
totp-console export ~/backups/totp-backup.json

# Import from backup
totp-console import ~/backups/totp-backup.json

# Live mode
totp-console loop github

# Sync to cloud
totp-console sync
```

## Security Notes

- TOTP secrets are stored locally in `totp.db` SQLite database
- Keep your database file and exports secure
- Use strong passwords for your Cloudflare account if using cloud sync
- The `kv.json` configuration file is automatically ignored by git

## Testing

Run the test suite:
```bash
cargo test
```

The project includes comprehensive tests for:
- TOTP code generation (RFC 6238 compliant)
- Base32 decoding
- Time remaining calculations

## License

This project is open source and available under the MIT License.