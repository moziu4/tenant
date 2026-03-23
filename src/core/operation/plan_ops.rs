use crate::core::commands::plan_command::PlanCommand;
use crate::core::domain::plans::plan_error::PlanError;
use crate::core::domain::plans::plan_type::{Plan, NewPlan, PlanState};
use crate::core::domain::plans::PlanEntity;
use crate::data::access::plan_repo::MongoPlanRepo;
use crate::utils::domains_ids::PlanID;
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
        let entity = PlanEntity::new(new_plan, self.repo);
        let plan = entity.create().await?;
        Ok(plan)
    }

    pub async fn dispatch_commands(&self, plan_id: PlanID, commands: &[PlanCommand]) -> Result<Plan, PlanError> {
        let mut entity = PlanEntity::load_by_id(plan_id.clone(), self.repo).await?;

        for command in commands {
            self.dispatch_command(&mut entity, command)?;
        }

        let plan = self.repo.save(entity.get_props().clone()).await?;
        Ok(plan)
    }

    fn dispatch_command(&self, entity: &mut PlanEntity<'a>, command: &PlanCommand) -> Result<(), PlanError> {
        match command {
            PlanCommand::UpdateName { name } => entity.update_name(name.clone()),
            PlanCommand::UpdatePrice { price } => entity.update_price(*price),
            PlanCommand::UpdateLimits { limit_tenant, limit_users } => entity.update_limits(*limit_tenant, *limit_users),
            PlanCommand::UpdateState { state } => entity.update_state(state.clone()),
            PlanCommand::UpdateFeatures { features } => entity.update_features(features.clone()),
        }
    }

    pub async fn delete_plan(&self, plan_id: PlanID) -> Result<(), PlanError> {
        let plan = self.repo.fetch_by_id(plan_id.clone()).await?;
        
        // Regla: No se puede borrar si no está desactivado
        if plan.state != PlanState::Inactive {
            return Err(PlanError::PlanMustBeInactiveToDelete);
        }

        // Regla: No se puede borrar si está en uso por alguna agencia
        let agency_repo = self.context.get_agency_repo();
    
        let filter = mongodb::bson::doc! { "plan": mongodb::bson::oid::ObjectId::from(plan_id.clone()) };
        let count = self.context.get_collection("agency").count_documents(filter).await.map_err(|_| PlanError::PlanNotFound)?;
        
        if count > 0 {
            return Err(PlanError::PlanInUse);
        }

        self.repo.delete(plan_id).await
    }
}
