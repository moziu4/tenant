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

use crate::core::domain::organization::organization_type::NewOrganization;
use crate::core::operation::organization_ops::OrganizationOps;
use crate::core::commands::organization_command::OrganizationCommand;
use crate::utils::domains_ids::OrganizationID;
use crate::context::Context;

#[rustfmt::skip]
pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(
        web::scope("/api/organization")
            .route("",              web::post().to(create_organization))
            .route("/all",          web::get().to(load_all_organizations))
            .route("/{id}",         web::get().to(get_organization_by_id))
            .route("/{id}",         web::patch().to(dispatch_commands))
            .route("/{id}",         web::delete().to(delete_organization_by_id)),
    );
}

use crate::utils::hub_context::HubContext;

async fn create_organization(req: HttpRequest, 
                            _hub_ctx: HubContext,
                            context: web::Data<Arc<Context>>, 
                            payload: Json<NewOrganization>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, ORGANIZATION_CREATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let organization_repo = context.get_ref().get_organization_repo();
    let organization_ops = OrganizationOps::new(&organization_repo, &context);

    match organization_ops.create_organization(payload.into_inner()).await
    {
        Ok(organization) => HttpResponse::Ok().json(organization),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn load_all_organizations(req: HttpRequest, 
                                _hub_ctx: HubContext,
                                context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, ORGANIZATION_READ_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let organization_repo = context.get_ref().get_organization_repo();
    match organization_repo.fetch_all().await
    {
        Ok(organizations) => HttpResponse::Ok().json(organizations),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_organization_by_id(req: HttpRequest, 
                                _hub_ctx: HubContext,
                                path: Path<String>, 
                                context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, ORGANIZATION_READ_OWN).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let organization_id = match OrganizationID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let organization_repo = context.get_ref().get_organization_repo();
    match organization_repo.fetch_by_id(organization_id).await
    {
        Ok(organization) => HttpResponse::Ok().json(organization),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn delete_organization_by_id(req: HttpRequest, 
                                   _hub_ctx: HubContext,
                                   path: Path<String>, 
                                   context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, ORGANIZATION_DELETE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let organization_id = match OrganizationID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let organization_repo = context.get_ref().get_organization_repo();
    let organization_ops = OrganizationOps::new(&organization_repo, &context);
    match organization_ops.delete_organization(organization_id).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn dispatch_commands(req: HttpRequest,
                           _hub_ctx: HubContext,
                           path: Path<String>,
                           context: web::Data<Arc<Context>>,
                           payload: Json<Vec<OrganizationCommand>>)
                           -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, ORGANIZATION_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let organization_id = match OrganizationID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let organization_repo = context.get_ref().get_organization_repo();
    let organization_ops = OrganizationOps::new(&organization_repo, &context);

    match organization_ops.dispatch_commands(organization_id, &payload.into_inner()).await
    {
        Ok(organization) => HttpResponse::Ok().json(organization),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
