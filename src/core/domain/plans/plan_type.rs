use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{PlanID, OrganizationID};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanTarget {
    Organization,
    Tenant,
}

pub fn default_plan_target() -> PlanTarget {
    PlanTarget::Organization
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BillingStrategy {
    /// Sin cobro al tenant / usuario (gratuito)
    Free,
    /// La organización le cobra directamente a sus usuarios / tenants
    Organization,
    /// Nuestra empresa / plataforma realiza el cobro al tenant
    Company,
    /// Reparto de cobro: una parte la cobra la organización y otra nuestra empresa (porcentajes)
    Split {
        organization_percentage: f64,
        company_percentage: f64,
    },
}

impl BillingStrategy {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            BillingStrategy::Free | BillingStrategy::Organization | BillingStrategy::Company => Ok(()),
            BillingStrategy::Split { organization_percentage, company_percentage } => {
                if *organization_percentage < 0.0 || *company_percentage < 0.0 {
                    return Err("Split percentages must be non-negative".to_string());
                }
                if (organization_percentage + company_percentage - 100.0).abs() > 0.01 {
                    return Err("Split percentages must sum to 100%".to_string());
                }
                Ok(())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plan {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<PlanID>,
    pub name: String,
    pub price: f64,
    #[serde(alias = "limit_tenants", default)]
    pub limit_tenant: i32,
    #[serde(default)]
    pub limit_users: i32,
    pub state: PlanState,
    pub features: PlanFeatures,
    #[serde(default = "default_plan_target")]
    pub target: PlanTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<OrganizationID>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_strategy: Option<BillingStrategy>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlanState {
    Active,
    Inactive,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PlanFeatures {
    #[serde(default)]
    pub migration: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewPlan {
    pub name: String,
    pub price: f64,
    #[serde(alias = "limit_tenants", default)]
    pub limit_tenant: i32,
    #[serde(default)]
    pub limit_users: i32,
    pub state: PlanState,
    pub features: PlanFeatures,
    #[serde(default = "default_plan_target")]
    pub target: PlanTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<OrganizationID>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_strategy: Option<BillingStrategy>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_billing_strategy_validation() {
        assert!(BillingStrategy::Free.validate().is_ok());
        assert!(BillingStrategy::Organization.validate().is_ok());
        assert!(BillingStrategy::Company.validate().is_ok());

        let valid_split = BillingStrategy::Split {
            organization_percentage: 70.0,
            company_percentage: 30.0,
        };
        assert!(valid_split.validate().is_ok());

        let invalid_negative = BillingStrategy::Split {
            organization_percentage: -10.0,
            company_percentage: 110.0,
        };
        assert!(invalid_negative.validate().is_err());

        let invalid_sum = BillingStrategy::Split {
            organization_percentage: 50.0,
            company_percentage: 40.0,
        };
        assert!(invalid_sum.validate().is_err());
    }

    #[test]
    fn test_plan_deserialization_defaults() {
        let json = r#"{
            "name": "Standard Org Plan",
            "price": 99.0,
            "limit_tenant": 10,
            "limit_users": 50,
            "state": "active",
            "features": { "shop": true }
        }"#;

        let plan: Plan = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(plan.target, PlanTarget::Organization);
        assert!(plan.organization_id.is_none());
        assert!(plan.billing_strategy.is_none());
    }

    #[test]
    fn test_all_billing_strategies_serialization() {
        let free = BillingStrategy::Free;
        let free_json = serde_json::to_string(&free).unwrap();
        assert_eq!(free_json, r#"{"type":"free"}"#);

        let org = BillingStrategy::Organization;
        let org_json = serde_json::to_string(&org).unwrap();
        assert_eq!(org_json, r#"{"type":"organization"}"#);

        let company = BillingStrategy::Company;
        let company_json = serde_json::to_string(&company).unwrap();
        assert_eq!(company_json, r#"{"type":"company"}"#);

        let split = BillingStrategy::Split {
            organization_percentage: 60.0,
            company_percentage: 40.0,
        };
        let split_json = serde_json::to_string(&split).unwrap();
        assert_eq!(split_json, r#"{"type":"split","organization_percentage":60.0,"company_percentage":40.0}"#);
    }
}
