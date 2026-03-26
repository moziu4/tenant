use std::sync::Arc;
use crate::context::Context;
use crate::core::commands::tenant_command::TenantCommand;
use crate::core::domain::tenant::tenant_error::TenantError;
use crate::core::domain::tenant::tenant_type::{Tenant, NewTenant, TenantState};
use crate::core::domain::tenant::TenantEntity;
use crate::data::access::tenant_repo::MongoTenantRepo;
use crate::utils::cache::RedisCache;
use crate::utils::cache_error::CacheError;
use crate::utils::domains_ids::{TenantID, AgencyID};
use crate::handlers::messages::nats_handler::NatsHandler;
use crate::handlers::messages::{StateChangedEvent, EntityType, TenantCreatedEvent};

pub struct TenantOps<'a> {
    repo: &'a MongoTenantRepo,
    redis_cache: Arc<RedisCache>,
    nats_handler: Arc<NatsHandler>,
}

impl<'a> TenantOps<'a> {
    pub fn new(repo: &'a MongoTenantRepo, context: &Context) -> Self {
        Self { 
            repo,
            redis_cache: context.get_redis_cache(),
            nats_handler: context.get_nats_handler()
        }
    }

    pub async fn create_tenant(&self, new_tenant: NewTenant) -> Result<Tenant, TenantError> {
        let entity = TenantEntity::new(new_tenant, self.repo);
        let tenant = entity.create().await?;
        
        // Notificar creación de tenant vía NATS
        if let Some(id) = &tenant.id {
            let event = TenantCreatedEvent {
                tenant_id: id.to_string(),
                name: tenant.name.clone(),
            };
            let _ = self.nats_handler.publish_tenant_created(event).await;
        }

        self.clear_cache().await?;
        Ok(tenant)
    }

    pub async fn load_all_tenants(&self, host: Option<String>, agency_id: Option<AgencyID>) -> Result<Vec<Tenant>, TenantError> {
        let cache_key = match (&host, &agency_id) {
            (Some(h), Some(aid)) => format!("all_tenants_{}_{}", h, aid),
            (Some(h), None) => format!("all_tenants_{}", h),
            (None, Some(aid)) => format!("all_tenants_{}", aid),
            (None, None) => "all_tenants".to_string(),
        };
        if let Some(tenants) = self.get_tenant_cache(&cache_key).await {
            return tenants;
        }

        let tenants = self.repo.fetch_all(host, agency_id).await?;
        self.set_tenant_cache(cache_key, &tenants).await?;
        Ok(tenants)
    }

    pub async fn load_tenant_by_id(&self, id: TenantID) -> Result<Tenant, TenantError> {
        self.repo.fetch_by_id(id).await
    }

    pub async fn load_tenant_by_host(&self, host: String) -> Result<Tenant, TenantError> {
        self.repo.fetch_by_host(host).await
    }

    pub async fn dispatch_commands(&self, tenant_id: TenantID, commands: &[TenantCommand]) -> Result<Tenant, TenantError> {
        let mut entity = TenantEntity::load_by_id(tenant_id, self.repo).await?;

        for command in commands {
            self.dispatch_command(&mut entity, command).await?;
        }

        let tenant = self.repo.save(entity.get_props().clone()).await?;
        self.clear_cache().await?;
        Ok(tenant)
    }

    async fn dispatch_command(&self, entity: &mut TenantEntity<'a>, command: &TenantCommand) -> Result<(), TenantError> {
        match command {
            TenantCommand::UpdateName { name } => entity.update_name(name.clone()),
            TenantCommand::UpdateState { state } => {
                let old_state = entity.get_props().state.clone();
                entity.update_state(state.clone())?;

                if old_state != *state {
                    let event = StateChangedEvent {
                        entity_type: EntityType::Tenant,
                        entity_id: entity.get_props().id.as_ref().unwrap().to_string(),
                        state: state.clone(),
                    };
                    let _ = self.nats_handler.publish_state_changed(event).await;
                }
                Ok(())
            },
            TenantCommand::UpdateConfiguration { configuration } => entity.update_configuration(configuration.clone()),
            TenantCommand::UpdateFeatures { features } => entity.update_features(features.clone()),
            TenantCommand::UpdateMenus { menus } => {
                entity.update_menus(menus.clone());
                Ok(())
            },
        }
    }

    pub async fn delete_tenant(&self, id: TenantID) -> Result<(), TenantError> {
        let tenant = self.repo.fetch_by_id(id.clone()).await?;

        // Regla: No se puede borrar si no está desactivado
        if tenant.state != TenantState::Inactive {
            return Err(TenantError::TenantMustBeInactiveToDelete);
        }

        self.repo.delete(id).await?;
        self.clear_cache().await?;
        Ok(())
    }

    async fn clear_cache(&self) -> Result<(), TenantError> {
        self.redis_cache
            .invalidate_cache("all_tenants")
            .await
            .map_err(|e| TenantError::RedisError(e.to_string()))?;
        Ok(())
    }

    async fn get_tenant_cache(&self, cache_key: &str) -> Option<Result<Vec<Tenant>, TenantError>> {
        match self.redis_cache.get_tenants(cache_key).await {
            Ok(Some(tenants)) => Some(Ok(tenants)),
            Err(CacheError::Redis(err)) => Some(Err(TenantError::RedisError(err.to_string()))),
            Err(_) => None,
            Ok(None) => None,
        }
    }

    async fn set_tenant_cache(&self, cache_key: String, tenants: &Vec<Tenant>) -> Result<(), TenantError> {
        self.redis_cache
            .set_tenants(&cache_key, tenants.clone(), 3600)
            .await
            .map_err(|err| match err {
                CacheError::Redis(redis_err) => TenantError::RedisError(redis_err.to_string()),
                _ => TenantError::TenantDocumentNotCreated,
            })?;
        Ok(())
    }
}
