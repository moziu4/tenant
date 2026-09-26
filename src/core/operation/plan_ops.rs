use crate::core::operation::tenant_ops::TenantOps;
use crate::core::commands::plan_command::PlanCommand;
use crate::core::domain::plans::plan_error::PlanError;
use crate::core::domain::plans::plan_type::{Plan, NewPlan, PlanState, PlanTarget};
use crate::core::domain::plans::PlanEntity;
use crate::data::access::plan_repo::MongoPlanRepo;
use crate::utils::domains_ids::{PlanID, OrganizationID};
use crate::context::Context;

pub struct PlanOps<'a> {
    repo: &'a MongoPlanRepo,
    context: &'a Context,
}

impl<'a> PlanOps<'a> {
    pub fn new(repo: &'a MongoPlanRepo, context: &'a Context) -> Self {
        Self { repo, context }
    }

    pub async fn create_plan(&self, new_plan: NewPlan) -> Result<Plan, PlanError> {
        // Validaciones previas según el target
        if new_plan.target == PlanTarget::Tenant {
            if let Some(ref org_id) = new_plan.organization_id {
                let organization_repo = self.context.get_organization_repo();
                organization_repo.fetch_by_id(org_id.clone())
                    .await
                    .map_err(|_| PlanError::OrganizationNotFound)?;
            }
        }

        if let Some(ref strategy) = new_plan.billing_strategy {
            strategy.validate().map_err(PlanError::InvalidBillingStrategy)?;
        }

        let entity = PlanEntity::new(new_plan, self.repo);
        let plan = entity.create().await?;

        // Invalidar cache de tenants
        let tenant_repo = self.context.get_tenant_repo();
        let tenant_ops = TenantOps::new(&tenant_repo, self.context);
        let _ = tenant_ops.clear_cache().await;

        Ok(plan)
    }

    pub async fn load_all_plans(&self, target: Option<PlanTarget>, organization_id: Option<OrganizationID>) -> Result<Vec<Plan>, PlanError> {
        self.repo.fetch_all(target, organization_id).await
    }

    pub async fn dispatch_commands(&self, plan_id: PlanID, commands: &[PlanCommand]) -> Result<Plan, PlanError> {
        let mut entity = PlanEntity::load_by_id(plan_id.clone(), self.repo).await?;

        for command in commands {
            self.dispatch_command(&mut entity, command).await?;
        }

        let plan = self.repo.save(entity.get_props().clone()).await?;

        // Invalidar cache de tenants ya que un cambio en el plan puede afectar a todos los tenants
        let tenant_repo = self.context.get_tenant_repo();
        let tenant_ops = TenantOps::new(&tenant_repo, self.context);
        let _ = tenant_ops.clear_cache().await;

        Ok(plan)
    }

    async fn dispatch_command(&self, entity: &mut PlanEntity<'a>, command: &PlanCommand) -> Result<(), PlanError> {
        match command {
            PlanCommand::UpdateName { name } => entity.update_name(name.clone()),
            PlanCommand::UpdatePrice { price } => entity.update_price(*price),
            PlanCommand::UpdateLimits { limit_tenant, limit_users } => entity.update_limits(*limit_tenant, *limit_users),
            PlanCommand::UpdateState { state } => entity.update_state(state.clone()),
            PlanCommand::UpdateFeatures { features } => entity.update_features(features.clone()),
            PlanCommand::UpdateTarget { target } => entity.update_target(target.clone()),
            PlanCommand::UpdateOrganizationId { organization_id } => {
                if let Some(ref org_id) = organization_id {
                    let organization_repo = self.context.get_organization_repo();
                    organization_repo.fetch_by_id(org_id.clone())
                        .await
                        .map_err(|_| PlanError::OrganizationNotFound)?;
                }
                entity.update_organization_id(organization_id.clone())
            },
            PlanCommand::UpdateBillingStrategy { billing_strategy } => entity.update_billing_strategy(billing_strategy.clone()),
        }
    }

    pub async fn delete_plan(&self, plan_id: PlanID) -> Result<(), PlanError> {
        let plan = self.repo.fetch_by_id(plan_id.clone()).await?;
        
        // Regla: No se puede borrar si no está desactivado
        if plan.state != PlanState::Inactive {
            return Err(PlanError::PlanMustBeInactiveToDelete);
        }

        // Regla: No se puede borrar si está en uso por alguna organizacion o tenant
        match plan.target {
            PlanTarget::Organization => {
                let filter = mongodb::bson::doc! { "plan": mongodb::bson::oid::ObjectId::from(plan_id.clone()) };
                let count = self.context.get_collection("organization").count_documents(filter).await.map_err(|_| PlanError::PlanNotFound)?;
                if count > 0 {
                    return Err(PlanError::PlanInUse);
                }
            },
            PlanTarget::Tenant => {
                let filter = mongodb::bson::doc! {
                    "$or": [
                        { "plan_id": mongodb::bson::oid::ObjectId::from(plan_id.clone()) },
                        { "plan": mongodb::bson::oid::ObjectId::from(plan_id.clone()) }
                    ]
                };
                let count = self.context.get_collection("tenant").count_documents(filter).await.map_err(|_| PlanError::PlanNotFound)?;
                if count > 0 {
                    return Err(PlanError::PlanInUse);
                }
            }
        }

        self.repo.delete(plan_id).await
    }
}
