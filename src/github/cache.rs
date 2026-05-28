use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

use crate::config::Config;
use crate::state::RepoInfo;

const CACHE_TTL_SECS: u64 = 3600;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    cached_at: DateTime<Utc>,
    repos: Vec<RepoInfo>,
}

pub struct RepoCache {
    cache_dir: PathBuf,
    memory: LruCache<String, (Vec<RepoInfo>, Instant)>,
}

impl RepoCache {
    pub fn new() -> Self {
        let cache_dir = Config::config_dir();
        let _ = std::fs::create_dir_all(&cache_dir);
        Self {
            cache_dir,
            memory: LruCache::new(NonZeroUsize::new(10).unwrap()),
        }
    }

    pub fn load_org(&mut self, org: &str) -> Option<Vec<RepoInfo>> {
        if let Some((repos, cached_at)) = self.memory.get(org) {
            if cached_at.elapsed() < Duration::from_secs(CACHE_TTL_SECS) {
                return Some(repos.clone());
            }
        }

        let path = self.cache_dir.join(format!("{org}.json"));
        let contents = std::fs::read_to_string(&path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&contents).ok()?;

        let age = Utc::now().signed_duration_since(entry.cached_at);
        if age.num_seconds() > CACHE_TTL_SECS as i64 {
            return None;
        }

        self.memory
            .put(org.to_string(), (entry.repos.clone(), Instant::now()));
        Some(entry.repos)
    }

    pub fn store_org(&mut self, org: &str, repos: &[RepoInfo]) {
        let entry = CacheEntry {
            cached_at: Utc::now(),
            repos: repos.to_vec(),
        };
        if let Ok(json) = serde_json::to_string(&entry) {
            let path = self.cache_dir.join(format!("{org}.json"));
            let _ = std::fs::write(path, json);
        }
        self.memory
            .put(org.to_string(), (repos.to_vec(), Instant::now()));
    }

    pub fn invalidate_org(&mut self, org: &str) {
        self.memory.pop(org);
        let path = self.cache_dir.join(format!("{org}.json"));
        let _ = std::fs::remove_file(path);
    }
}
