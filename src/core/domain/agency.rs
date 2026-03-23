pub mod agency_type;
pub mod agency_error;

use crate::data::access::agency_repo::MongoAgencyRepo;
use crate::utils::domains_ids::{AgencyID, PlanID};
use self::agency_type::{Agency, NewAgency};
use self::agency_error::AgencyError;
use crate::core::commands::agency_command::AgencyState;

pub struct AgencyEntity<'a> {
    props: Agency,
    repo:  &'a MongoAgencyRepo,
}

impl<'a> AgencyEntity<'a> {
    pub fn new(new_agency: NewAgency, repo: &'a MongoAgencyRepo) -> Self {
        Self {
            repo,
            props: Agency {
                id: None,
                name: new_agency.name,
                plan: new_agency.plan,
                state: new_agency.state,
                limits: new_agency.limits,
                users: new_agency.users,
            },
        }
    }

    pub async fn create(self) -> Result<Agency, AgencyError> {
        self.repo.create(self.props).await
    }

    pub async fn load_by_id(agency_id: AgencyID, repo: &'a MongoAgencyRepo) -> Result<Self, AgencyError> {
        let agency = repo.fetch_by_id(agency_id).await?;
        Ok(Self { props: agency, repo })
    }

    pub fn update_name(&mut self, name: String) -> Result<(), AgencyError> {
        if name.is_empty() {
            return Err(AgencyError::EmptyName);
        }
        self.props.name = name;
        Ok(())
    }

    pub fn update_state(&mut self, state: AgencyState) -> Result<(), AgencyError> {
        self.props.state = state;
        Ok(())
    }

    pub fn update_plan(&mut self, plan: PlanID) -> Result<(), AgencyError> {
        self.props.plan = plan;
        Ok(())
    }

    pub fn add_user(&mut self, user_id: String) -> Result<(), AgencyError> {
        if !self.props.users.contains(&user_id) {
            self.props.users.push(user_id);
        }
        Ok(())
    }

    pub fn remove_user(&mut self, user_id: String) -> Result<(), AgencyError> {
        self.props.users.retain(|u| u != &user_id);
        Ok(())
    }

    pub fn get_props(&self) -> &Agency {
        &self.props
    }
}
