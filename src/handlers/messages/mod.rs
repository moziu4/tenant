pub mod nats_handler;

use serde::{Serialize, Deserialize};
use crate::core::domain::tenant::tenant_type::TenantState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EntityType {
    Tenant,
    Agency,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateChangedEvent {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub state: TenantState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenantCreatedEvent {
    pub tenant_id: String,
    pub name: String,
}
