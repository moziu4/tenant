use serde::{Deserialize, Serialize};

use crate::utils::domains_ids::PlanID;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum OrganizationCommand {
    UpdateName { name: String },
    UpdateState { state: OrganizationState },
    UpdatePlan { plan: PlanID },
    
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationState {
    Active,
    Inactive,
}
