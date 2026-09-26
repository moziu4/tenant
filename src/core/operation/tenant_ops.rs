use std::sync::Arc;
use crate::context::Context;
use crate::core::commands::tenant_command::TenantCommand;
use crate::core::domain::tenant::tenant_error::TenantError;
use crate::core::domain::tenant::tenant_type::{Tenant, NewTenant, TenantState};
use crate::core::domain::tenant::TenantEntity;
use crate::data::access::tenant_repo::MongoTenantRepo;
use crate::utils::cache::RedisCache;
use crate::utils::cache_error::CacheError;
use crate::utils::domains_ids::{TenantID, OrganizationID};
use crate::handlers::messages::nats_handler::NatsHandler;
use crate::handlers::messages::{StateChangedEvent, EntityType, TenantCreatedEvent};

pub struct TenantOps<'a> {
    repo: &'a MongoTenantRepo,
    context: &'a Context,
    redis_cache: Arc<RedisCache>,
    nats_handler: Arc<NatsHandler>,
}

impl<'a> TenantOps<'a> {
    pub fn new(repo: &'a MongoTenantRepo, context: &'a Context) -> Self {
        Self { 
            repo,
            context,
            redis_cache: context.get_redis_cache(),
            nats_handler: context.get_nats_handler()
        }
    }

    pub async fn create_tenant(&self, new_tenant: NewTenant) -> Result<Tenant, TenantError> {
        // 1. Validar que la organización existe y está activa
        let organization_repo = self.context.get_organization_repo();
        let org = organization_repo.fetch_by_id(new_tenant.organization_id.clone())
            .await
            .map_err(|_| TenantError::OrganizationNotFound)?;

        if org.state == crate::core::commands::organization_command::OrganizationState::Inactive {
            return Err(TenantError::OrganizationInactive);
        }

        // 2. Validar que el plan de la organización existe y que las features solicitadas están permitidas por el plan
        let plan_repo = self.context.get_plan_repo();
        let org_plan = plan_repo.fetch_by_id(org.plan.clone())
            .await
            .map_err(|_| TenantError::PlanNotFound)?;

        new_tenant.features.validate_against_plan(&org_plan.features)?;

        // 3. Validar límite de tenants si aplica (limit_tenant > 0)
        if org_plan.limit_tenant > 0 {
            let existing_tenants = self.repo.fetch_all(None, Some(new_tenant.organization_id.clone())).await?;
            if existing_tenants.len() >= org_plan.limit_tenant as usize {
                return Err(TenantError::TenantLimitReached);
            }
        }

        // 4. Si el tenant tiene un plan propio de tenant (plan_id), validarlo
        if let Some(ref tenant_plan_id) = new_tenant.plan_id {
            let tenant_plan = plan_repo.fetch_by_id(tenant_plan_id.clone())
                .await
                .map_err(|_| TenantError::TenantPlanNotFound)?;

            if tenant_plan.target != crate::core::domain::plans::plan_type::PlanTarget::Tenant {
                return Err(TenantError::InvalidTenantPlan("Selected plan is not a tenant plan".to_string()));
            }

            if tenant_plan.state == crate::core::domain::plans::plan_type::PlanState::Inactive {
                return Err(TenantError::TenantPlanInactive);
            }

            if let Some(ref allowed_org_id) = tenant_plan.organization_id {
                if *allowed_org_id != new_tenant.organization_id {
                    return Err(TenantError::TenantPlanNotAllowedForOrganization);
                }
            }

            // Validar que las features del tenant están permitidas por el plan de tenant
            new_tenant.features.validate_against_plan(&tenant_plan.features)
                .map_err(|_| TenantError::FeatureNotAllowedByTenantPlan("Feature not allowed by tenant plan".to_string()))?;
        }

        let entity = TenantEntity::new(new_tenant, self.repo);
        let tenant = entity.create().await?;
        
        // Notificar creación de tenant vía NATS
        if let Some(id) = &tenant.id {
            let event = TenantCreatedEvent {
                organization_id: tenant.organization_id.to_string(),
                tenant_id: id.to_string(),
                name: tenant.name.clone(),
                available_languages: tenant.available_languages.clone(),
                features: tenant.features.clone(),
                plan_id: tenant.plan_id.as_ref().map(|p| p.to_string()),
            };
            println!("Event: {:?}", event);
            let _ = self.nats_handler.publish_tenant_created(event).await;
        }

        self.clear_cache().await?;
        Ok(tenant)
    }

