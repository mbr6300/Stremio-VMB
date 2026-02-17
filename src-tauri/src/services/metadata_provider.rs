use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastMember {
    pub id: Option<i64>,
    pub name: String,
    pub character: Option<String>,
    pub profile_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewMember {
    pub name: String,
    pub job: String,
    pub department: String,
    pub profile_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastCrew {
    pub cast: Vec<CastMember>,
    pub crew: Vec<CrewMember>,
}

#[derive(Debug, Clone)]
pub struct MetadataResult {
    pub provider_id: i64,
    pub title: String,
    pub overview: String,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub release_date: Option<String>,
    pub rating: Option<f64>,
    pub runtime: Option<i64>,
    pub genres: Vec<String>,
    pub cast_crew: Option<CastCrew>,
    pub raw_response: String,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub title: String,
    pub year: Option<u16>,
    pub media_type: String,
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn name(&self) -> &str;

    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<MetadataResult>, String>;

    async fn fetch_details(
        &self,
        provider_id: i64,
        media_type: &str,
    ) -> Result<Option<MetadataResult>, String>;
}
