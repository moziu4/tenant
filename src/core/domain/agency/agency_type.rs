use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{AgencyID, PlanID};
use crate::core::commands::agency_command::AgencyState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agency {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<AgencyID>,
    pub name: String,
    pub plan: PlanID,
    pub state: AgencyState,
    pub limits: AgencyLimits,
    pub users: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgencyLimits {
    pub max_tenants: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewAgency {
    pub name: String,
    pub plan: PlanID,
    pub state: AgencyState,
    pub limits: AgencyLimits,
    #[serde(default)]
    pub users: Vec<String>,
}
