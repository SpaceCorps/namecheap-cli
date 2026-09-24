use crate::models::{DnsHostRecord, DomainCheckResult, DomainInfo, DomainItem, PagingInfo};
use anyhow::Result;

pub fn print_domain_check(results: &[DomainCheckResult], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(results)?);
        return Ok(());
    }

    println!(
        "{:<35} {:<15} {:<10} {:<15}",
        "DOMAIN", "STATUS", "PREMIUM", "PRICE"
    );
    println!("{:-<75}", "");
    for res in results {
        let status = if res.available { "AVAILABLE" } else { "TAKEN" };
        let premium = if res.is_premium { "YES" } else { "NO" };
        let price = if let Some(p) = res.premium_registration_price {
            format!("${:.2}", p)
        } else if res.available {
            "Standard".to_string()
        } else {
            "-".to_string()
        };

        println!(
            "{:<35} {:<15} {:<10} {:<15}",
            res.domain, status, premium, price
        );
    }
    Ok(())
}

pub fn print_domain_list(items: &[DomainItem], paging: &PagingInfo, json: bool) -> Result<()> {
    if json {
        let out = serde_json::json!({
            "domains": items,
            "paging": paging
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    if items.is_empty() {
        println!("No domains found.");
        return Ok(());
    }

    println!(
        "{:<6} {:<30} {:<12} {:<12} {:<10} {:<10} {:<12}",
        "ID", "DOMAIN", "EXPIRES", "AUTO-RENEW", "LOCKED", "EXPIRED", "WHOISGUARD"
    );
    println!("{:-<95}", "");
    for d in items {
        println!(
            "{:<6} {:<30} {:<12} {:<12} {:<10} {:<10} {:<12}",
            d.id,
            d.name,
            d.expires,
            if d.auto_renew { "Yes" } else { "No" },
            if d.is_locked { "Yes" } else { "No" },
            if d.is_expired { "Yes" } else { "No" },
            d.whois_guard
        );
    }
    println!(
        "\nPage {} of {} (Total domains: {})",
        paging.current_page,
        (paging.total_items + paging.page_size - 1) / paging.page_size.max(1),
        paging.total_items
    );
    Ok(())
}

pub fn print_domain_info(info: &DomainInfo, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(info)?);
        return Ok(());
    }

    println!("Domain Details:");
    println!("  Name:             {}", info.name);
    println!("  ID:               {}", info.id);
    println!("  Status:           {}", info.status);
    println!("  Created:          {}", info.created_date);
    println!("  Expires:          {}", info.expired_date);
    println!(
        "  Locked:           {}",
        if info.is_locked { "Yes" } else { "No" }
    );
    println!(
        "  WhoisGuard:       {}",
        if info.whois_guard_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!(
        "  Using Our DNS:    {}",
        if info.is_using_our_dns {
            "Yes (Namecheap BasicDNS)"
        } else {
            "No (Custom DNS)"
        }
    );
    println!("  Nameservers ({}):", info.nameservers.len());
    for ns in &info.nameservers {
        println!("    - {}", ns);
    }
    Ok(())
}

pub fn print_dns_hosts(domain: &str, hosts: &[DnsHostRecord], json: bool) -> Result<()> {
    if json {
        let out = serde_json::json!({
            "domain": domain,
            "records": hosts
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    if hosts.is_empty() {
        println!("No DNS host records configured for {domain}.");
        return Ok(());
    }

    println!("DNS Records for {domain}:");
    println!(
        "{:<8} {:<15} {:<10} {:<35} {:<8} {:<8}",
        "ID", "HOST", "TYPE", "VALUE / TARGET", "MX PREF", "TTL"
    );
    println!("{:-<88}", "");
    for h in hosts {
        let id_str = h
            .host_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".into());
        let mx_str = h
            .mx_pref
            .map(|mx| mx.to_string())
            .unwrap_or_else(|| "-".into());
        println!(
            "{:<8} {:<15} {:<10} {:<35} {:<8} {:<8}",
            id_str, h.name, h.record_type, h.address, mx_str, h.ttl
        );
    }
    Ok(())
}
