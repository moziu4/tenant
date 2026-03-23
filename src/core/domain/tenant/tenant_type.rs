use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{AgencyID, TenantID};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenantColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenantConfiguration {
    #[serde(default)]
    pub logo: String,
    pub colors: TenantColors,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub menu_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tenant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<TenantID>,
    pub host: String,
    pub name: String,
    pub agency_id: AgencyID,
    pub configuration: TenantConfiguration,
    pub state: TenantState,
    pub features: TenantFeatures,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TenantState {
    Active,
    Inactive,
    Trial,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenantFeatures {
    pub shop: bool,
    pub blog: bool,
    pub academy: bool,
    pub analytics: bool,
    pub custom: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTenant {
    pub host: String,
    pub name: String,
    pub agency_id: AgencyID,
    pub configuration: TenantConfiguration,
    pub state: TenantState,
    pub features: TenantFeatures,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadTenantsByAgency {
    pub agency_id: AgencyID,
}
