use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::{TenantID, OrganizationID};

pub use perms::{Claims, Token, Role};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthLogin
{
    pub username:        String,
    pub password:        String,
    pub tenant_id:       Option<TenantID>,
    pub organization_id: Option<OrganizationID>,
}
