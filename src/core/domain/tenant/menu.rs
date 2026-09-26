use serde::{Deserialize, Serialize};
use crate::utils::domains_ids::MenuItemID;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureType {
    Migration
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MenuLabel {
    pub lang: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum MenuItemType {
    Page,
    ExternalUrl { url: String, labels: Vec<MenuLabel> },
    Feature { feature: FeatureType },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MenuItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<MenuItemID>,
    pub item_type: MenuItemType,
    pub order: i32,
    pub group_id: String,
    pub is_visible_front: bool, // Visible en el front
    pub is_visible_admin: bool, // Siempre visible en el admin (implícito, pero se puede usar para deshabilitar)
    #[serde(default)]
    pub permissions: Vec<String>, // Permisos requeridos
    #[serde(default)]
    pub children: Vec<MenuItem>, // Para menús anidados
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Menu {
    pub name: String, // e.g., "main", "footer"
    pub items: Vec<MenuItem>,
    #[serde(default)]
    pub is_active: bool
}
