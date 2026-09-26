pub mod nats_handler;
pub mod nats_subscriber;

use serde::{Serialize, Deserialize, Deserializer};
use crate::core::domain::tenant::tenant_type::{TenantFeatures, TenantState};

// Deserializador personalizado para manejar tanto String plano como {"$oid": "..."}
fn deserialize_mongo_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MongoId {
        String(String),
        Map { #[serde(rename = "$oid")] oid: String },
    }

    match MongoId::deserialize(deserializer)? {
        MongoId::String(s) => Ok(s),
        MongoId::Map { oid } => Ok(oid),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EntityType {
    Tenant,
    Organization,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateChangedEvent {
    pub entity_type: EntityType,
    #[serde(deserialize_with = "deserialize_mongo_id")]
    pub entity_id: String,
    pub state: TenantState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenantCreatedEvent {
    #[serde(deserialize_with = "deserialize_mongo_id")]
    pub tenant_id: String,
    pub name: String,
    pub available_languages: Vec<String>,
    pub features: TenantFeatures,
    pub organization_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupIDCreatedEvent {
    #[serde(deserialize_with = "deserialize_mongo_id")]
    pub tenant_id: String,
    #[serde(deserialize_with = "deserialize_mongo_id")]
    pub group_id: String,
    pub slug: String,
}
