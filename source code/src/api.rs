
use crate::models::*;
use crate::proxy::{spawn_port, stop_port, ProxyRegistry};
use crate::state::{ok, err, Store};
use axum::{extract::{Path, State}, routing::{get, post, put, delete}, Json, Router};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppCtx {
    pub store: Store,
    pub reg: ProxyRegistry,
}

pub fn app_router(ctx: AppCtx) -> Router {
    Router::new()
        .route("/api/dashboard", get(dashboard))
        .route("/api/ports", get(list_ports).post(create_port))
        .route("/api/ports/:id", put(update_port).delete(delete_port))
        .route("/api/ports/:id/miners", get(list_miners))
        .route("/api/ports/:id/wallets", post(add_wallet).get(list_wallets))
        .route("/api/ports/:id/wallets/:wid", delete(remove_wallet).put(update_wallet))
        .route("/api/ports/:id/fee-log", get(get_fee_log))
        .with_state(ctx)
}

async fn dashboard(State(ctx): State<AppCtx>) -> Json<serde_json::Value> {
    let ports = ctx.store.list_ports();
    let mut total_miners = 0usize;
    let mut total_shares = 0u64;
    let mut fee_shares = 0u64;
    for p in &ports {
        if let Some(rt) = ctx.reg.ports.get(&p.id) {
            total_miners += rt.miners.len();
            for m in rt.miners.iter() {
                total_shares += m.value().info.total_shares;
                fee_shares += m.value().info.fee_shares;
            }
        }
    }
    ok(&DashboardSummary {
        ports: ports.iter().map(|p| (p.id, p.listen_port)).collect(),
        total_miners, total_shares, total_fee_shares: fee_shares,
    })
}

async fn list_ports(State(ctx): State<AppCtx>) -> Json<serde_json::Value> { ok(&ctx.store.list_ports()) }

#[derive(Deserialize)]
struct CreatePortReq { listen_port: u16, upstream_addr: String }

async fn create_port(State(ctx): State<AppCtx>, Json(req): Json<CreatePortReq>) -> Json<serde_json::Value> {
    let cfg = PortConfig {
        id: Uuid::new_v4(),
        listen_port: req.listen_port,
        upstream_addr: req.upstream_addr,
        enabled: true,
        fee_wallets: vec![],
    };
    if let Err(e) = ctx.store.upsert_port(cfg.clone()) { return err(&format!("store: {e}")); }
    if let Err(e) = spawn_port(ctx.reg.clone(), cfg).await { return err(&format!("spawn: {e}")); }
    ok(&"created")
}

#[derive(Deserialize)]
struct UpdatePortReq { listen_port: Option<u16>, upstream_addr: Option<String>, enabled: Option<bool> }

async fn update_port(State(ctx): State<AppCtx>, Path(id): Path<Uuid>, Json(req): Json<UpdatePortReq>) -> Json<serde_json::Value> {
    let ports = ctx.store.list_ports();
    if let Some(mut cfg) = ports.into_iter().find(|p| p.id == id) {
        if let Some(v) = req.listen_port { cfg.listen_port = v; }
        if let Some(v) = req.upstream_addr { cfg.upstream_addr = v; }
        if let Some(v) = req.enabled { cfg.enabled = v; }
        if let Err(e) = ctx.store.upsert_port(cfg.clone()) { return err(&format!("store: {e}")); }
        // restart
        stop_port(&ctx.reg, id);
        if cfg.enabled { if let Err(e) = spawn_port(ctx.reg.clone(), cfg).await { return err(&format!("spawn: {e}")); } }
        ok(&"updated")
    } else { err("not found") }
}

async fn delete_port(State(ctx): State<AppCtx>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    stop_port(&ctx.reg, id);
    if let Err(e) = ctx.store.delete_port(id) { return err(&format!("store: {e}")); }
    ok(&"deleted")
}

#[derive(Deserialize)]
struct AddWalletReq { login: String, worker_suffix: Option<String>, rate: f64, upstream_addr: Option<String> }

