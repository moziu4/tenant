use serde::{Deserialize, Serialize};
use crate::core::domain::tenant::tenant_type::{TenantState, TenantFeatures, TenantConfiguration};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum TenantCommand {
    UpdateName { name: String },
    UpdateState { state: TenantState },
    UpdateConfiguration { configuration: TenantConfiguration },
    UpdateFeatures { features: TenantFeatures },
}
