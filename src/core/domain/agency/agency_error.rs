use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum AgencyError {
    AgencyNotFound,
    AgencyDocumentNotCreated,
    AgencyDocNotUpdated,
    EmptyName,
    RedisError(String),
    NotHasPermission,
    AgencyMustBeInactiveToDelete,
}

impl std::fmt::Display for AgencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgencyError::AgencyNotFound => write!(f, "Agency not found"),
            AgencyError::AgencyDocumentNotCreated => write!(f, "Agency document not created"),
            AgencyError::AgencyDocNotUpdated => write!(f, "Agency document not updated"),
            AgencyError::EmptyName => write!(f, "Agency name cannot be empty"),
            AgencyError::RedisError(e) => write!(f, "Redis error: {}", e),
            AgencyError::NotHasPermission => write!(f, "You do not have permission to perform this action"),
            AgencyError::AgencyMustBeInactiveToDelete => write!(f, "Agency must be inactive before it can be deleted"),
        }
    }
}

impl From<mongodb::error::Error> for AgencyError {
    fn from(_: mongodb::error::Error) -> Self {
        AgencyError::AgencyNotFound
    }
}

impl From<mongodb::bson::de::Error> for AgencyError {
    fn from(_: mongodb::bson::de::Error) -> Self {
        AgencyError::AgencyNotFound
    }
}
