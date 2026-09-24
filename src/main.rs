use anyhow::{Context, Result};
use clap::Parser;

use namecheap_cli::cli::{Cli, Commands, ConfigCommands, DnsCommands, DomainCommands, LoginArgs};
use namecheap_cli::client::NamecheapClient;
use namecheap_cli::config::NamecheapConfig;
use namecheap_cli::models::DnsHostRecord;
use namecheap_cli::output;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login(login_args) => handle_login_command(login_args)?,
        Commands::Config(config_args) => handle_config_command(config_args)?,
        Commands::Ip => handle_ip_command(cli.json).await?,

        Commands::Domains(domains_args) => {
            let config =
                NamecheapConfig::resolve(cli.api_user, cli.api_key, cli.client_ip, cli.sandbox)?;
            let client = NamecheapClient::new(config);

            match domains_args.command {
                DomainCommands::Check { domains } => {
                    let results = client.check_domains(&domains).await?;
                    output::print_domain_check(&results, cli.json)?;
                }
                DomainCommands::List {
                    page,
                    page_size,
                    search,
                    sort_by,
                } => {
                    let (items, paging) = client
                        .list_domains(page, page_size, search.as_deref(), sort_by.as_deref())
                        .await?;
                    output::print_domain_list(&items, &paging, cli.json)?;
                }
                DomainCommands::Info { domain } => {
                    let info = client.get_domain_info(&domain).await?;
                    output::print_domain_info(&info, cli.json)?;
                }
            }
        }
        Commands::Dns(dns_args) => {
            let config =
                NamecheapConfig::resolve(cli.api_user, cli.api_key, cli.client_ip, cli.sandbox)?;
            let client = NamecheapClient::new(config);

            match dns_args.command {
                DnsCommands::List { domain } => {
                    let hosts = client.get_dns_hosts(&domain).await?;
                    output::print_dns_hosts(&domain, &hosts, cli.json)?;
                }
                DnsCommands::Add {
                    domain,
                    name,
                    record_type,
                    address,
                    mx_pref,
                    ttl,
                } => {
                    let mut existing = client.get_dns_hosts(&domain).await?;
                    println!(
                        "Adding DNS record to {domain}: {name} ({record_type}) -> {address} (TTL: {ttl})..."
                    );
                    existing.push(DnsHostRecord {
                        host_id: None,
                        name,
                        record_type,
                        address,
                        mx_pref,
                        ttl,
                    });
                    let ok = client.set_dns_hosts(&domain, &existing).await?;
                    if ok {
                        println!("Successfully added record to {domain}.");
                    } else {
                        anyhow::bail!("Failed to update DNS records on Namecheap.");
                    }
                }
                DnsCommands::Delete {
                    domain,
                    name,
                    record_type,
                } => {
                    let existing = client.get_dns_hosts(&domain).await?;
                    let initial_len = existing.len();
                    let filtered: Vec<DnsHostRecord> = existing
                        .into_iter()
                        .filter(|r| {
                            let name_matches = r.name.eq_ignore_ascii_case(&name);
                            if let Some(ref rt) = record_type {
                                !(name_matches && r.record_type.eq_ignore_ascii_case(rt))
                            } else {
                                !name_matches
                            }
                        })
                        .collect();

                    if filtered.len() == initial_len {
                        println!("No matching DNS records found to delete.");
                        return Ok(());
                    }

                    println!(
                        "Deleting matching records for {domain} (retaining {} records)...",
                        filtered.len()
                    );
                    let ok = client.set_dns_hosts(&domain, &filtered).await?;
                    if ok {
                        println!("Successfully updated DNS records for {domain}.");
                    } else {
                        anyhow::bail!("Failed to update DNS records on Namecheap.");
                    }
                }
                DnsCommands::SetCustom {
                    domain,
                    nameservers,
                } => {
                    println!(
                        "Setting custom nameservers for {domain}: {}",
                        nameservers.join(", ")
                    );
                    let ok = client.set_nameservers(&domain, &nameservers).await?;
                    if ok {
                        println!("Successfully updated nameservers for {domain}.");
                    } else {
                        anyhow::bail!("Failed to update nameservers on Namecheap.");
                    }
                }
                DnsCommands::SetDefault { domain } => {
                    println!("Resetting {domain} nameservers to Namecheap default DNS...");
                    let ok = client.set_default_nameservers(&domain).await?;
                    if ok {
                        println!("Successfully reset {domain} to Namecheap default DNS.");
                    } else {
                        anyhow::bail!("Failed to reset nameservers on Namecheap.");
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_config_command(args: namecheap_cli::cli::ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommands::Show => {
            let file_config = NamecheapConfig::load_file();
            let config_path = NamecheapConfig::config_path()?;
            println!("Configuration File: {}", config_path.display());
            println!(
                "  API User:  {}",
                file_config
                    .api_user
                    .as_deref()
                    .unwrap_or("<not configured>")
            );
            println!(
                "  API Key:   {}",
                if file_config.api_key.is_some() {
                    "******** (configured)"
                } else {
                    "<not configured>"
                }
            );
            println!(
                "  Client IP: {}",
                file_config
                    .client_ip
                    .as_deref()
                    .unwrap_or("<not configured>")
            );
            println!(
                "  Sandbox:   {}",
                if file_config.sandbox.unwrap_or(false) {
                    "true (sandbox API)"
                } else {
                    "false (production API)"
                }
            );
        }
        ConfigCommands::Set {
            api_user,
            api_key,
            client_ip,
            sandbox,
        } => {
            let mut file_config = NamecheapConfig::load_file();
            if let Some(u) = api_user {
                file_config.api_user = Some(u);
            }
            if let Some(k) = api_key {
                file_config.api_key = Some(k);
            }
            if let Some(ip) = client_ip {
                file_config.client_ip = Some(ip);
            }
            if let Some(sb) = sandbox {
                file_config.sandbox = Some(sb);
            }

            let path = NamecheapConfig::save_file(&file_config)?;
            println!("Saved configuration to {}", path.display());
        }
        ConfigCommands::Path => {
            let path = NamecheapConfig::config_path()?;
            println!("{}", path.display());
        }
    }
    Ok(())
}

async fn handle_ip_command(json: bool) -> Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.ipify.org?format=json")
        .send()
        .await
        .context("Failed to contact IP lookup service")?;

    #[derive(serde::Deserialize)]
    struct IpResp {
        ip: String,
    }

    let ip_obj = res
        .json::<IpResp>()
        .await
        .context("Failed to parse IP response")?;

    if json {
        println!("{}", serde_json::json!({ "public_ip": ip_obj.ip }));
    } else {
        println!("Public IP: {}", ip_obj.ip);
        println!("\nNote: Make sure this IP is whitelisted in your Namecheap API access settings:");
        println!("  https://ap.www.namecheap.com/settings/tools/apiaccess/whitelisted-ips");
    }
    Ok(())
}

fn handle_login_command(args: LoginArgs) -> Result<()> {
    let mut file_config = NamecheapConfig::load_file();

    let key = if args.api_key_stdin {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        let trimmed = buf.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    } else {
        args.api_key
    };

    if let Some(u) = args.api_user {
        file_config.api_user = Some(u);
    }
    if let Some(k) = key {
        file_config.api_key = Some(k);
    }
    if let Some(ip) = args.client_ip {
        file_config.client_ip = Some(ip);
    }
    if args.sandbox {
        file_config.sandbox = Some(true);
    }

    let path = NamecheapConfig::save_file(&file_config)?;
    println!("✓ Saved Namecheap credentials to {}", path.display());
    Ok(())
}
