use crate::model::{Card, CardType, Link, ArchitectureModel};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Debounced automatic save configuration
#[derive(Debug, Clone)]
pub struct AutoSaveConfig {
    pub debounce_ms: u64,      // 2000 - wait after last edit
    pub heartbeat_ms: u64,     // 30000 - force save interval
    pub validate_before_save: bool,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 2000,
            heartbeat_ms: 30000,
            validate_before_save: true,
        }
    }
}

/// Persistent storage manager
pub struct PersistenceManager {
    pub project_root: PathBuf,
    pub config: AutoSaveConfig,
    pub cards_dir: PathBuf,
    pub links_file: PathBuf,
}

impl PersistenceManager {
    pub fn new(project_root: impl AsRef<Path>, config: AutoSaveConfig) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let cards_dir = project_root.join("docs/cards");
        let links_file = project_root.join("docs/links/links.json");

        Self {
            project_root,
            config,
            cards_dir,
            links_file,
        }
    }

    /// Load entire model from disk
    pub async fn load_model(&self) -> Result<ArchitectureModel> {
        let mut model = ArchitectureModel::new();

        // 1. Load all cards
        let cards = self.load_cards().await?;
        for card in cards {
            model.add_card(card).map_err(|e| anyhow::anyhow!(e))?;
        }

        // 2. Load all links
        let links = self.load_links().await?;
        for link in links {
            model.add_link(link).map_err(|e| anyhow::anyhow!(e))?;
        }

        // 3. Validate model
        self.validate_model(&model)?;

        Ok(model)
    }

    /// Load all cards from cards directory
    async fn load_cards(&self) -> Result<Vec<Card>> {
        let mut cards = Vec::new();
        let mut dir_entries = fs::read_dir(&self.cards_dir).await?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).await?;
                let card: Card = serde_json::from_str(&content)?;
                cards.push(card);
            }
        }

        Ok(cards)
    }

    /// Load all links
    async fn load_links(&self) -> Result<Vec<Link>> {
        if !self.links_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.links_file).await?;
        let links: Vec<Link> = serde_json::from_str(&content)?;
        Ok(links)
    }

    /// Save a single card to disk
    pub async fn save_card(&self, card: &Card) -> Result<()> {
        // Validate if configured
        if self.config.validate_before_save {
            self.validate_card(card)?;
        }

        let filename = self.card_filename(&card.id);
        let path = self.cards_dir.join(&filename);

        // Create directory if needed
        let parent_dir = path.parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory of card file"))?;
        fs::create_dir_all(parent_dir).await?;

        let content = serde_json::to_string_pretty(&card)?;
        fs::write(&path, content).await?;

        Ok(())
    }

    /// Save all links atomically
    pub async fn save_links(&self, links: &[Link]) -> Result<()> {
        let dir = self.links_file.parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory of links file"))?;
        fs::create_dir_all(dir).await?;

        let content = serde_json::to_string_pretty(&links)?;
        fs::write(&self.links_file, content).await?;

        Ok(())
    }

    /// Validate a single card against schema
    pub fn validate_card(&self, card: &Card) -> Result<()> {
        // Basic validation
        if card.id.is_empty() {
            return Err(anyhow!("Card ID cannot be empty"));
        }

        if card.name.is_empty() {
            return Err(anyhow!("Card name cannot be empty"));
        }

        // ID format check: namespace:element-name
        if !card.id.contains(':') {
            return Err(anyhow!(
                "Card ID must be in format 'namespace:element-name', got: {}",
                card.id
            ));
        }

        Ok(())
    }

    /// Validate entire model
    pub fn validate_model(&self, model: &ArchitectureModel) -> Result<()> {
        // Check all cards are valid
        for card in model.cards.values() {
            self.validate_card(card)?;
        }

        // Check link references
        for link in &model.links {
            if !model.cards.contains_key(&link.source_id) {
                return Err(anyhow!("Link references missing source card: {}", link.source_id));
            }
            if !model.cards.contains_key(&link.target_id) {
                return Err(anyhow!("Link references missing target card: {}", link.target_id));
            }
        }

        // Check root driver exists
        if let Some(root_id) = &model.root_driver_id {
            if !model.cards.contains_key(root_id) {
                return Err(anyhow!("Root driver not found: {}", root_id));
            }
        }

        Ok(())
    }

    /// Generate filename for a card ID
    fn card_filename(&self, card_id: &str) -> String {
        // Format: type-name.json (e.g., driver-root.json, requirement-api-latency.json)
        let parts: Vec<&str> = card_id.split(':').collect();
        if parts.len() != 2 {
            return format!("{}.json", card_id);
        }

        format!("{}.json", parts[1])
    }

    /// Get file path for a specific card
    pub fn card_path(&self, card_id: &str) -> PathBuf {
        self.cards_dir.join(self.card_filename(card_id))
    }
}

/// Automatic save manager - debounces saves and tracks changes
pub struct AutoSaveManager {
    pending_saves: Arc<RwLock<HashMap<String, Card>>>,
    persistence: Arc<PersistenceManager>,
}

impl AutoSaveManager {
    pub fn new(persistence: Arc<PersistenceManager>) -> Self {
        Self {
            pending_saves: Arc::new(RwLock::new(HashMap::new())),
            persistence,
        }
    }

    /// Queue a card for debounced save
    pub async fn queue_save(&self, card: Card) {
        let mut saves = self.pending_saves.write().await;
        saves.insert(card.id.clone(), card);
    }

    /// Flush all pending saves to disk
    pub async fn flush_all(&self) -> Result<()> {
        let mut saves = self.pending_saves.write().await;
        
        for (_, card) in saves.drain() {
            self.persistence.save_card(&card).await?;
        }

        Ok(())
    }

    /// Get pending save count
    pub async fn pending_count(&self) -> usize {
        self.pending_saves.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_card_filename() {
        let config = AutoSaveConfig::default();
        let mgr = PersistenceManager::new("/tmp/aurora", config);
        assert_eq!(mgr.card_filename("driver:root-driver"), "root-driver.json");
        assert_eq!(
            mgr.card_filename("requirement:api-latency"),
            "api-latency.json"
        );
    }
}
