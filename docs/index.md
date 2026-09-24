# Namecheap CLI Documentation

A fast, ergonomic Rust-based command-line interface (CLI) for Namecheap to manage domains, DNS records, and account settings.

## Getting Started
```bash
cargo install --git https://github.com/SpaceCorps/namecheap-cli.git
namecheap-cli login --api-user <USER> --api-key <KEY> --client-ip <IP>
```

## Features
- Check domain availability and pricing
- Manage DNS host records (A, AAAA, CNAME, MX, TXT)
- Public IP detection (`namecheap-cli ip`) for API whitelist setup
- Machine-readable `--json` output across all commands
