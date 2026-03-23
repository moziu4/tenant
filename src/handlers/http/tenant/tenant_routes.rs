use std::sync::Arc;

use actix_web::{
    web,
    web::{Json, Path},
    HttpRequest,
    HttpResponse,
    Responder,
};
use std::env;
use perms::has_permission;
use crate::core::domain::perms_cat::*;

use crate::{
    context::Context,
    core::{
        commands::tenant_command::TenantCommand,
        domain::tenant::{
            tenant_error::TenantError,
            tenant_type::{LoadTenantsByAgency, NewTenant},
        },
        operation::tenant_ops::TenantOps,
    },
    utils::domains_ids::{AgencyID, TenantID},
};

#[rustfmt::skip]
pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(
        web::scope("/api/tenant")
            .route("",              web::post().to(create_tenant))
            .route("/all",          web::get().to(load_all_tenants))
            .route("/by_agency",    web::post().to(load_tenants_by_agency))
            .route("/info",         web::get().to(get_tenant_by_host))
            .route("/{id}",         web::get().to(get_tenant_by_id))
            .route("/{id}",         web::patch().to(dispatch_commands))
            .route("/{id}",         web::delete().to(delete_tenant_by_id)),
    );
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

use crate::utils::hub_context::HubContext;

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

async fn load_tenants_by_agency(req: HttpRequest,
                               hub_ctx: HubContext,
                               context: web::Data<Arc<Context>>,
                               payload: Json<LoadTenantsByAgency>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    // Usamos el mismo permiso de lectura global o propio para agencias
    if !has_permission(secret, req, TENANT_READ_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_ops = TenantOps::new(&tenant_repo, &context);

    let agency_id = payload.into_inner().agency_id;

    // Para buscar por agencia, ignoramos el filtro de host que viene del Hub
    // para que devuelva todos los tenants de esa agencia.
    match tenant_ops.load_all_tenants(None, Some(agency_id)).await
    {
        Ok(tenants) => HttpResponse::Ok().json(tenants),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_tenant_by_host(hub_ctx: HubContext,
                            context: web::Data<Arc<Context>>) -> impl Responder
{
    println!("tenant_routes::get_tenant_by_host: {}", hub_ctx.tenant_id);
    if hub_ctx.tenant_id.is_empty() {
        return HttpResponse::BadRequest().body("No X-Tenant-Id found in request headers");
    }

    let tenant_repo = context.get_ref().get_tenant_repo();
    let tenant_ops = TenantOps::new(&tenant_repo, &context);

    match tenant_ops.load_tenant_by_host(hub_ctx.tenant_id).await
    {
        Ok(tenant) => HttpResponse::Ok().json(tenant),
        Err(TenantError::TenantNotFound) => HttpResponse::NotFound().body("Tenant not found for the provided host"),
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