    pub async fn load_all_tenants(&self, host: Option<String>, organization_id: Option<OrganizationID>) -> Result<Vec<Tenant>, TenantError> {
        let cache_key = match (&host, &organization_id) {
            (Some(h), Some(oid)) => format!("all_tenants_{}_{}", h, oid),
            (Some(h), None) => format!("all_tenants_{}", h),
            (None, Some(oid)) => format!("all_tenants_{}", oid),
            (None, None) => "all_tenants".to_string(),
        };
        if let Some(tenants) = self.get_tenant_cache(&cache_key).await {
            return tenants;
        }

        let tenants = self.repo.fetch_all(host, organization_id).await?;
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
                if *state == TenantState::Active && old_state != TenantState::Active {
                    // Validar que la organización existe y está activa
                    let organization_repo = self.context.get_organization_repo();
                    let org = organization_repo.fetch_by_id(entity.get_props().organization_id.clone())
                        .await
                        .map_err(|_| TenantError::OrganizationNotFound)?;

                    if org.state == crate::core::commands::organization_command::OrganizationState::Inactive {
                        return Err(TenantError::OrganizationInactive);
                    }
                }

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
            TenantCommand::UpdateFeatures { features } => {
                // Validar que las features están permitidas por el plan de la organización
                let organization_repo = self.context.get_organization_repo();
                let org = organization_repo.fetch_by_id(entity.get_props().organization_id.clone())
                    .await
                    .map_err(|_| TenantError::OrganizationNotFound)?;

                let plan_repo = self.context.get_plan_repo();
                let org_plan = plan_repo.fetch_by_id(org.plan.clone())
                    .await
                    .map_err(|_| TenantError::PlanNotFound)?;

                features.validate_against_plan(&org_plan.features)?;

                // Si el tenant tiene un plan propio asignado, validar también contra él
                if let Some(ref tenant_plan_id) = entity.get_props().plan_id {
                    let tenant_plan = plan_repo.fetch_by_id(tenant_plan_id.clone())
                        .await
                        .map_err(|_| TenantError::TenantPlanNotFound)?;

                    features.validate_against_plan(&tenant_plan.features)
                        .map_err(|_| TenantError::FeatureNotAllowedByTenantPlan("Feature not allowed by tenant plan".to_string()))?;
                }

                entity.update_features(features.clone())
            },
            TenantCommand::UpdatePlan { plan_id } => {
                if let Some(ref pid) = plan_id {
                    let plan_repo = self.context.get_plan_repo();
                    let tenant_plan = plan_repo.fetch_by_id(pid.clone())
                        .await
                        .map_err(|_| TenantError::TenantPlanNotFound)?;

                    if tenant_plan.target != crate::core::domain::plans::plan_type::PlanTarget::Tenant {
                        return Err(TenantError::InvalidTenantPlan("Selected plan is not a tenant plan".to_string()));
                    }

                    if tenant_plan.state == crate::core::domain::plans::plan_type::PlanState::Inactive {
                        return Err(TenantError::TenantPlanInactive);
                    }

                    if let Some(ref allowed_org_id) = tenant_plan.organization_id {
                        if *allowed_org_id != entity.get_props().organization_id {
                            return Err(TenantError::TenantPlanNotAllowedForOrganization);
                        }
                    }

                    // Validar que las features actuales del tenant son compatibles con el nuevo plan
                    entity.get_props().features.validate_against_plan(&tenant_plan.features)
                        .map_err(|_| TenantError::FeatureNotAllowedByTenantPlan("Current tenant features exceed the new tenant plan".to_string()))?;
                }

                entity.update_plan_id(plan_id.clone())
            },
            TenantCommand::UpdateDefaultLanguage { default_language } => entity.update_default_language(default_language.clone()),
            TenantCommand::UpdateAvailableLanguages { available_languages } => entity.update_available_languages(available_languages.clone()),
            TenantCommand::UpdateMenus { menus } => entity.update_menus(menus.clone()),
            TenantCommand::UpdateMenuItemGroupId { menu_name, item_id, group_id } => entity.update_menu_item_group_id(menu_name.clone(), item_id.clone(), group_id.clone()),
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

    pub async fn clear_cache(&self) -> Result<(), TenantError> {
        let mut con = self.redis_cache.get_client().get_multiplexed_async_connection().await
            .map_err(|e| TenantError::RedisError(e.to_string()))?;
        
        // Obtenemos todas las claves que empiezan por all_tenants
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("all_tenants*")
            .query_async(&mut con)
            .await
            .map_err(|e| TenantError::RedisError(e.to_string()))?;

        for key in keys {
            let _: () = redis::cmd("DEL")
                .arg(key)
                .query_async(&mut con)
                .await
                .map_err(|e| TenantError::RedisError(e.to_string()))?;
        }

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
