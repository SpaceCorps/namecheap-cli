use anyhow::{Context, Result, anyhow};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;

use crate::config::NamecheapConfig;
use crate::models::{DnsHostRecord, DomainCheckResult, DomainInfo, DomainItem, PagingInfo};

pub struct NamecheapClient {
    config: NamecheapConfig,
    http: reqwest::Client,
}

impl NamecheapClient {
    pub fn new(config: NamecheapConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    async fn send_request(&self, command: &str, extra_params: &[(&str, &str)]) -> Result<String> {
        let mut params = vec![
            ("ApiUser", self.config.api_user.as_str()),
            ("ApiKey", self.config.api_key.as_str()),
            ("UserName", self.config.api_user.as_str()),
            ("Command", command),
            ("ClientIp", self.config.client_ip.as_str()),
        ];
        params.extend_from_slice(extra_params);

        let url = self.config.base_url();
        let response = self
            .http
            .post(url)
            .form(&params)
            .send()
            .await
            .with_context(|| format!("Failed to send request to Namecheap ({})", url))?;

        let body = response
            .text()
            .await
            .context("Failed to read response body from Namecheap")?;

        check_api_response_errors(&body)?;
        Ok(body)
    }

    /// Check availability of one or more domains
    pub async fn check_domains(&self, domain_list: &[String]) -> Result<Vec<DomainCheckResult>> {
        if domain_list.is_empty() {
            return Ok(Vec::new());
        }
        let domain_csv = domain_list.join(",");
        let body = self
            .send_request("namecheap.domains.check", &[("DomainList", &domain_csv)])
            .await?;
        parse_domain_check_response(&body)
    }

    /// List user's registered domains
    pub async fn list_domains(
        &self,
        page: u32,
        page_size: u32,
        search_term: Option<&str>,
        sort_by: Option<&str>,
    ) -> Result<(Vec<DomainItem>, PagingInfo)> {
        let page_str = page.to_string();
        let page_size_str = page_size.to_string();
        let mut params = vec![
            ("Page", page_str.as_str()),
            ("PageSize", page_size_str.as_str()),
        ];
        if let Some(term) = search_term {
            params.push(("SearchTerm", term));
        }
        if let Some(sort) = sort_by {
            params.push(("SortBy", sort));
        }

        let body = self
            .send_request("namecheap.domains.getList", &params)
            .await?;
        parse_domain_get_list_response(&body)
    }

    /// Get detailed information about a domain
    pub async fn get_domain_info(&self, domain: &str) -> Result<DomainInfo> {
        let body = self
            .send_request("namecheap.domains.getInfo", &[("DomainName", domain)])
            .await?;
        parse_domain_get_info_response(&body, domain)
    }

    /// Get DNS host records for a domain
    pub async fn get_dns_hosts(&self, domain: &str) -> Result<Vec<DnsHostRecord>> {
        let (sld, tld) = split_domain(domain);
        let body = self
            .send_request(
                "namecheap.domains.dns.getHosts",
                &[("SLD", &sld), ("TLD", &tld)],
            )
            .await?;
        parse_dns_get_hosts_response(&body)
    }

    /// Set DNS host records for a domain (replaces all existing records)
    pub async fn set_dns_hosts(&self, domain: &str, records: &[DnsHostRecord]) -> Result<bool> {
        let (sld, tld) = split_domain(domain);
        let mut params: Vec<(String, String)> = vec![
            ("SLD".into(), sld),
            ("TLD".into(), tld),
        ];

        for (idx, record) in records.iter().enumerate() {
            let i = idx + 1;
            params.push((format!("HostName{i}"), record.name.clone()));
            params.push((format!("RecordType{i}"), record.record_type.clone()));
            params.push((format!("Address{i}"), record.address.clone()));
            if let Some(mx) = record.mx_pref {
                params.push((format!("MXPref{i}"), mx.to_string()));
            }
            params.push((format!("TTL{i}"), record.ttl.to_string()));
        }

        let slice_params: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let body = self
            .send_request("namecheap.domains.dns.setHosts", &slice_params)
            .await?;

        Ok(body.contains("IsSuccess=\"true\""))
    }

    /// Set custom nameservers for a domain
    pub async fn set_nameservers(&self, domain: &str, nameservers: &[String]) -> Result<bool> {
        let (sld, tld) = split_domain(domain);
        let ns_str = nameservers.join(",");
        let body = self
            .send_request(
                "namecheap.domains.dns.setCustom",
                &[("SLD", &sld), ("TLD", &tld), ("Nameservers", &ns_str)],
            )
            .await?;
        Ok(body.contains("Update=\"true\"") || body.contains("IsSuccess=\"true\""))
    }

    /// Set Namecheap default nameservers
    pub async fn set_default_nameservers(&self, domain: &str) -> Result<bool> {
        let (sld, tld) = split_domain(domain);
        let body = self
            .send_request(
                "namecheap.domains.dns.setDefault",
                &[("SLD", &sld), ("TLD", &tld)],
            )
            .await?;
        Ok(body.contains("Updated=\"true\"") || body.contains("IsSuccess=\"true\""))
    }
}

/// Split domain into (SLD, TLD).
/// Example: `example.com` -> (`example`, `com`)
/// Example: `mysite.co.uk` -> (`mysite`, `co.uk`)
pub fn split_domain(domain: &str) -> (String, String) {
    let clean = domain.trim().trim_end_matches('.').to_lowercase();
    let two_part_tlds = [
        "co.uk", "org.uk", "me.uk", "com.au", "net.au", "org.au", "co.nz", "net.nz", "org.nz",
        "co.za", "com.br", "co.jp", "com.sg", "com.mx", "co.in", "net.in", "org.in",
    ];
    for tld in two_part_tlds {
        let suffix = format!(".{tld}");
        if clean.ends_with(&suffix) {
            let sld = &clean[..clean.len() - suffix.len()];
            let sld_part = sld.split('.').next_back().unwrap_or(sld);
            return (sld_part.to_string(), tld.to_string());
        }
    }
    if let Some(pos) = clean.rfind('.') {
        let pre = &clean[..pos];
        let tld = &clean[pos + 1..];
        let sld = pre.split('.').next_back().unwrap_or(pre);
        (sld.to_string(), tld.to_string())
    } else {
        (clean, String::new())
    }
}

pub fn check_api_response_errors(xml: &str) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut in_errors = false;
    let mut errors = Vec::new();
    let mut current_error_num = String::new();
    let mut current_error_text = String::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name.as_ref().eq_ignore_ascii_case("errors") {
                    in_errors = true;
                } else if in_errors && name.as_ref().eq_ignore_ascii_case("error") {
                    current_error_num.clear();
                    current_error_text.clear();
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        if key.eq_ignore_ascii_case("number") {
                            current_error_num = attr.value.as_ref().to_string();
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) if in_errors => {
                current_error_text.push_str(e.as_ref());
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref().eq_ignore_ascii_case("errors") {
                    in_errors = false;
                } else if in_errors && name.as_ref().eq_ignore_ascii_case("error") {
                    let msg = if current_error_num.is_empty() {
                        current_error_text.trim().to_string()
                    } else {
                        format!("[{}] {}", current_error_num, current_error_text.trim())
                    };
                    if !msg.is_empty() {
                        errors.push(msg);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(err) => return Err(anyhow!("Failed to parse XML response: {}", err)),
            _ => {}
        }
        buf.clear();
    }

    if !errors.is_empty() {
        return Err(anyhow!("Namecheap API error: {}", errors.join("; ")));
    }
    Ok(())
}

pub fn parse_domain_check_response(xml: &str) -> Result<Vec<DomainCheckResult>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut results = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name.as_ref().eq_ignore_ascii_case("domaincheckresult") {
                    let mut domain = String::new();
                    let mut available = false;
                    let mut is_premium = false;
                    let mut reg_price = None;
                    let mut renewal_price = None;
                    let mut icann_fee = None;

                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let val = attr.value.as_ref();
                        if key.eq_ignore_ascii_case("domain") {
                            domain = val.to_string();
                        } else if key.eq_ignore_ascii_case("available") {
                            available = val.eq_ignore_ascii_case("true");
                        } else if key.eq_ignore_ascii_case("ispremiumname") {
                            is_premium = val.eq_ignore_ascii_case("true");
                        } else if key.eq_ignore_ascii_case("premiumregistrationprice") {
                            reg_price = val.parse::<f64>().ok();
                        } else if key.eq_ignore_ascii_case("premiumrenewalprice") {
                            renewal_price = val.parse::<f64>().ok();
                        } else if key.eq_ignore_ascii_case("icannfee") {
                            icann_fee = val.parse::<f64>().ok();
                        }
                    }

                    if !domain.is_empty() {
                        results.push(DomainCheckResult {
                            domain,
                            available,
                            is_premium,
                            premium_registration_price: reg_price,
                            premium_renewal_price: renewal_price,
                            icann_fee,
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML error parsing domain check: {}", e)),
            _ => {}
        }
        buf.clear();
    }
    Ok(results)
}

pub fn parse_domain_get_list_response(xml: &str) -> Result<(Vec<DomainItem>, PagingInfo)> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut paging = PagingInfo {
        total_items: 0,
        current_page: 1,
        page_size: 20,
    };
    let mut current_tag = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                current_tag = e.name().as_ref().to_string();
                if current_tag.eq_ignore_ascii_case("domain") {
                    let mut map = HashMap::new();
                    for attr in e.attributes().flatten() {
                        let k = attr.key.as_ref().to_lowercase();
                        let v = attr.value.as_ref().to_string();
                        map.insert(k, v);
                    }
                    items.push(DomainItem {
                        id: map.get("id").and_then(|s| s.parse().ok()).unwrap_or(0),
                        name: map.get("name").cloned().unwrap_or_default(),
                        user: map.get("user").cloned().unwrap_or_default(),
                        created: map.get("created").cloned().unwrap_or_default(),
                        expires: map.get("expires").cloned().unwrap_or_default(),
                        is_expired: map
                            .get("isexpired")
                            .map(|s| s.eq_ignore_ascii_case("true"))
                            .unwrap_or(false),
                        is_locked: map
                            .get("islocked")
                            .map(|s| s.eq_ignore_ascii_case("true"))
                            .unwrap_or(false),
                        auto_renew: map
                            .get("autorenew")
                            .map(|s| s.eq_ignore_ascii_case("true"))
                            .unwrap_or(false),
                        whois_guard: map.get("whoisguard").cloned().unwrap_or_default(),
                    });
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.as_ref();
                if current_tag.eq_ignore_ascii_case("totalitems") {
                    paging.total_items = text.trim().parse().unwrap_or(0);
                } else if current_tag.eq_ignore_ascii_case("currentpage") {
                    paging.current_page = text.trim().parse().unwrap_or(1);
                } else if current_tag.eq_ignore_ascii_case("pagesize") {
                    paging.page_size = text.trim().parse().unwrap_or(20);
                }
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML error parsing domain list: {}", e)),
            _ => {}
        }
        buf.clear();
    }
    Ok((items, paging))
}

