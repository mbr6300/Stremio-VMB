use std::sync::Arc;
use tokio::sync::RwLock;

use super::metadata_provider::{MetadataProvider, MetadataResult, SearchQuery};
use super::tmdb::TmdbProvider;

pub struct MetadataService {
    providers: RwLock<Vec<Arc<dyn MetadataProvider>>>,
}

impl MetadataService {
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(Vec::new()),
        }
    }

    pub async fn register_provider(&self, provider: Arc<dyn MetadataProvider>) {
        let mut providers = self.providers.write().await;
        providers.retain(|p| p.name() != provider.name());
        providers.push(provider);
        log::info!("Registered metadata provider: {}", providers.last().unwrap().name());
    }

    pub async fn set_tmdb_key(&self, api_key: String) {
        self.register_provider(Arc::new(TmdbProvider::new(api_key))).await;
    }

    pub async fn search(
        &self,
        title: &str,
        year: Option<u16>,
        media_type: &str,
    ) -> Result<Vec<MetadataResult>, String> {
        let query = SearchQuery {
            title: title.to_string(),
            year,
            media_type: media_type.to_string(),
        };

        let providers = self.providers.read().await;
        for provider in providers.iter() {
            match provider.search(&query).await {
                Ok(results) if !results.is_empty() => return Ok(results),
                Ok(_) => continue,
                Err(e) => {
                    log::warn!("Provider {} search failed: {}", provider.name(), e);
                    continue;
                }
            }
        }

        Ok(Vec::new())
    }

    pub async fn fetch_details(
        &self,
        provider_id: i64,
        media_type: &str,
        provider_name: Option<&str>,
    ) -> Result<Option<MetadataResult>, String> {
        let providers = self.providers.read().await;

        if let Some(name) = provider_name {
            if let Some(provider) = providers.iter().find(|p| p.name() == name) {
                return provider.fetch_details(provider_id, media_type).await;
            }
            return Err(format!("Provider '{}' not registered", name));
        }

        for provider in providers.iter() {
            match provider.fetch_details(provider_id, media_type).await {
                Ok(Some(result)) => return Ok(Some(result)),
                Ok(None) => continue,
                Err(e) => {
                    log::warn!("Provider {} fetch_details failed: {}", provider.name(), e);
                    continue;
                }
            }
        }

        Ok(None)
    }

    pub async fn has_providers(&self) -> bool {
        !self.providers.read().await.is_empty()
    }
}
