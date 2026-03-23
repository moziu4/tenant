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

use crate::core::domain::agency::{
    agency_type::{Agency, NewAgency},
    AgencyEntity,
};
use crate::core::operation::agency_ops::AgencyOps;
use crate::core::commands::agency_command::AgencyCommand;
use crate::utils::domains_ids::AgencyID;
use crate::context::Context;

#[rustfmt::skip]
pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(
        web::scope("/api/agency")
            .route("",              web::post().to(create_agency))
            .route("/all",          web::get().to(load_all_agencies))
            .route("/{id}",         web::get().to(get_agency_by_id))
            .route("/{id}",         web::patch().to(dispatch_commands))
            .route("/{id}",         web::delete().to(delete_agency_by_id)),
    );
}

use crate::utils::hub_context::HubContext;

async fn create_agency(req: HttpRequest, 
                      _hub_ctx: HubContext,
                      context: web::Data<Arc<Context>>, 
                      payload: Json<NewAgency>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, AGENCY_CREATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let agency_repo = context.get_ref().get_agency_repo();
    let agency_ops = AgencyOps::new(&agency_repo, &context);

    match agency_ops.create_agency(payload.into_inner()).await
    {
        Ok(agency) => HttpResponse::Ok().json(agency),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn load_all_agencies(req: HttpRequest, 
                          _hub_ctx: HubContext,
                          context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, AGENCY_READ_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let agency_repo = context.get_ref().get_agency_repo();
    match agency_repo.fetch_all().await
    {
        Ok(agencies) => HttpResponse::Ok().json(agencies),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_agency_by_id(req: HttpRequest, 
                          _hub_ctx: HubContext,
                          path: Path<String>, 
                          context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, AGENCY_READ_OWN).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let agency_id = match AgencyID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let agency_repo = context.get_ref().get_agency_repo();
    match agency_repo.fetch_by_id(agency_id).await
    {
        Ok(agency) => HttpResponse::Ok().json(agency),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn delete_agency_by_id(req: HttpRequest, 
                             _hub_ctx: HubContext,
                             path: Path<String>, 
                             context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, AGENCY_DELETE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let agency_id = match AgencyID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let agency_repo = context.get_ref().get_agency_repo();
    let agency_ops = AgencyOps::new(&agency_repo, &context);
    match agency_ops.delete_agency(agency_id).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn dispatch_commands(req: HttpRequest,
                           _hub_ctx: HubContext,
                           path: Path<String>,
                           context: web::Data<Arc<Context>>,
                           payload: Json<Vec<AgencyCommand>>)
                           -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, AGENCY_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let agency_id = match AgencyID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let agency_repo = context.get_ref().get_agency_repo();
    let agency_ops = AgencyOps::new(&agency_repo, &context);

    match agency_ops.dispatch_commands(agency_id, &payload.into_inner()).await
    {
        Ok(agency) => HttpResponse::Ok().json(agency),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
