pub mod organization_type;
pub mod organization_error;

use crate::data::access::organization_repo::MongoOrganizationRepo;
use crate::utils::domains_ids::{OrganizationID, PlanID};
use self::organization_type::{Organization, NewOrganization};
use self::organization_error::OrganizationError;
use crate::core::commands::organization_command::OrganizationState;

pub struct OrganizationEntity<'a> {
    props: Organization,
    repo:  &'a MongoOrganizationRepo,
}

impl<'a> OrganizationEntity<'a> {
    pub fn new(new_organization: NewOrganization, repo: &'a MongoOrganizationRepo) -> Self {
        Self {
            repo,
            props: Organization {
                id: None,
                name: new_organization.name,
                plan: new_organization.plan,
                state: new_organization.state,
                limits: new_organization.limits
            },
        }
    }

    pub async fn create(self) -> Result<Organization, OrganizationError> {
        self.repo.create(self.props).await
    }

    pub async fn load_by_id(organization_id: OrganizationID, repo: &'a MongoOrganizationRepo) -> Result<Self, OrganizationError> {
        let organization = repo.fetch_by_id(organization_id).await?;
        Ok(Self { props: organization, repo })
    }

    pub fn update_name(&mut self, name: String) -> Result<(), OrganizationError> {
        if name.is_empty() {
            return Err(OrganizationError::EmptyName);
        }
        self.props.name = name;
        Ok(())
    }

    pub fn update_state(&mut self, state: OrganizationState) -> Result<(), OrganizationError> {
        self.props.state = state;
        Ok(())
    }

    pub fn update_plan(&mut self, plan: PlanID) -> Result<(), OrganizationError> {
        self.props.plan = plan;
        Ok(())
    }
    

    pub fn get_props(&self) -> &Organization {
        &self.props
    }
}
