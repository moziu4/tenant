pub mod tenant_type;
pub mod tenant_error;
pub mod menu;

use crate::data::access::tenant_repo::MongoTenantRepo;
use crate::utils::domains_ids::TenantID;
use self::tenant_type::{Tenant, NewTenant, TenantState, TenantFeatures, TenantConfiguration};
use self::menu::Menu;
use self::tenant_error::TenantError;

pub struct TenantEntity<'a> {
    props: Tenant,
    repo:  &'a MongoTenantRepo,
}

impl<'a> TenantEntity<'a> {
    pub fn new(new_tenant: NewTenant, repo: &'a MongoTenantRepo) -> Self {
        let menus = if new_tenant.menus.is_empty() {
            Self::default_menus(&new_tenant.features)
        } else {
            new_tenant.menus
        };

        Self {
            repo,
            props: Tenant {
                id: None,
                host: new_tenant.host,
                name: new_tenant.name,
                agency_id: new_tenant.agency_id,
                configuration: new_tenant.configuration,
                state: new_tenant.state,
                features: new_tenant.features,
                menus,
            },
        }
    }

    fn default_menus(features: &TenantFeatures) -> Vec<Menu> {
        use self::menu::{Menu, MenuItem, MenuItemType, FeatureType};

        let mut items = Vec::new();

        // Siempre añadimos Home
        items.push(MenuItem {
            id: None,
            title: "Home".to_string(),
            item_type: MenuItemType::Page("home".to_string()),
            order: 0,
            is_visible_front: true,
            is_visible_admin: true,
            permissions: vec![],
            children: vec![],
        });

        let mut order = 1;

        if features.shop {
            items.push(MenuItem {
                id: None,
                title: "Shop".to_string(),
                item_type: MenuItemType::Feature(FeatureType::Shop),
                order,
                is_visible_front: true,
                is_visible_admin: true,
                permissions: vec![],
                children: vec![],
            });
            order += 1;
        }

        if features.blog {
            items.push(MenuItem {
                id: None,
                title: "Blog".to_string(),
                item_type: MenuItemType::Feature(FeatureType::Blog),
                order,
                is_visible_front: true,
                is_visible_admin: true,
                permissions: vec![],
                children: vec![],
            });
            order += 1;
        }

        if features.academy {
            items.push(MenuItem {
                id: None,
                title: "Academy".to_string(),
                item_type: MenuItemType::Feature(FeatureType::Academy),
                order,
                is_visible_front: true,
                is_visible_admin: true,
                permissions: vec![],
                children: vec![],
            });
        }

        vec![Menu {
            name: "main".to_string(),
            items,
        }]
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

    pub fn get_props(&self) -> &Tenant {
        &self.props
    }

    pub fn get_public_menus(&self) -> Vec<Menu> {
        self.props.menus.iter().map(|menu| {
            let filtered_items = self.filter_items(&menu.items);
            Menu {
                name: menu.name.clone(),
                items: filtered_items,
            }
        }).collect()
    }

    fn filter_items(&self, items: &[self::menu::MenuItem]) -> Vec<self::menu::MenuItem> {
        let mut filtered = Vec::new();
        for item in items {
            if !item.is_visible_front {
                continue;
            }

            // Validar feature
            if let self::menu::MenuItemType::Feature(ref feature_type) = item.item_type {
                let has_feature = match feature_type {
                    self::menu::FeatureType::Shop => self.props.features.shop,
                    self::menu::FeatureType::Blog => self.props.features.blog,
                    self::menu::FeatureType::Academy => self.props.features.academy,
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

    pub fn update_menus(&mut self, menus: Vec<Menu>) {
        self.props.menus = menus;
    }
}
