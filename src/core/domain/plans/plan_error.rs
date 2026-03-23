use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum PlanError {
    PlanNotFound,
    PlanDocumentNotCreated,
    PlanDocNotUpdated,
    EmptyName,
    RedisError(String),
    NotHasPermission,
    PlanInUse,
    PlanMustBeInactiveToDelete,
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::PlanNotFound => write!(f, "Plan not found"),
            PlanError::PlanDocumentNotCreated => write!(f, "Plan document not created"),
            PlanError::PlanDocNotUpdated => write!(f, "Plan document not updated"),
            PlanError::EmptyName => write!(f, "Plan name cannot be empty"),
            PlanError::RedisError(e) => write!(f, "Redis error: {}", e),
            PlanError::NotHasPermission => write!(f, "You do not have permission to perform this action"),
            PlanError::PlanInUse => write!(f, "Plan is in use and cannot be modified or deleted"),
            PlanError::PlanMustBeInactiveToDelete => write!(f, "Plan must be inactive before it can be deleted"),
        }
    }
}

impl From<mongodb::error::Error> for PlanError {
    fn from(_: mongodb::error::Error) -> Self {
        PlanError::PlanNotFound
    }
}

impl From<mongodb::bson::de::Error> for PlanError {
    fn from(_: mongodb::bson::de::Error) -> Self {
        PlanError::PlanNotFound
    }
}
