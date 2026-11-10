use serde::{Deserialize, Serialize};
use crate::core::domain::tenant::tenant_type::{TenantState, RegistrationType};
use crate::core::domain::tenant::menu::Menu;

use crate::utils::domains_ids::{MenuItemID, PlanID};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum TenantCommand {
    UpdateState { state: TenantState },
    UpdateRegistrationType { registration_type: RegistrationType },
    UpdateDefaultLanguage { default_language: String },
    UpdateAvailableLanguages { available_languages: Vec<String> },
    UpdateMenus { menus: Vec<Menu> },
    UpdateMenuItemGroupId { menu_name: String, item_id: MenuItemID, group_id: String },
    UpdatePlan { #[serde(default, alias = "plan")] plan_id: Option<PlanID> },
}
