#![allow(dead_code, unused_imports)]

use std::sync::Arc;
use crate::context::Context;
use crate::handlers::messages::GroupIDCreatedEvent;
use crate::core::domain::tenant::menu::{Menu, MenuItem, MenuItemType, FeatureType};
use crate::utils::domains_ids::TenantID;
use futures_util::StreamExt;
use tracing::{error, info};
use crate::core::domain::tenant::TenantEntity;

pub async fn start_nats_subscriber(context: Arc<Context>) {
    let client = context.get_nats_handler().get_client();
    // Suscribirse a todos los eventos de creación de grupos de contenido
    let subject = "content.group.created.*";
    
    info!("Iniciando suscriptor NATS para subject: {}", subject);
    
    match client.subscribe(subject.to_string()).await {
        Ok(mut subscription) => {
            info!("Suscrito correctamente a {}", subject);
            while let Some(message) = subscription.next().await {
                let payload = message.payload;
                match serde_json::from_slice::<GroupIDCreatedEvent>(&payload) {
                    Ok(event) => {
                        info!("Recibido evento GroupIDCreated: tenant_id={}, slug={}, group_id={}", 
                            event.tenant_id, event.slug, event.group_id);
                        
                        let context_clone = context.clone();
                        actix_web::rt::spawn(async move {
                            // if let Err(e) = handle_group_id_created(context_clone, event).await {
                            //     error!("Error manejando GroupIDCreated: {}", e);
                            // }
                        });
                    }
                    Err(e) => {
                        error!("Error deserializando GroupIDCreatedEvent: {} de payload {:?}", e, payload);
                    }
                }
            }
        }
        Err(e) => {
            error!("Error al suscribirse a {}: {}", subject, e);
        }
    }
}

// async fn handle_group_id_created(context: Arc<Context>, event: GroupIDCreatedEvent) -> Result<(), String> {
//     let tenant_id = TenantID::parse_str(&event.tenant_id)
//         .map_err(|e| format!("Invalid tenant_id {}: {}", event.tenant_id, e))?;
//
//     let repo = context.get_tenant_repo();
//     let mut tenant_entity = TenantEntity::load_by_id(tenant_id, &repo).await
//         .map_err(|e| format!("Failed to load tenant {}: {:?}", event.tenant_id, e))?;
//
//     let tenant_props = tenant_entity.get_props().clone();
//     let mut menus = tenant_props.menus.clone();
//
//     // Si no hay menús, creamos uno básico "main"
//     if menus.is_empty() {
//         menus.push(Menu {
//             name: "main".to_string(),
//             items: Vec::new(),
//             is_active: true,
//         });
//     }
//
//     // Identificar tipo de item basado en el slug
//     for menu in menus.iter_mut() {
//         let item_type = get_item_type_from_slug(&event.slug);
//
//         // Evitar duplicados por group_id
//         let exists = menu.items.iter().any(|item| item.group_id == event.group_id);
//
//         if !exists {
//             let order = menu.items.len() as i32;
//             menu.items.push(MenuItem {
//                 id: None,
//                 item_type,
//                 order,
//                 group_id: event.group_id.clone(),
//                 is_visible_front: true,
//                 is_visible_admin: true,
//                 permissions: vec![],
//                 children: vec![],
//             });
//             info!("Añadido item {} al menú {} del tenant {}",
//                 event.slug, menu.name, event.tenant_id);
//         }
//     }
//
//     tenant_entity.update_menus(menus).map_err(|e| format!("Failed to update menus: {:?}", e))?;
//     tenant_entity.save().await.map_err(|e| format!("Failed to save tenant: {:?}", e))?;
//
//     info!("Tenant {} actualizado con el nuevo group_id {}", event.tenant_id, event.group_id);
//     Ok(())
// }

// fn get_item_type_from_slug(slug: &str) -> MenuItemType {
//     match slug {
//         "home" => {
//             MenuItemType::Page
//         },
//         "migration" => {
//             MenuItemType::Feature { feature: FeatureType::Migration }
//         },
//         _ => {
//             MenuItemType::Page
//         }
//     }
// }
