use super::menu::Menu;
use crate::utils::domains_ids::{OrganizationID, PlanID, TenantID};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tenant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<TenantID>,
    pub organization_id: OrganizationID,
    #[serde(default, alias = "plan", skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<PlanID>,
    pub state: TenantState,
    #[serde(default)]
    pub registration_type: RegistrationType,
    #[serde(default)]
    pub default_language: String,
    #[serde(default)]
    pub available_languages: Vec<String>,
    #[serde(default)]
    pub menus: Vec<Menu>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TenantState {
    Active,
    Inactive,
    Trial,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationType {
    SelfService,
    Deplinking,
}

impl Default for RegistrationType {
    fn default() -> Self {
        RegistrationType::SelfService
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TenantFeatures {
    #[serde(default)]
    pub migration: bool,
}

impl TenantFeatures {
    pub fn validate_against_plan(&self, plan_features: &crate::core::domain::plans::plan_type::PlanFeatures) -> Result<(), super::tenant_error::TenantError> {
        if self.migration && !plan_features.migration {
            return Err(super::tenant_error::TenantError::FeatureNotAllowedByPlan("migration".to_string()));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTenant {
    pub organization_id: OrganizationID,
    #[serde(default, alias = "plan", skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<PlanID>,
    pub state: TenantState,
    #[serde(default)]
    pub registration_type: RegistrationType,
    #[serde(default)]
    pub default_language: String,
    #[serde(default)]
    pub available_languages: Vec<String>,
    #[serde(default)]
    pub menus: Vec<Menu>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadTenantsByOrganization {
    pub organization_id: OrganizationID,
}

#[cfg(test)]
mod tests {}
