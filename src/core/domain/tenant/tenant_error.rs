use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum TenantError {
    TenantNotFound,
    TenantDocumentNotCreated,
    TenantDocNotUpdated,
    EmptyName,
    RedisError(String),
    NotHasPermission,
    TenantMustBeInactiveToDelete,
    OnlyOneActiveMainMenu,
    MenuNotFound,
    MenuItemNotFound,
    FeatureNotAllowedByPlan(String),
    FeatureNotAllowedByTenantPlan(String),
    TenantLimitReached,
    OrganizationNotFound,
    OrganizationInactive,
    PlanNotFound,
    TenantPlanNotFound,
    TenantPlanInactive,
    TenantPlanNotAllowedForOrganization,
    InvalidTenantPlan(String),
}

impl std::fmt::Display for TenantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantError::TenantNotFound => write!(f, "Tenant not found"),
            TenantError::TenantDocumentNotCreated => write!(f, "Tenant document not created"),
            TenantError::TenantDocNotUpdated => write!(f, "Tenant document not updated"),
            TenantError::EmptyName => write!(f, "Tenant name cannot be empty"),
            TenantError::RedisError(e) => write!(f, "Redis error: {}", e),
            TenantError::NotHasPermission => write!(f, "You do not have permission to perform this action"),
            TenantError::TenantMustBeInactiveToDelete => write!(f, "Tenant must be inactive before it can be deleted"),
            TenantError::OnlyOneActiveMainMenu => write!(f, "Only one main menu can be active at the same time"),
            TenantError::MenuNotFound => write!(f, "Menu not found"),
            TenantError::MenuItemNotFound => write!(f, "Menu item not found"),
            TenantError::FeatureNotAllowedByPlan(feature) => write!(f, "Feature '{}' is not allowed by the organization's plan", feature),
            TenantError::FeatureNotAllowedByTenantPlan(feature) => write!(f, "Feature '{}' is not allowed by the tenant's plan", feature),
            TenantError::TenantLimitReached => write!(f, "Tenant limit reached for this organization's plan"),
            TenantError::OrganizationNotFound => write!(f, "Organization not found"),
            TenantError::OrganizationInactive => write!(f, "Organization is inactive"),
            TenantError::PlanNotFound => write!(f, "Plan not found"),
            TenantError::TenantPlanNotFound => write!(f, "Tenant plan not found"),
            TenantError::TenantPlanInactive => write!(f, "Tenant plan is inactive"),
            TenantError::TenantPlanNotAllowedForOrganization => write!(f, "Tenant plan is not assigned to this organization"),
            TenantError::InvalidTenantPlan(msg) => write!(f, "Invalid tenant plan: {}", msg),
        }
    }
}

impl From<mongodb::error::Error> for TenantError {
    fn from(_: mongodb::error::Error) -> Self {
        TenantError::TenantNotFound
    }
}
