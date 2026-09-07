# namecheap-cli

A fast, ergonomic Rust-based command-line interface (CLI) for [Namecheap](https://www.namecheap.com/) to manage domains, DNS records, and account settings.

## Features

- **Domain Availability**: Check availability and premium pricing for multiple domains simultaneously.
- **Domain Management**: List all registered domains in your account, filter by search term, sort, and inspect domain details (status, expiration, nameservers, WhoisGuard).
- **DNS Records Management**:
  - List host records (A, AAAA, CNAME, TXT, MX, etc.).
  - Add and delete DNS host records.
  - Switch between Namecheap BasicDNS and custom nameservers.
- **Credential & Config Management**: Store credentials securely in local configuration or supply them via environment variables or CLI flags.
- **IP Detection Utility**: Quick helper command (`namecheap-cli ip`) to detect your public IP address for Namecheap API IP whitelisting.
- **Scripting & Automation Ready**: Output any command formatted as structured JSON with `--json`.

## Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/) (1.80+ recommended)

### Build from Source
```bash
git clone https://github.com/SpaceCorps/namecheap-cli.git
cd namecheap-cli
cargo build --release
```
The binary will be located at `target/release/namecheap-cli`.

## Setup & Authentication

Namecheap API requires three credentials:
1. **API Username**
2. **API Key** (generated under Namecheap Profile -> Tools -> Business & Dev Tools -> Namecheap API Access)
3. **Whitelisted Client IP** (Namecheap enforces IP whitelisting)

### 1. Check your public IP
```bash
namecheap-cli ip
```
Add this IP to your Namecheap Whitelisted IPs in the dashboard.

### 2. Configure credentials
Save credentials once:
```bash
namecheap-cli config set --api-user <YOUR_USER> --api-key <YOUR_KEY> --client-ip <YOUR_IP>
```
Or use environment variables:
```bash
export NAMECHEAP_API_USER="your-username"
export NAMECHEAP_API_KEY="your-api-key"
export NAMECHEAP_CLIENT_IP="your-public-ip"
export NAMECHEAP_SANDBOX="false"
```

To view current settings:
```bash
namecheap-cli config show
```

## Usage

### Domain Availability Check
```bash
# Check availability of one or more domains
namecheap-cli domains check example.com mycoolapp.io rustacean.dev

# JSON output
namecheap-cli domains check example.com --json
```

### List Registered Domains
```bash
# List domains
namecheap-cli domains list

# Filter and paginate
namecheap-cli domains list --search "myproject" --page 1 --page-size 50
```

### Domain Details
```bash
namecheap-cli domains info example.com
```

### DNS Records
```bash
# List DNS host records
namecheap-cli dns list example.com

# Add an A record
namecheap-cli dns add example.com --name "@" --type "A" --address "192.0.2.1" --ttl 1800

# Add a CNAME record
namecheap-cli dns add example.com --name "www" --type "CNAME" --address "example.com."

# Delete a record
namecheap-cli dns delete example.com --name "www" --type "CNAME"

# Point to custom nameservers (Cloudflare, Route53, etc.)
namecheap-cli dns set-custom example.com --nameservers "ns1.example.com,ns2.example.com"

# Reset to Namecheap default DNS
namecheap-cli dns set-default example.com
```

### Using Sandbox
To test commands against the Namecheap Sandbox API, pass the `--sandbox` flag or set `--sandbox true` in `config set`:
```bash
namecheap-cli --sandbox domains check sandbox-test-123.com
```

## License

MIT License. See [LICENSE](LICENSE) for details.
