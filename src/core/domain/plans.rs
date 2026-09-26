pub mod plan_type;
pub mod plan_error;

use crate::data::access::plan_repo::MongoPlanRepo;
use crate::utils::domains_ids::{PlanID, OrganizationID};
use self::plan_type::{Plan, NewPlan, PlanState, PlanFeatures, PlanTarget, BillingStrategy};
use self::plan_error::PlanError;

pub struct PlanEntity<'a> {
    props: Plan,
    repo:  &'a MongoPlanRepo,
}

impl<'a> PlanEntity<'a> {
    pub fn new(new_plan: NewPlan, repo: &'a MongoPlanRepo) -> Self {
        Self {
            repo,
            props: Plan {
                id: None,
                name: new_plan.name,
                price: new_plan.price,
                limit_tenant: new_plan.limit_tenant,
                limit_users: new_plan.limit_users,
                state: new_plan.state,
                features: new_plan.features,
                target: new_plan.target,
                organization_id: new_plan.organization_id,
                billing_strategy: new_plan.billing_strategy,
            },
        }
    }

    pub async fn create(self) -> Result<Plan, PlanError> {
        self.repo.create(self.props).await
    }

    pub async fn load_by_id(plan_id: PlanID, repo: &'a MongoPlanRepo) -> Result<Self, PlanError> {
        let plan = repo.fetch_by_id(plan_id).await?;
        Ok(Self { props: plan, repo })
    }

    pub fn update_name(&mut self, name: String) -> Result<(), PlanError> {
        if name.is_empty() {
            return Err(PlanError::EmptyName);
        }
        self.props.name = name;
        Ok(())
    }

    pub fn update_price(&mut self, price: f64) -> Result<(), PlanError> {
        self.props.price = price;
        Ok(())
    }

    pub fn update_limits(&mut self, limit_tenant: i32, limit_users: i32) -> Result<(), PlanError> {
        self.props.limit_tenant = limit_tenant;
        self.props.limit_users = limit_users;
        Ok(())
    }

    pub fn update_state(&mut self, state: PlanState) -> Result<(), PlanError> {
        self.props.state = state;
        Ok(())
    }

    pub fn update_features(&mut self, features: PlanFeatures) -> Result<(), PlanError> {
        self.props.features = features;
        Ok(())
    }

    pub fn update_target(&mut self, target: PlanTarget) -> Result<(), PlanError> {
        self.props.target = target;
        Ok(())
    }

    pub fn update_organization_id(&mut self, organization_id: Option<OrganizationID>) -> Result<(), PlanError> {
        self.props.organization_id = organization_id;
        Ok(())
    }

    pub fn update_billing_strategy(&mut self, billing_strategy: Option<BillingStrategy>) -> Result<(), PlanError> {
        if let Some(ref strategy) = billing_strategy {
            strategy.validate().map_err(PlanError::InvalidBillingStrategy)?;
        }
        self.props.billing_strategy = billing_strategy;
        Ok(())
    }

    pub fn get_props(&self) -> &Plan {
        &self.props
    }
}
