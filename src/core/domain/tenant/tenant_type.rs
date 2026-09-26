use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{OrganizationID, TenantID, PlanID};
use super::menu::Menu;

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
    pub organization_id: OrganizationID,
    #[serde(default, alias = "plan", skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<PlanID>,
    pub configuration: TenantConfiguration,
    pub state: TenantState,
    pub features: TenantFeatures,
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
    pub host: String,
    pub name: String,
    pub organization_id: OrganizationID,
    #[serde(default, alias = "plan", skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<PlanID>,
    pub configuration: TenantConfiguration,
    pub state: TenantState,
    pub features: TenantFeatures,
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
mod tests {
    use super::*;
    use crate::core::domain::plans::plan_type::PlanFeatures;

    #[test]
    fn test_validate_against_plan_allowed() {
        let plan_features = PlanFeatures {
            migration: true,
        };

        let tenant_features = TenantFeatures {
            migration: true,
        };

        assert!(tenant_features.validate_against_plan(&plan_features).is_ok());
    }

    #[test]
    fn test_validate_against_plan_disallowed() {
        let plan_features = PlanFeatures {
            migration: false,
        };

        let tenant_features = TenantFeatures {
            migration: false,
        };

        let res = tenant_features.validate_against_plan(&plan_features);
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_against_plan_migration_disallowed() {
        let plan_features = PlanFeatures {
            migration: false,
        };

        let tenant_features = TenantFeatures {
            migration: true,
        };

        let res = tenant_features.validate_against_plan(&plan_features);
        assert!(res.is_err());
    }

    #[test]
    fn test_tenant_deserialization_with_plan_id() {
        let json = "{\"host\":\"my-tenant.example.com\",\"name\":\"My Tenant\",\"organization_id\":\"507f1f77bcf86cd799439011\",\"plan_id\":\"507f1f77bcf86cd799439022\",\"configuration\":{\"logo\":\"\",\"colors\":{\"primary\":\"#fff\",\"secondary\":\"#000\",\"accent\":\"#f00\",\"background\":\"#eee\",\"text\":\"#111\"},\"theme\":\"\",\"menu_type\":\"\"},\"state\":\"active\",\"features\":{\"shop\":true}}";

        let new_tenant: NewTenant = serde_json::from_str(json).expect("Should deserialize");
        assert!(new_tenant.plan_id.is_some());
        assert_eq!(new_tenant.plan_id.unwrap().to_string(), "507f1f77bcf86cd799439022");
    }

    #[test]
    fn test_tenant_deserialization_with_plan_alias() {
        let json = "{\"host\":\"my-tenant2.example.com\",\"name\":\"My Tenant 2\",\"organization_id\":\"507f1f77bcf86cd799439011\",\"plan\":\"507f1f77bcf86cd799439033\",\"configuration\":{\"logo\":\"\",\"colors\":{\"primary\":\"#fff\",\"secondary\":\"#000\",\"accent\":\"#f00\",\"background\":\"#eee\",\"text\":\"#111\"},\"theme\":\"\",\"menu_type\":\"\"},\"state\":\"active\",\"features\":{\"shop\":true}}";

        let new_tenant: NewTenant = serde_json::from_str(json).expect("Should deserialize with alias 'plan'");
        assert!(new_tenant.plan_id.is_some());
        assert_eq!(new_tenant.plan_id.unwrap().to_string(), "507f1f77bcf86cd799439033");
    }
}
