//! Seat Browser tab ledger. Cap 3. Runtime is SoT; Shell/overlay display only.

use serde::{Deserialize, Serialize};
use url::Url;

pub const MAX_TABS: usize = 3;
const TAB_URL_CAP: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabRecord {
    pub id: u32,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabState {
    #[serde(default = "default_next_id")]
    pub next_id: u32,
    #[serde(default)]
    pub active: Option<u32>,
    #[serde(default)]
    pub items: Vec<TabRecord>,
}

fn default_next_id() -> u32 {
    1
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            next_id: 1,
            active: None,
            items: Vec::new(),
        }
    }
}

#[must_use]
pub fn tab_host(url: &str) -> String {
    if url == "about:blank" || url.starts_with("about:blank?") {
        return "blank".into();
    }
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

#[must_use]
pub fn bounded_tab_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.len() <= TAB_URL_CAP {
        return trimmed.to_string();
    }
    trimmed[..TAB_URL_CAP].to_string()
}

#[must_use]
pub fn sanitize_tabs(mut state: TabState) -> TabState {
    let mut seen = std::collections::BTreeSet::new();
    state.items.retain(|tab| tab.id != 0 && seen.insert(tab.id));
    state.items.truncate(MAX_TABS);
    for tab in &mut state.items {
        tab.url = bounded_tab_url(&tab.url);
    }
    let max_id = state.items.iter().map(|tab| tab.id).max().unwrap_or(0);
    if state.next_id <= max_id {
        state.next_id = max_id.saturating_add(1).max(1);
    }
    if state.next_id == 0 {
        state.next_id = 1;
    }
    if let Some(active) = state.active {
        if state.get(active).is_none() {
            state.active = state.items.last().map(|tab| tab.id);
        }
    } else if !state.items.is_empty() {
        state.active = state.items.last().map(|tab| tab.id);
    }
    state
}

impl TabState {
    pub fn get(&self, id: u32) -> Option<&TabRecord> {
        self.items.iter().find(|tab| tab.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut TabRecord> {
        self.items.iter_mut().find(|tab| tab.id == id)
    }

    pub fn active_tab(&self) -> Option<&TabRecord> {
        self.active.and_then(|id| self.get(id))
    }

    pub fn open(&mut self, url: String) -> Result<u32, &'static str> {
        if self.items.len() >= MAX_TABS {
            return Err("tab_limit");
        }
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1).max(1);
        self.items.push(TabRecord {
            id,
            url: bounded_tab_url(&url),
        });
        self.active = Some(id);
        Ok(id)
    }

    pub fn set_active_url(&mut self, url: &str) -> Result<u32, &'static str> {
        if self.items.is_empty() {
            return self.open(url.to_string());
        }
        let id = self.active.ok_or("no_active_tab")?;
        let tab = self.get_mut(id).ok_or("unknown_tab")?;
        tab.url = bounded_tab_url(url);
        Ok(id)
    }

    pub fn select(&mut self, id: u32) -> Result<u32, &'static str> {
        if self.get(id).is_none() {
            return Err("unknown_tab");
        }
        self.active = Some(id);
        Ok(id)
    }

    pub fn close(&mut self, id: u32) -> Result<Option<u32>, &'static str> {
        let before = self.items.len();
        self.items.retain(|tab| tab.id != id);
        if self.items.len() == before {
            return Err("unknown_tab");
        }
        if self.active == Some(id) {
            self.active = self.items.last().map(|tab| tab.id);
        }
        Ok(self.active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_three_and_close_selects_remaining() {
        let mut state = TabState::default();
        state.open("https://app.eliteew.ru/".into()).unwrap();
        state.open("https://auth.eliteew.ru/".into()).unwrap();
        state.open("https://meet.eliteew.ru/".into()).unwrap();
        assert_eq!(state.items.len(), 3);
        assert_eq!(state.open("https://app.eliteew.ru/x".into()), Err("tab_limit"));
        let remain = state.close(2).unwrap();
        assert_eq!(remain, Some(3));
        assert_eq!(state.items.len(), 2);
    }
}
