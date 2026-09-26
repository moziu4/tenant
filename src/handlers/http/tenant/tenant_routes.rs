#![allow(dead_code, unused_imports)]

use std::sync::Arc;

use actix_web::{
    web,
    web::{Json, Path, Query},
    HttpRequest,
    HttpResponse,
    Responder,
};
use serde::Deserialize;
use std::env;
use perms::has_permission;
use crate::core::domain::perms_cat::*;

use crate::{
    context::Context,
    core::{
        commands::tenant_command::TenantCommand,
        domain::tenant::{
            tenant_error::TenantError,
            tenant_type::{LoadTenantsByOrganization, NewTenant},
            menu::Menu,
            TenantEntity,
        },
        operation::tenant_ops::TenantOps,
    },
    utils::domains_ids::TenantID,
};

#[rustfmt::skip]
pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(
        web::scope("/api/tenant")
            .route("",                      web::post().to(create_tenant))
            .route("/all",          web::get().to(load_all_tenants))
            .route("/by_organization", web::post().to(load_tenants_by_organization))
            .route("/info",         web::get().to(get_tenant_by_host))
            // Endpoints de menús deshabilitados temporalmente
            // .route("/menus/public", web::get().to(get_public_menus))
            // .route("/menus",        web::get().to(get_all_menus))
            // .route("/menus",        web::post().to(add_menu))
            // .route("/menus/{name}", web::patch().to(update_menu))
            // .route("/menus/{name}", web::delete().to(delete_menu))
            // .route("/{id}/menus/public", web::get().to(get_public_menus_with_id))
            // .route("/{id}/menus",        web::get().to(get_all_menus_with_id))
            // .route("/{id}/menus",        web::post().to(add_menu_with_id))
            // .route("/{id}/menus/{name}", web::patch().to(update_menu_with_id))
            // .route("/{id}/menus/{name}", web::delete().to(delete_menu_with_id))
            .route("/{id}",         web::get().to(get_tenant_by_id))
            .route("/{id}",         web::patch().to(dispatch_commands))
            .route("/{id}",         web::delete().to(delete_tenant_by_id)),
    );
    // Scope de menús deshabilitado temporalmente
    // cfg.service(
    //     web::scope("/api/{id}/menus")
    //         .route("/public", web::get().to(get_public_menus_with_id))
    //         .route("",        web::get().to(get_all_menus_with_id))
    //         .route("",        web::post().to(add_menu_with_id))
    //         .route("/{name}", web::patch().to(update_menu_with_id))
    //         .route("/{name}", web::delete().to(delete_menu_with_id)),
    // );
}

use crate::utils::hub_context::HubContext;

#[derive(Deserialize)]
struct TenantQuery {
    tenant_id: Option<String>,
}

fn get_effective_tenant_id(
    path_id: Option<String>,
    query: Query<TenantQuery>,
    hub_ctx: HubContext
) -> Option<String> {
    if let Some(id) = path_id {
        if !id.is_empty() {
            return Some(id);
        }
    }
    if let Some(id) = query.tenant_id.clone() {
        if !id.is_empty() {
            return Some(id);
        }
    }
    if !hub_ctx.tenant_id.is_empty() {
        return Some(hub_ctx.tenant_id);
    }
    None
}

