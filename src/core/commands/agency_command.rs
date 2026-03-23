use serde::{Deserialize, Serialize};

use crate::utils::domains_ids::PlanID;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum AgencyCommand {
    UpdateName { name: String },
    UpdateState { state: AgencyState },
    UpdatePlan { plan: PlanID },
    AddUser { user_id: String },
    RemoveUser { user_id: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgencyState {
    Active,
    Inactive,
}
