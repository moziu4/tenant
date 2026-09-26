use std::sync::Arc;
use crate::context::Context;
use crate::core::commands::organization_command::{OrganizationCommand, OrganizationState};
use crate::core::domain::organization::organization_error::OrganizationError;
use crate::core::domain::organization::organization_type::{Organization, NewOrganization};
use crate::core::domain::organization::OrganizationEntity;
use crate::core::domain::tenant::tenant_type::TenantState;
use crate::core::commands::tenant_command::TenantCommand;
use crate::core::operation::tenant_ops::TenantOps;
use crate::data::access::organization_repo::MongoOrganizationRepo;
use crate::utils::domains_ids::OrganizationID;
use crate::handlers::messages::nats_handler::NatsHandler;
use crate::handlers::messages::{StateChangedEvent, EntityType};

pub struct OrganizationOps<'a> {
    repo: &'a MongoOrganizationRepo,
    context: &'a Context,
    nats_handler: Arc<NatsHandler>,
}

impl<'a> OrganizationOps<'a> {
    pub fn new(repo: &'a MongoOrganizationRepo, context: &'a Context) -> Self {
        Self { 
            repo,
            context,
            nats_handler: context.get_nats_handler()
        }
    }

    pub async fn create_organization(&self, new_organization: NewOrganization) -> Result<Organization, OrganizationError> {
        let entity = OrganizationEntity::new(new_organization, self.repo);
        let organization = entity.create().await?;

        // Invalidar cache de tenants
        let tenant_repo = self.context.get_tenant_repo();
        let tenant_ops = TenantOps::new(&tenant_repo, self.context);
        let _ = tenant_ops.clear_cache().await;

        Ok(organization)
    }

    pub async fn dispatch_commands(&self, organization_id: OrganizationID, commands: &[OrganizationCommand]) -> Result<Organization, OrganizationError> {
        let mut entity = OrganizationEntity::load_by_id(organization_id.clone(), self.repo).await?;

        for command in commands {
            self.dispatch_command(&mut entity, command).await?;
        }

        let organization = self.repo.save(entity.get_props().clone()).await?;

        // Invalidar cache de tenants ya que la organizacion puede haber cambiado de estado o plan
        let tenant_repo = self.context.get_tenant_repo();
        let tenant_ops = TenantOps::new(&tenant_repo, self.context);
        let _ = tenant_ops.clear_cache().await;

        Ok(organization)
    }

    async fn dispatch_command(&self, entity: &mut OrganizationEntity<'a>, command: &OrganizationCommand) -> Result<(), OrganizationError> {
        match command {
            OrganizationCommand::UpdateName { name } => entity.update_name(name.clone()),
            OrganizationCommand::UpdatePlan { plan } => entity.update_plan(plan.clone()),
            OrganizationCommand::UpdateState { state } => {
                let old_state = entity.get_props().state.clone();
                entity.update_state(state.clone())?;
                
                if old_state != *state {
                    // Notificar cambio de estado de la organizacion
                    let event = StateChangedEvent {
                        entity_type: EntityType::Organization,
                        entity_id: entity.get_props().id.as_ref().unwrap().to_string(),
                        state: match state {
                            OrganizationState::Active => TenantState::Active,
                            OrganizationState::Inactive => TenantState::Inactive,
                        },
                    };
                    let _ = self.nats_handler.publish_state_changed(event).await;

                    // Cascada a Tenants
                    self.cascade_state_to_tenants(entity.get_props().id.as_ref().unwrap().clone(), state.clone()).await?;
                }
                Ok(())
            },
         
        }
    }

    pub async fn delete_organization(&self, organization_id: OrganizationID) -> Result<(), OrganizationError> {
        let organization = self.repo.fetch_by_id(organization_id.clone()).await?;

        // Regla: No se puede borrar si no está desactivada
        if organization.state != OrganizationState::Inactive {
            return Err(OrganizationError::OrganizationMustBeInactiveToDelete);
        }

        self.repo.delete(organization_id).await
    }

    async fn cascade_state_to_tenants(&self, organization_id: OrganizationID, state: OrganizationState) -> Result<(), OrganizationError> {
        let tenant_repo = self.context.get_tenant_repo();
        let tenant_ops = TenantOps::new(&tenant_repo, self.context);
        
        let filter = mongodb::bson::doc! { "organization_id": mongodb::bson::oid::ObjectId::from(organization_id) };
        let mut cursor = self.context.get_collection("tenant").find(filter).await?;
        
        use futures_util::TryStreamExt;
        let new_tenant_state = match state {
            OrganizationState::Active => TenantState::Active,
            OrganizationState::Inactive => TenantState::Inactive,
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
