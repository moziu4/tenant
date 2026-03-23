use std::sync::Arc;
use crate::context::Context;
use crate::core::commands::agency_command::{AgencyCommand, AgencyState};
use crate::core::domain::agency::agency_error::AgencyError;
use crate::core::domain::agency::agency_type::{Agency, NewAgency};
use crate::core::domain::agency::AgencyEntity;
use crate::core::domain::tenant::tenant_type::TenantState;
use crate::core::commands::tenant_command::TenantCommand;
use crate::core::operation::tenant_ops::TenantOps;
use crate::data::access::agency_repo::MongoAgencyRepo;
use crate::utils::domains_ids::AgencyID;
use crate::handlers::messages::nats_handler::NatsHandler;
use crate::handlers::messages::{StateChangedEvent, EntityType};

pub struct AgencyOps<'a> {
    repo: &'a MongoAgencyRepo,
    context: &'a Context,
    nats_handler: Arc<NatsHandler>,
}

impl<'a> AgencyOps<'a> {
    pub fn new(repo: &'a MongoAgencyRepo, context: &'a Context) -> Self {
        Self { 
            repo,
            context,
            nats_handler: context.get_nats_handler()
        }
    }

    pub async fn create_agency(&self, new_agency: NewAgency) -> Result<Agency, AgencyError> {
        let entity = AgencyEntity::new(new_agency, self.repo);
        let agency = entity.create().await?;
        Ok(agency)
    }

    pub async fn dispatch_commands(&self, agency_id: AgencyID, commands: &[AgencyCommand]) -> Result<Agency, AgencyError> {
        let mut entity = AgencyEntity::load_by_id(agency_id.clone(), self.repo).await?;

        for command in commands {
            self.dispatch_command(&mut entity, command).await?;
        }

        let agency = self.repo.save(entity.get_props().clone()).await?;
        Ok(agency)
    }

    async fn dispatch_command(&self, entity: &mut AgencyEntity<'a>, command: &AgencyCommand) -> Result<(), AgencyError> {
        match command {
            AgencyCommand::UpdateName { name } => entity.update_name(name.clone()),
            AgencyCommand::UpdatePlan { plan } => entity.update_plan(plan.clone()),
            AgencyCommand::UpdateState { state } => {
                let old_state = entity.get_props().state.clone();
                entity.update_state(state.clone())?;
                
                if old_state != *state {
                    // Notificar cambio de estado de la agencia
                    let event = StateChangedEvent {
                        entity_type: EntityType::Agency,
                        entity_id: entity.get_props().id.as_ref().unwrap().to_string(),
                        state: match state {
                            AgencyState::Active => TenantState::Active,
                            AgencyState::Inactive => TenantState::Inactive,
                        },
                    };
                    let _ = self.nats_handler.publish_state_changed(event).await;

                    // Cascada a Tenants
                    self.cascade_state_to_tenants(entity.get_props().id.as_ref().unwrap().clone(), state.clone()).await?;
                }
                Ok(())
            },
            AgencyCommand::AddUser { user_id } => entity.add_user(user_id.clone()),
            AgencyCommand::RemoveUser { user_id } => entity.remove_user(user_id.clone()),
        }
    }

    pub async fn delete_agency(&self, agency_id: AgencyID) -> Result<(), AgencyError> {
        let agency = self.repo.fetch_by_id(agency_id.clone()).await?;

        // Regla: No se puede borrar si no está desactivada
        if agency.state != AgencyState::Inactive {
            return Err(AgencyError::AgencyMustBeInactiveToDelete);
        }

        self.repo.delete(agency_id).await
    }

    async fn cascade_state_to_tenants(&self, agency_id: AgencyID, state: AgencyState) -> Result<(), AgencyError> {
        let tenant_repo = self.context.get_tenant_repo();
        let tenant_ops = TenantOps::new(&tenant_repo, self.context);
        
        // Encontrar todos los tenants de esta agencia
        // Nota: Tendríamos que añadir un método al repositorio para buscar por agency_id o usar fetch_all con filtro.
        // Por ahora, usamos el repo directamente para filtrar.
        let filter = mongodb::bson::doc! { "agency_id": mongodb::bson::oid::ObjectId::from(agency_id) };
        let mut cursor = self.context.get_collection("tenant").find(filter).await?;
        
        use futures_util::TryStreamExt;
        let new_tenant_state = match state {
            AgencyState::Active => TenantState::Active,
            AgencyState::Inactive => TenantState::Inactive,
        };

        while let Some(tenant_doc) = cursor.try_next().await? {
            let tenant: crate::core::domain::tenant::tenant_type::Tenant = mongodb::bson::from_document(tenant_doc)?;
            if tenant.state != new_tenant_state {
                if let Some(tid) = tenant.id {
                    let commands = vec![TenantCommand::UpdateState { state: new_tenant_state.clone() }];
                    let _ = tenant_ops.dispatch_commands(tid, &commands).await;
                }
            }
        }

        Ok(())
    }
}
