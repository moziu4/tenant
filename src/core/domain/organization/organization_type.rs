use crate::core::commands::organization_command::OrganizationState;
use crate::utils::domains_ids::{OrganizationID, PlanID};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Organization {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<OrganizationID>,
    pub host: String,
    pub name: String,
    pub plan: PlanID,
    pub state: OrganizationState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<OrganizationLimits>,
    pub configuration: OrganizationConfiguration,
    pub features: OrganizationFeatures,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrganizationColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrganizationConfiguration {
    #[serde(default)]
    pub logo: String,
    pub colors: OrganizationColors,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct OrganizationFeatures {
    #[serde(default)]
    pub migration: bool,
}

impl OrganizationFeatures {
    pub fn validate_against_plan(&self, plan_features: &crate::core::domain::plans::plan_type::PlanFeatures) -> Result<(), super::organization_error::OrganizationError> {
        if self.migration && !plan_features.migration {
            return Err(super::organization_error::OrganizationError::FeatureNotAllowedByPlan("migration".to_string()));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrganizationLimits {
    pub max_tenants: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewOrganization {
    pub host: String,
    pub name: String,
    pub plan: PlanID,
    pub state: OrganizationState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<OrganizationLimits>,
    pub configuration: OrganizationConfiguration,
    pub features: OrganizationFeatures,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configuration() -> OrganizationConfiguration {
        OrganizationConfiguration {
            logo: "logo.svg".to_string(),
            colors: OrganizationColors {
                primary: "#fff".to_string(),
                secondary: "#000".to_string(),
                accent: "#f00".to_string(),
                background: "#eee".to_string(),
                text: "#111".to_string(),
            },
        }
    }

    #[test]
    fn organization_deserializes_without_limits() {
        let json = r###"{
            "host":"example.com",
            "name":"Example",
            "plan":"507f1f77bcf86cd799439022",
            "state":"active",
            "configuration":{"logo":"logo.svg","colors":{"primary":"#fff","secondary":"#000","accent":"#f00","background":"#eee","text":"#111"}},
            "features":{"migration":true}
        }"###;

        let organization: Organization = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(organization.host, "example.com");
        assert!(organization.limits.is_none());
        assert!(organization.features.migration);
    }

    #[test]
    fn configuration_contains_only_logo_and_colors() {
        let configuration = configuration();
        let serialized = serde_json::to_value(configuration).expect("Should serialize");
        assert!(serialized.get("logo").is_some());
        assert!(serialized.get("colors").is_some());
        assert!(serialized.get("theme").is_none());
        assert!(serialized.get("menu_type").is_none());
    }
}
