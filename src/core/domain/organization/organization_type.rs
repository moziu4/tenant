use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{OrganizationID, PlanID};
use crate::core::commands::organization_command::OrganizationState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Organization {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<OrganizationID>,
    pub name: String,
    pub plan: PlanID,
    pub state: OrganizationState,
    pub limits: OrganizationLimits,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrganizationLimits {
    pub max_tenants: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewOrganization {
    pub name: String,
    pub plan: PlanID,
    pub state: OrganizationState,
    pub limits: OrganizationLimits,

}
