use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum OrganizationError {
    OrganizationNotFound,
    OrganizationDocumentNotCreated,
    OrganizationDocNotUpdated,
    EmptyName,
    RedisError(String),
    NotHasPermission,
    OrganizationMustBeInactiveToDelete,
}

impl std::fmt::Display for OrganizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrganizationError::OrganizationNotFound => write!(f, "Organization not found"),
            OrganizationError::OrganizationDocumentNotCreated => write!(f, "Organization document not created"),
            OrganizationError::OrganizationDocNotUpdated => write!(f, "Organization document not updated"),
            OrganizationError::EmptyName => write!(f, "Organization name cannot be empty"),
            OrganizationError::RedisError(e) => write!(f, "Redis error: {}", e),
            OrganizationError::NotHasPermission => write!(f, "You do not have permission to perform this action"),
            OrganizationError::OrganizationMustBeInactiveToDelete => write!(f, "Organization must be inactive before it can be deleted"),
        }
    }
}

impl From<mongodb::error::Error> for OrganizationError {
    fn from(_: mongodb::error::Error) -> Self {
        OrganizationError::OrganizationNotFound
    }
}

impl From<mongodb::bson::de::Error> for OrganizationError {
    fn from(_: mongodb::bson::de::Error) -> Self {
        OrganizationError::OrganizationNotFound
    }
}
