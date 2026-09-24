use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "namecheap",
    about = "A fast, ergonomic CLI for Namecheap to manage domains, DNS records, and account settings.",
    version
)]
pub struct Cli {
    /// Namecheap API Username (can also be set via NAMECHEAP_API_USER env var)
    #[arg(long, global = true)]
    pub api_user: Option<String>,

    /// Namecheap API Key (can also be set via NAMECHEAP_API_KEY env var)
    #[arg(long, global = true)]
    pub api_key: Option<String>,

    /// Whitelisted Client IP Address (can also be set via NAMECHEAP_CLIENT_IP env var)
    #[arg(long, global = true)]
    pub client_ip: Option<String>,

    /// Use Namecheap Sandbox API environment (api.sandbox.namecheap.com)
    #[arg(long, global = true)]
    pub sandbox: bool,

    /// Output results in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Domain management commands (check, list, info)
    Domains(DomainsArgs),

    /// DNS record management commands (list, add, delete, nameservers)
    Dns(DnsArgs),

    /// View or configure API credentials and defaults
    Config(ConfigArgs),

    /// Log in and store Namecheap API credentials
    Login(LoginArgs),

    /// Look up your current public IP address (useful for Namecheap API IP whitelisting)
    Ip,
}

#[derive(Args, Debug, Clone)]
pub struct LoginArgs {
    /// Namecheap API username
    #[arg(short = 'u', long)]
    pub api_user: Option<String>,

    /// Namecheap API key
    #[arg(short = 'k', long)]
    pub api_key: Option<String>,

    /// Read API key from stdin
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Whitelisted client IP address
    #[arg(long)]
    pub client_ip: Option<String>,

    /// Default to sandbox environment
    #[arg(long)]
    pub sandbox: bool,
}

#[derive(Args, Debug)]
pub struct DomainsArgs {
    #[command(subcommand)]
    pub command: DomainCommands,
}

#[derive(Subcommand, Debug)]
pub enum DomainCommands {
    /// Check availability of one or more domain names
    Check {
        /// One or more domain names to check (e.g. example.com test.org)
        #[arg(required = true)]
        domains: Vec<String>,
    },

    /// List registered domains in your Namecheap account
    List {
        /// Page number (1-based)
        #[arg(short, long, default_value_t = 1)]
        page: u32,

        /// Number of domains per page (max 100)
        #[arg(short = 's', long, default_value_t = 20)]
        page_size: u32,

        /// Filter domains containing this search term
        #[arg(long)]
        search: Option<String>,

        /// Sort order: NAME, NAME_DESC, EXPIREDATE, EXPIREDATE_DESC, CREATEDATE, CREATEDATE_DESC
        #[arg(long)]
        sort_by: Option<String>,
    },

    /// Get detailed information about a registered domain
    Info {
        /// Domain name (e.g. example.com)
        domain: String,
    },
}

#[derive(Args, Debug)]
pub struct DnsArgs {
    #[command(subcommand)]
    pub command: DnsCommands,
}

#[derive(Subcommand, Debug)]
pub enum DnsCommands {
    /// List all DNS host records for a domain
    List {
        /// Domain name (e.g. example.com)
        domain: String,
    },

    /// Add a new DNS host record
    Add {
        /// Domain name (e.g. example.com)
        domain: String,

        /// Host name / subdomain (e.g. "@", "www", "mail", "*")
        #[arg(short, long)]
        name: String,

        /// Record type: A, AAAA, CNAME, TXT, MX, etc.
        #[arg(short = 't', long)]
        record_type: String,

        /// Record target value or IP address
        #[arg(short, long)]
        address: String,

        /// MX preference (only for MX records)
        #[arg(long)]
        mx_pref: Option<u32>,

        /// Time-to-live in seconds (default: 1800)
        #[arg(long, default_value_t = 1800)]
        ttl: u32,
    },

    /// Delete a DNS host record by host name and optional record type
    Delete {
        /// Domain name (e.g. example.com)
        domain: String,

        /// Host name to remove (e.g. "www")
        #[arg(short, long)]
        name: String,

        /// Optionally specify record type to delete (e.g. "A")
        #[arg(short = 't', long)]
        record_type: Option<String>,
    },

    /// Point domain to custom nameservers
    SetCustom {
        /// Domain name (e.g. example.com)
        domain: String,

        /// Comma-separated list of nameservers (e.g. ns1.example.com,ns2.example.com)
        #[arg(short, long, value_delimiter = ',')]
        nameservers: Vec<String>,
    },

    /// Reset domain nameservers back to Namecheap default DNS
    SetDefault {
        /// Domain name (e.g. example.com)
        domain: String,
    },
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration and active credentials
    Show,

    /// Set and persist configuration values
    Set {
        /// Namecheap API username
        #[arg(long)]
        api_user: Option<String>,

        /// Namecheap API key
        #[arg(long)]
        api_key: Option<String>,

        /// Whitelisted client IP address
        #[arg(long)]
        client_ip: Option<String>,

        /// Default to sandbox environment
        #[arg(long)]
        sandbox: Option<bool>,
    },

    /// Print path to the local configuration file
    Path,
}
