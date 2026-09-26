pub mod tenant_type;
pub mod tenant_error;
pub mod menu;

use crate::data::access::tenant_repo::MongoTenantRepo;
use crate::utils::domains_ids::{TenantID, MenuItemID, PlanID};
use self::tenant_type::{Tenant, NewTenant, TenantState, TenantFeatures, TenantConfiguration};
use self::menu::{Menu, MenuItem};
use self::tenant_error::TenantError;

pub struct TenantEntity<'a> {
    props: Tenant,
    repo:  &'a MongoTenantRepo,
}

impl<'a> TenantEntity<'a> {
    pub fn new(new_tenant: NewTenant, repo: &'a MongoTenantRepo) -> Self {
        let menus = if new_tenant.menus.is_empty() {
            Vec::new()
        } else {
            new_tenant.menus
        };

        Self {
            repo,
            props: Tenant {
                id: None,
                host: new_tenant.host,
                name: new_tenant.name,
                organization_id: new_tenant.organization_id,
                plan_id: new_tenant.plan_id,
                configuration: new_tenant.configuration,
                state: new_tenant.state,
                features: new_tenant.features,
                default_language: new_tenant.default_language,
                available_languages: new_tenant.available_languages,
                menus,
            },
        }
    }

    pub fn from_props(props: Tenant, repo: &'a MongoTenantRepo) -> Self {
        Self { props, repo }
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

    pub fn update_plan_id(&mut self, plan_id: Option<PlanID>) -> Result<(), TenantError> {
        self.props.plan_id = plan_id;
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

    pub fn update_default_language(&mut self, default_language: String) -> Result<(), TenantError> {
        self.props.default_language = default_language;
        Ok(())
    }

    pub fn update_available_languages(&mut self, available_languages: Vec<String>) -> Result<(), TenantError> {
        self.props.available_languages = available_languages;
        Ok(())
    }

    pub fn get_props(&self) -> &Tenant {
        &self.props
    }

    pub fn get_public_menus(&self, _language: Option<String>) -> Vec<Menu> {
        self.props.menus.iter()
            .filter(|menu| menu.is_active)
            .map(|menu| {
                let filtered_items = self.filter_items(&menu.items);
                Menu {
                    name: menu.name.clone(),
                    items: filtered_items,
                    is_active: menu.is_active,
                }
            }).collect()
    }

    pub fn get_filtered_tenant(&self, language: Option<String>) -> Tenant {
        let mut tenant = self.props.clone();
        tenant.menus = self.get_public_menus(language);
        tenant
    }

    fn filter_items(&self, items: &[self::menu::MenuItem]) -> Vec<self::menu::MenuItem> {
        let mut filtered = Vec::new();
        for item in items {
            if !item.is_visible_front {
                continue;
            }

            // Validar feature
            if let self::menu::MenuItemType::Feature { feature, .. } = &item.item_type {
                let has_feature = match feature {
                    self::menu::FeatureType::Migration => self.props.features.migration,
                };
                if !has_feature {
                    continue;
                }
            }

            // Recursividad para hijos
            let mut item_clone = item.clone();
            item_clone.children = self.filter_items(&item.children);

            filtered.push(item_clone);
        }

        // Ordenar por el campo 'order'
        filtered.sort_by_key(|i| i.order);
        filtered
    }

    pub async fn save(self) -> Result<Tenant, TenantError> {
        self.repo.save(self.props).await
    }

    pub fn update_menus(&mut self, menus: Vec<Menu>) -> Result<(), TenantError> {
        // Validar que solo haya un main activo
        let active_main_count = menus.iter()
            .filter(|m| m.name == "main" && m.is_active)
            .count();

        if active_main_count > 1 {
            return Err(TenantError::OnlyOneActiveMainMenu);
        }

        self.props.menus = menus;
        Ok(())
    }

    pub fn add_menu(&mut self, menu: Menu) -> Result<(), TenantError> {
        if menu.name == "main" && menu.is_active {
            // Desactivar otros main si este se añade como activo
            for m in self.props.menus.iter_mut() {
                if m.name == "main" {
                    m.is_active = false;
                }
            }
        }
        self.props.menus.push(menu);
        Ok(())
    }

    pub fn update_specific_menu(&mut self, name: String, updated_menu: Menu) -> Result<(), TenantError> {
        if updated_menu.name == "main" && updated_menu.is_active {
            // Desactivar otros main
            for m in self.props.menus.iter_mut() {
                if m.name == "main" && m.name != name {
                    m.is_active = false;
                }
            }
        }

        if let Some(pos) = self.props.menus.iter().position(|m| m.name == name) {
            self.props.menus[pos] = updated_menu;
            Ok(())
        } else {
            Err(TenantError::TenantDocNotUpdated) // O crear un error MenuNotFound
        }
    }

    pub fn delete_menu(&mut self, name: String) -> Result<(), TenantError> {
        self.props.menus.retain(|m| m.name != name);
        Ok(())
    }

    pub fn update_menu_item_group_id(&mut self, menu_name: String, item_id: MenuItemID, new_group_id: String) -> Result<(), TenantError> {
        let menu = self.props.menus.iter_mut()
            .find(|m| m.name == menu_name)
            .ok_or(TenantError::MenuNotFound)?;

        fn find_and_update(items: &mut [MenuItem], target_id: &MenuItemID, group_id: &str) -> bool {
            for item in items.iter_mut() {
                if let Some(ref id) = item.id {
                    if id == target_id {
                        item.group_id = group_id.to_string();
                        return true;
                    }
                }
                if find_and_update(&mut item.children, target_id, group_id) {
                    return true;
                }
            }
            false
        }

        if find_and_update(&mut menu.items, &item_id, &new_group_id) {
            Ok(())
        } else {
            Err(TenantError::MenuItemNotFound)
        }
    }
}
