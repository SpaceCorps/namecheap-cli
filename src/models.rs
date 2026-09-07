use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomainCheckResult {
    pub domain: String,
    pub available: bool,
    pub is_premium: bool,
    pub premium_registration_price: Option<f64>,
    pub premium_renewal_price: Option<f64>,
    pub icann_fee: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomainItem {
    pub id: u64,
    pub name: String,
    pub user: String,
    pub created: String,
    pub expires: String,
    pub is_expired: bool,
    pub is_locked: bool,
    pub auto_renew: bool,
    pub whois_guard: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomainInfo {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub created_date: String,
    pub expired_date: String,
    pub is_locked: bool,
    pub whois_guard_enabled: bool,
    pub is_using_our_dns: bool,
    pub nameservers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DnsHostRecord {
    pub host_id: Option<u64>,
    pub name: String,
    pub record_type: String,
    pub address: String,
    pub mx_pref: Option<u32>,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagingInfo {
    pub total_items: u32,
    pub current_page: u32,
    pub page_size: u32,
}
