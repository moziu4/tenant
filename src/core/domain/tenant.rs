pub mod tenant_type;
pub mod tenant_error;

use crate::data::access::tenant_repo::MongoTenantRepo;
use crate::utils::domains_ids::TenantID;
use self::tenant_type::{Tenant, NewTenant, TenantState, TenantFeatures, TenantConfiguration};
use self::tenant_error::TenantError;

pub struct TenantEntity<'a> {
    props: Tenant,
    repo:  &'a MongoTenantRepo,
}

impl<'a> TenantEntity<'a> {
    pub fn new(new_tenant: NewTenant, repo: &'a MongoTenantRepo) -> Self {
        Self {
            repo,
            props: Tenant {
                id: None,
                host: new_tenant.host,
                name: new_tenant.name,
                agency_id: new_tenant.agency_id,
                configuration: new_tenant.configuration,
                state: new_tenant.state,
                features: new_tenant.features,
            },
        }
    }

    pub async fn create(self) -> Result<Tenant, TenantError> {
        self.repo.create(self.props).await
    }

    pub async fn load_by_id(tenant_id: TenantID, repo: &'a MongoTenantRepo) -> Result<Self, TenantError> {
        let tenant = repo.fetch_by_id(tenant_id).await?;
        Ok(Self { props: tenant, repo })
    }

    pub fn update_name(&mut self, name: String) -> Result<(), TenantError> {
        if name.is_empty() {
            return Err(TenantError::EmptyName);
        }
        self.props.name = name;
        Ok(())
    }

    pub fn update_state(&mut self, state: TenantState) -> Result<(), TenantError> {
        self.props.state = state;
        Ok(())
    }

    pub fn update_configuration(&mut self, configuration: TenantConfiguration) -> Result<(), TenantError> {
        self.props.configuration = configuration;
        Ok(())
    }

    pub fn update_features(&mut self, features: TenantFeatures) -> Result<(), TenantError> {
        self.props.features = features;
        Ok(())
    }

    pub fn get_props(&self) -> &Tenant {
        &self.props
    }
}