async fn add_wallet(State(ctx): State<AppCtx>, Path(id): Path<Uuid>, Json(req): Json<AddWalletReq>) -> Json<serde_json::Value> {
    let ports = ctx.store.list_ports();
    if let Some(mut cfg) = ports.into_iter().find(|p| p.id == id) {
        let fw = FeeWallet { id: Uuid::new_v4(), login: req.login, worker_suffix: req.worker_suffix, rate: req.rate.max(0.0), upstream_addr: req.upstream_addr };
        cfg.fee_wallets.push(fw);
        if let Err(e) = ctx.store.upsert_port(cfg.clone()) { return err(&format!("store: {e}")); }
        stop_port(&ctx.reg, id);
        if cfg.enabled { if let Err(e) = spawn_port(ctx.reg.clone(), cfg).await { return err(&format!("spawn: {e}")); } }
        ok(&"wallet added")
    } else { err("port not found") }
}

async fn remove_wallet(State(ctx): State<AppCtx>, Path((id, wid)): Path<(Uuid, Uuid)>) -> Json<serde_json::Value> {
    let ports = ctx.store.list_ports();
    if let Some(mut cfg) = ports.into_iter().find(|p| p.id == id) {
        cfg.fee_wallets.retain(|w| w.id != wid);
        if let Err(e) = ctx.store.upsert_port(cfg.clone()) { return err(&format!("store: {e}")); }
        stop_port(&ctx.reg, id);
        if cfg.enabled { if let Err(e) = spawn_port(ctx.reg.clone(), cfg).await { return err(&format!("spawn: {e}")); } }
        ok(&"wallet removed")
    } else { err("port not found") }
}

#[derive(Deserialize)]
struct UpdateWalletReq { login: Option<String>, worker_suffix: Option<String>, rate: Option<f64>, upstream_addr: Option<String> }

async fn update_wallet(State(ctx): State<AppCtx>, Path((id, wid)): Path<(Uuid, Uuid)>, Json(req): Json<UpdateWalletReq>) -> Json<serde_json::Value> {
    let ports = ctx.store.list_ports();
    if let Some(mut cfg) = ports.into_iter().find(|p| p.id == id) {
        if let Some(w) = cfg.fee_wallets.iter_mut().find(|w| w.id == wid) {
            if let Some(v) = req.login { w.login = v; }
            if let Some(v) = req.worker_suffix { w.worker_suffix = Some(v); }
            if let Some(v) = req.rate { w.rate = v.max(0.0); }
            if let Some(v) = req.upstream_addr { w.upstream_addr = Some(v); }
        } else { return err("wallet not found"); }
        if let Err(e) = ctx.store.upsert_port(cfg.clone()) { return err(&format!("store: {e}")); }
        stop_port(&ctx.reg, id);
        if cfg.enabled { if let Err(e) = spawn_port(ctx.reg.clone(), cfg).await { return err(&format!("spawn: {e}")); } }
        ok(&"wallet updated")
    } else { err("port not found") }
}

async fn list_wallets(State(ctx): State<AppCtx>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let ports = ctx.store.list_ports();
    if let Some(cfg) = ports.into_iter().find(|p| p.id == id) {
        ok(&cfg.fee_wallets)
    } else { err("port not found") }
}

async fn list_miners(State(ctx): State<AppCtx>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    if let Some(rt) = ctx.reg.ports.get(&id) {
        let mut miners: Vec<MinerInfo> = Vec::new();
        for m in rt.miners.iter() { miners.push(m.value().info.clone()); }
        ok(&miners)
    } else { err("port not running") }
}

async fn get_fee_log(State(ctx): State<AppCtx>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    if let Some(rt) = ctx.reg.ports.get(&id) {
        let q = rt.fee_log.lock();
        let mut v: Vec<FeeLogEntry> = q.iter().cloned().collect();
        let n = 500usize;
        if v.len() > n { v = v[v.len()-n..].to_vec(); }
        ok(&v)
    } else { err("port not running") }
}