pub fn parse_domain_get_info_response(xml: &str, domain_name: &str) -> Result<DomainInfo> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut id = 0;
    let mut status = "Unknown".to_string();
    let mut created_date = String::new();
    let mut expired_date = String::new();
    let mut is_locked = false;
    let mut whois_guard_enabled = false;
    let mut is_using_our_dns = false;
    let mut nameservers = Vec::new();

    let mut current_tag = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                current_tag = e.name().as_ref().to_string();
                if current_tag.eq_ignore_ascii_case("domaingetinforesult") {
                    for attr in e.attributes().flatten() {
                        let k = attr.key.as_ref().to_lowercase();
                        let v = attr.value.as_ref().to_string();
                        if k == "id" {
                            id = v.parse().unwrap_or(0);
                        } else if k == "status" {
                            status = v;
                        }
                    }
                } else if current_tag.eq_ignore_ascii_case("whoisguard") {
                    for attr in e.attributes().flatten() {
                        let k = attr.key.as_ref().to_lowercase();
                        let v = attr.value.as_ref().to_string();
                        if k == "enabled" {
                            whois_guard_enabled = v.eq_ignore_ascii_case("true");
                        }
                    }
                } else if current_tag.eq_ignore_ascii_case("dnsdetails") {
                    for attr in e.attributes().flatten() {
                        let k = attr.key.as_ref().to_lowercase();
                        let v = attr.value.as_ref().to_string();
                        if k == "isusingourdns" {
                            is_using_our_dns = v.eq_ignore_ascii_case("true");
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.as_ref().trim().to_string();
                if current_tag.eq_ignore_ascii_case("createddate") {
                    created_date = text;
                } else if current_tag.eq_ignore_ascii_case("expireddate") {
                    expired_date = text;
                } else if current_tag.eq_ignore_ascii_case("islocked") {
                    is_locked = text.eq_ignore_ascii_case("true");
                } else if current_tag.eq_ignore_ascii_case("nameserver") && !text.is_empty() {
                    nameservers.push(text);
                }
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML error parsing domain info: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(DomainInfo {
        id,
        name: domain_name.to_string(),
        status,
        created_date,
        expired_date,
        is_locked,
        whois_guard_enabled,
        is_using_our_dns,
        nameservers,
    })
}

pub fn parse_dns_get_hosts_response(xml: &str) -> Result<Vec<DnsHostRecord>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut records = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = e.name();
                if tag.as_ref().eq_ignore_ascii_case("host") {
                    let mut map = HashMap::new();
                    for attr in e.attributes().flatten() {
                        let k = attr.key.as_ref().to_lowercase();
                        let v = attr.value.as_ref().to_string();
                        map.insert(k, v);
                    }
                    records.push(DnsHostRecord {
                        host_id: map.get("hostid").and_then(|s| s.parse().ok()),
                        name: map.get("name").cloned().unwrap_or_default(),
                        record_type: map.get("type").cloned().unwrap_or_default(),
                        address: map.get("address").cloned().unwrap_or_default(),
                        mx_pref: map.get("mxpref").and_then(|s| s.parse().ok()),
                        ttl: map.get("ttl").and_then(|s| s.parse().ok()).unwrap_or(1800),
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML error parsing DNS hosts: {}", e)),
            _ => {}
        }
        buf.clear();
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_domain() {
        assert_eq!(split_domain("example.com"), ("example".into(), "com".into()));
        assert_eq!(
            split_domain("sub.example.com"),
            ("example".into(), "com".into())
        );
        assert_eq!(
            split_domain("spacecorps.co.uk"),
            ("spacecorps".into(), "co.uk".into())
        );
        assert_eq!(
            split_domain("app.spacecorps.co.uk"),
            ("spacecorps".into(), "co.uk".into())
        );
        assert_eq!(
            split_domain("domain.com.au"),
            ("domain".into(), "com.au".into())
        );
    }

    #[test]
    fn test_check_api_response_errors() {
        let error_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ApiResponse Status="ERROR" xmlns="http://api.namecheap.com/xml.response">
  <Errors>
    <Error Number="2011166">Client IP address is not whitelisted</Error>
  </Errors>
</ApiResponse>"#;

        let err = check_api_response_errors(error_xml).unwrap_err();
        assert!(err.to_string().contains("2011166"));
        assert!(err.to_string().contains("Client IP address is not whitelisted"));

        let ok_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ApiResponse Status="OK" xmlns="http://api.namecheap.com/xml.response">
  <Errors />
</ApiResponse>"#;
        assert!(check_api_response_errors(ok_xml).is_ok());
    }

    #[test]
    fn test_parse_domain_check() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ApiResponse Status="OK" xmlns="http://api.namecheap.com/xml.response">
  <CommandResponse Type="namecheap.domains.check">
    <DomainCheckResult Domain="available-domain123.com" Available="true" IsPremiumName="false" PremiumRegistrationPrice="0" />
    <DomainCheckResult Domain="google.com" Available="false" IsPremiumName="false" PremiumRegistrationPrice="0" />
  </CommandResponse>
</ApiResponse>"#;

        let res = parse_domain_check_response(xml).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].domain, "available-domain123.com");
        assert!(res[0].available);
        assert_eq!(res[1].domain, "google.com");
        assert!(!res[1].available);
    }

    #[test]
    fn test_parse_domain_get_list() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ApiResponse Status="OK" xmlns="http://api.namecheap.com/xml.response">
  <CommandResponse Type="namecheap.domains.getList">
    <DomainGetListResult>
      <Domain ID="987" Name="my-cool-site.com" User="user1" Created="01/01/2024" Expires="01/01/2028" IsExpired="false" IsLocked="true" AutoRenew="true" WhoisGuard="ENABLED" />
    </DomainGetListResult>
    <Paging>
      <TotalItems>1</TotalItems>
      <CurrentPage>1</CurrentPage>
      <PageSize>20</PageSize>
    </Paging>
  </CommandResponse>
</ApiResponse>"#;

        let (items, paging) = parse_domain_get_list_response(xml).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "my-cool-site.com");
        assert_eq!(items[0].id, 987);
        assert_eq!(paging.total_items, 1);
    }

    #[test]
    fn test_parse_dns_hosts() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ApiResponse Status="OK" xmlns="http://api.namecheap.com/xml.response">
  <CommandResponse Type="namecheap.domains.dns.getHosts">
    <DomainDNSGetHostsResult Domain="my-cool-site.com" IsUsingOurDNS="true">
      <host HostId="1" Name="@" Type="A" Address="1.2.3.4" MXPref="10" TTL="1800" />
      <host HostId="2" Name="www" Type="CNAME" Address="my-cool-site.com." MXPref="10" TTL="1800" />
    </DomainDNSGetHostsResult>
  </CommandResponse>
</ApiResponse>"#;

        let hosts = parse_dns_get_hosts_response(xml).unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].name, "@");
        assert_eq!(hosts[0].record_type, "A");
        assert_eq!(hosts[0].address, "1.2.3.4");
        assert_eq!(hosts[1].name, "www");
        assert_eq!(hosts[1].record_type, "CNAME");
    }
}
