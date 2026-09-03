
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeWallet {
    pub id: Uuid,
    pub login: String,                 // fee wallet or account
    pub worker_suffix: Option<String>, // e.g. ".fee"
    pub rate: f64,                     // 0.05 = 5%
    pub upstream_addr: Option<String>, // optional alternate pool host:port (e.g. "btc.f2pool.com:1314")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortConfig {
    pub id: Uuid,
    pub listen_port: u16,
    pub upstream_addr: String,
    pub enabled: bool,
    pub fee_wallets: Vec<FeeWallet>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StoreFile { pub ports: Vec<PortConfig> }

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MinerInfo {
    pub peer: String,
    pub login: Option<String>,
    pub parsed_wallet: Option<String>,
    pub parsed_worker: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_submit_at: Option<DateTime<Utc>>,
    pub diff: f64,
    pub total_shares: u64,
    pub fee_shares: u64,
    pub rejected_shares: u64,
    pub h15m_hs: f64,
    pub online_secs: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitRecord { pub ts: DateTime<Utc>, pub diff: f64, pub is_fee: bool }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub ports: Vec<(Uuid, u16)>,
    pub total_miners: usize,
    pub total_shares: u64,
    pub total_fee_shares: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeLogEntry {
    pub ts: DateTime<Utc>,
    pub wallet_login: String,
    pub miner_worker: Option<String>,
    pub rate: f64,
    pub upstream_addr: String,
    pub accepted: bool,
    pub message: Option<String>,
}
