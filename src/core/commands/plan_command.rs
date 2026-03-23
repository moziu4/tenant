use serde::{Deserialize, Serialize};
use crate::core::domain::plans::plan_type::{PlanState, PlanFeatures};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum PlanCommand {
    UpdateName { name: String },
    UpdatePrice { price: f64 },
    UpdateLimits { limit_tenant: i32, limit_users: i32 },
    UpdateState { state: PlanState },
    UpdateFeatures { features: PlanFeatures },
}
