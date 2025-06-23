# TOTP in Console

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
  sync (not supported yet)         Sync to Cloudflare KV
  load (not supported yet)         Load from Cloudflare KV

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