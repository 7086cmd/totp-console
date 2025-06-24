# TOTP in Console

To use Cloudflare KV database, create a KV namespace in your Cloudflare account and set the following environment variables, or create a `kv.json` file that contains following keys:

```json
{
  "account_id": "your_account_id",
  "namespace_id": "your_namespace_id",
  "api_token": "your_api_token"
}

```

```
🔐 TOTP Console Manager
Usage: totp-console <command> [args]

Commands:
  add <name> <secret> [issuer]     Add a new TOTP entry
  list                             List all entries
  get <name>                       Get TOTP code for specific entry
  generate                         Generate codes for all entries
  delete <name>                    Delete an entry
  loop [name]                      Continuous refresh mode
  copy <name>                      Copy TOTP code to clipboard
  sync                             Sync to Cloudflare KV
  load                             Load from Cloudflare KV

Environment Variables (for Cloudflare KV):
  CF_ACCOUNT_ID                    Cloudflare account ID
  CF_NAMESPACE_ID                  KV namespace ID
  CF_API_TOKEN                     API token

Examples:
  totp-console add github 0123456789ABCDEF GitHub
  totp-console get github
  totp-console loop
  totp-console loop github
```