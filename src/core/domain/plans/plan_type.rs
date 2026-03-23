use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::PlanID;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plan {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<PlanID>,
    pub name: String,
    pub price: f64,
    pub limit_tenant: i32,
    pub limit_users: i32,
    pub state: PlanState,
    pub features: PlanFeatures,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlanState {
    Active,
    Inactive,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanFeatures {
    pub shop: bool,
    pub blog: bool,
    pub academy: bool,
    pub analytics: bool,
    pub custom: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewPlan {
    pub name: String,
    pub price: f64,
    pub limit_tenant: i32,
    pub limit_users: i32,
    pub state: PlanState,
    pub features: PlanFeatures,
}