async fn create_tenant(req: HttpRequest, 
                       hub_ctx: HubContext,
                       context: web::Data<Arc<Context>>, 
                       payload: Json<NewTenant>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_CREATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let mut new_tenant = payload.into_inner();
    // Si viene del Hub con un host y el payload tiene host vacío, usamos el del Hub
    if !hub_ctx.tenant_id.is_empty() && new_tenant.host.is_empty() {
        new_tenant.host = hub_ctx.tenant_id;
    }

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_ops = TenantOps::new(&tenant_repo, &context);

    match tenant_ops.create_tenant(new_tenant).await
    {
        Ok(tenant) => HttpResponse::Ok().json(tenant),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn load_all_tenants(req: HttpRequest, 
                         hub_ctx: HubContext,
                         context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_READ_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_ops = TenantOps::new(&tenant_repo, &context);

    // Si viene del Hub con un tenant_id (host), filtramos.
    let host_filter = if hub_ctx.tenant_id.is_empty() {
        None
    } else {
        Some(hub_ctx.tenant_id)
    };

    match tenant_ops.load_all_tenants(host_filter, None).await
    {
        Ok(tenants) => HttpResponse::Ok().json(tenants),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn load_tenants_by_organization(req: HttpRequest,
                                     _hub_ctx: HubContext,
                                     context: web::Data<Arc<Context>>,
                                     payload: Json<LoadTenantsByOrganization>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    // Usamos el mismo permiso de lectura global o propio para organizaciones
    if !has_permission(secret, req, TENANT_READ_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_ops = TenantOps::new(&tenant_repo, &context);

    let organization_id = payload.into_inner().organization_id;

    // Para buscar por organizacion, ignoramos el filtro de host que viene del Hub
    // para que devuelva todos los tenants de esa organizacion.
    match tenant_ops.load_all_tenants(None, Some(organization_id)).await
    {
        Ok(tenants) => HttpResponse::Ok().json(tenants),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_tenant_by_host(hub_ctx: HubContext,
                            context: web::Data<Arc<Context>>) -> impl Responder
{
    println!("tenant_routes::get_tenant_by_host (raw): {}", hub_ctx.tenant_id);
    if hub_ctx.tenant_id.is_empty() {
        return HttpResponse::BadRequest().body("No X-Tenant-Id found in request headers");
    }

    let tenant_repo = context.get_ref().get_tenant_repo();

    // Intentamos cargar por ID o por Host de forma flexible
    match get_entity_by_flexible_id(hub_ctx.tenant_id.clone(), &tenant_repo, &context).await {
        Ok(entity) => {
            let filtered_tenant = entity.get_filtered_tenant(hub_ctx.language);
            HttpResponse::Ok().json(filtered_tenant)
        },
        Err(TenantError::TenantNotFound) => HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_entity_by_flexible_id<'a>(id_or_host: String, 
                                     repo: &'a crate::data::access::tenant_repo::MongoTenantRepo, 
                                     context: &web::Data<Arc<Context>>) -> Result<TenantEntity<'a>, TenantError> {
    match TenantID::parse_str(&id_or_host) {
        Ok(id) => TenantEntity::load_by_id(id, repo).await,
        Err(_) => {
            let tenant_ops = TenantOps::new(repo, context);
            let props = tenant_ops.load_tenant_by_host(id_or_host).await?;
            Ok(TenantEntity::from_props(props, repo))
        }
    }
}

async fn get_public_menus(hub_ctx: HubContext,
                        query: Query<TenantQuery>,
                        context: web::Data<Arc<Context>>) -> impl Responder
{
    let tenant_id_str = match get_effective_tenant_id(None, query, hub_ctx.clone()) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();

    match get_entity_by_flexible_id(tenant_id_str, &tenant_repo, &context).await {
        Ok(entity) => HttpResponse::Ok().json(entity.get_public_menus(None)),
        Err(TenantError::TenantNotFound) => HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_public_menus_with_id(hub_ctx: HubContext,
                                 query: Query<TenantQuery>,
                                 path: Path<String>,
                                 context: web::Data<Arc<Context>>) -> impl Responder
{
    let tenant_id_str = match get_effective_tenant_id(Some(path.into_inner()), query, hub_ctx.clone()) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();

    match get_entity_by_flexible_id(tenant_id_str, &tenant_repo, &context).await {
        Ok(entity) => HttpResponse::Ok().json(entity.get_public_menus(None)),
        Err(TenantError::TenantNotFound) => HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_all_menus(req: HttpRequest,
                       hub_ctx: HubContext,
                       query: Query<TenantQuery>,
                       context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_id_str = match get_effective_tenant_id(None, query, hub_ctx) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_id = match TenantID::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Tenant ID"),
    };

    match TenantEntity::load_by_id(tenant_id, &tenant_repo).await {
        Ok(entity) => HttpResponse::Ok().json(entity.get_props().menus.clone()),
        Err(TenantError::TenantNotFound) => HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_all_menus_with_id(req: HttpRequest,
                              hub_ctx: HubContext,
                              query: Query<TenantQuery>,
                              path: Path<String>,
                              context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_id_str = match get_effective_tenant_id(Some(path.into_inner()), query, hub_ctx) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_id = match TenantID::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Tenant ID"),
    };

    match TenantEntity::load_by_id(tenant_id, &tenant_repo).await {
        Ok(entity) => HttpResponse::Ok().json(entity.get_props().menus.clone()),
        Err(TenantError::TenantNotFound) => HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn add_menu(req: HttpRequest,
                  hub_ctx: HubContext,
                  query: Query<TenantQuery>,
                  context: web::Data<Arc<Context>>,
                  payload: Json<Menu>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_id_str = match get_effective_tenant_id(None, query, hub_ctx) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_id = match TenantID::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Tenant ID"),
    };

    let mut entity = match TenantEntity::load_by_id(tenant_id, &tenant_repo).await {
        Ok(entity) => entity,
        Err(TenantError::TenantNotFound) => return HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if let Err(e) = entity.add_menu(payload.into_inner()) {
        return HttpResponse::BadRequest().json(e.to_string());
    }

    match entity.save().await {
        Ok(tenant) => {
            let tenant_ops = TenantOps::new(&tenant_repo, context.get_ref());
            let _ = tenant_ops.clear_cache().await;
            HttpResponse::Ok().json(tenant.menus)
        },
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn add_menu_with_id(req: HttpRequest,
                         hub_ctx: HubContext,
                         query: Query<TenantQuery>,
                         path: Path<String>,
                         context: web::Data<Arc<Context>>,
                         payload: Json<Menu>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_id_str = match get_effective_tenant_id(Some(path.into_inner()), query, hub_ctx) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_id = match TenantID::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Tenant ID"),
    };

    let mut entity = match TenantEntity::load_by_id(tenant_id, &tenant_repo).await {
        Ok(entity) => entity,
        Err(TenantError::TenantNotFound) => return HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if let Err(e) = entity.add_menu(payload.into_inner()) {
        return HttpResponse::BadRequest().json(e.to_string());
    }

    match entity.save().await {
        Ok(tenant) => {
            let tenant_ops = TenantOps::new(&tenant_repo, context.get_ref());
            let _ = tenant_ops.clear_cache().await;
            HttpResponse::Ok().json(tenant.menus)
        },
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn update_menu(req: HttpRequest,
                     hub_ctx: HubContext,
                     query: Query<TenantQuery>,
                     path: Path<String>,
                     context: web::Data<Arc<Context>>,
                     payload: Json<Menu>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let name = path.into_inner();
    let tenant_id_str = match get_effective_tenant_id(None, query, hub_ctx) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_id = match TenantID::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Tenant ID"),
    };

    let mut entity = match TenantEntity::load_by_id(tenant_id, &tenant_repo).await {
        Ok(entity) => entity,
        Err(TenantError::TenantNotFound) => return HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if let Err(e) = entity.update_specific_menu(name, payload.into_inner()) {
        return HttpResponse::BadRequest().json(e.to_string());
    }

    match entity.save().await {
        Ok(tenant) => {
            let tenant_ops = TenantOps::new(&tenant_repo, context.get_ref());
            let _ = tenant_ops.clear_cache().await;
            HttpResponse::Ok().json(tenant.menus)
        },
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn update_menu_with_id(req: HttpRequest,
                             hub_ctx: HubContext,
                             query: Query<TenantQuery>,
                             path: Path<(String, String)>,
                             context: web::Data<Arc<Context>>,
                             payload: Json<Menu>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let (tenant_id_str_from_path, name) = path.into_inner();
    let tenant_id_str = match get_effective_tenant_id(Some(tenant_id_str_from_path), query, hub_ctx) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_id = match TenantID::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Tenant ID"),
    };

    let mut entity = match TenantEntity::load_by_id(tenant_id, &tenant_repo).await {
        Ok(entity) => entity,
        Err(TenantError::TenantNotFound) => return HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if let Err(e) = entity.update_specific_menu(name, payload.into_inner()) {
        return HttpResponse::BadRequest().json(e.to_string());
    }

    match entity.save().await {
        Ok(tenant) => {
            let tenant_ops = TenantOps::new(&tenant_repo, context.get_ref());
            let _ = tenant_ops.clear_cache().await;
            HttpResponse::Ok().json(tenant.menus)
        },
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn delete_menu(req: HttpRequest,
                     hub_ctx: HubContext,
                     query: Query<TenantQuery>,
                     path: Path<String>,
                     context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let name = path.into_inner();
    let tenant_id_str = match get_effective_tenant_id(None, query, hub_ctx) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_id = match TenantID::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Tenant ID"),
    };

    let mut entity = match TenantEntity::load_by_id(tenant_id, &tenant_repo).await {
        Ok(entity) => entity,
        Err(TenantError::TenantNotFound) => return HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if let Err(e) = entity.delete_menu(name) {
        return HttpResponse::BadRequest().json(e.to_string());
    }

    match entity.save().await {
        Ok(tenant) => {
            let tenant_ops = TenantOps::new(&tenant_repo, context.get_ref());
            let _ = tenant_ops.clear_cache().await;
            HttpResponse::Ok().json(tenant.menus)
        },
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn delete_menu_with_id(req: HttpRequest,
                             hub_ctx: HubContext,
                             query: Query<TenantQuery>,
                             path: Path<(String, String)>,
                             context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let (tenant_id_str_from_path, name) = path.into_inner();
    let tenant_id_str = match get_effective_tenant_id(Some(tenant_id_str_from_path), query, hub_ctx) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().body("No tenant_id found in path, query or headers"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_id = match TenantID::parse_str(&tenant_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Tenant ID"),
    };

    let mut entity = match TenantEntity::load_by_id(tenant_id, &tenant_repo).await {
        Ok(entity) => entity,
        Err(TenantError::TenantNotFound) => return HttpResponse::NotFound().body("Tenant not found"),
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    if let Err(e) = entity.delete_menu(name) {
        return HttpResponse::BadRequest().json(e.to_string());
    }

    match entity.save().await {
        Ok(tenant) => {
            let tenant_ops = TenantOps::new(&tenant_repo, context.get_ref());
            let _ = tenant_ops.clear_cache().await;
            HttpResponse::Ok().json(tenant.menus)
        },
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_tenant_by_id(req: HttpRequest, 
                         _hub_ctx: HubContext,
                         path: Path<String>, 
                         context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_READ_OWN).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_id = match TenantID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_ops = TenantOps::new(&tenant_repo, &context);

    match tenant_ops.load_tenant_by_id(tenant_id).await
    {
        Ok(tenant) => HttpResponse::Ok().json(tenant),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn dispatch_commands(req: HttpRequest,
                           _hub_ctx: HubContext,
                           path: Path<String>,
                           context: web::Data<Arc<Context>>,
                           payload: Json<Vec<TenantCommand>>)
                           -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_id = match TenantID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_ops = TenantOps::new(&tenant_repo, &context);

    match tenant_ops.dispatch_commands(tenant_id, &payload.into_inner()).await
    {
        Ok(tenant) => HttpResponse::Ok().json(tenant),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn delete_tenant_by_id(req: HttpRequest, 
                            _hub_ctx: HubContext,
                            path: Path<String>, 
                            context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, TENANT_DELETE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_id = match TenantID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_ops = TenantOps::new(&tenant_repo, &context);

    match tenant_ops.delete_tenant(tenant_id).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
