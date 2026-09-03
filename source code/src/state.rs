
use crate::models::{StoreFile, PortConfig};
use anyhow::Result;
use parking_lot::Mutex;
use serde_json::json;
use std::{fs, path::PathBuf, sync::Arc};

#[derive(Clone)]
pub struct Store {
    path: PathBuf,
    inner: Arc<Mutex<StoreFile>>,
}

impl Store {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if let Some(dir) = path.parent() { fs::create_dir_all(dir)?; }
        let inner = if path.exists() {
            let txt = fs::read_to_string(&path)?;
            let parsed: StoreFile = serde_json::from_str(&txt)?;
            Arc::new(Mutex::new(parsed))
        } else {
            Arc::new(Mutex::new(StoreFile::default()))
        };
        Ok(Self { path, inner })
    }

    pub fn list_ports(&self) -> Vec<PortConfig> { self.inner.lock().ports.clone() }

    pub fn upsert_port(&self, p: PortConfig) -> Result<()> {
        let mut g = self.inner.lock();
        if let Some(i) = g.ports.iter().position(|x| x.id == p.id) { g.ports[i] = p; } else { g.ports.push(p); }
        self.save_locked(&g)
    }

    pub fn delete_port(&self, id: uuid::Uuid) -> Result<()> {
        let mut g = self.inner.lock();
        g.ports.retain(|x| x.id != id);
        self.save_locked(&g)
    }

    fn save_locked(&self, g: &StoreFile) -> Result<()> {
        let txt = serde_json::to_string_pretty(g)?;
        fs::write(&self.path, txt)?;
        Ok(())
    }
}

pub fn ok<T: serde::Serialize>(v: &T) -> axum::Json<serde_json::Value> {
    axum::Json(json!({ "ok": true, "data": v }))
}
pub fn err(msg: &str) -> axum::Json<serde_json::Value> {
    axum::Json(json!({ "ok": false, "error": msg }))
}
