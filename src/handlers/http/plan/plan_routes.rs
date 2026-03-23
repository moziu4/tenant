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

use crate::core::domain::plans::{
    plan_type::{Plan, NewPlan},
};
use crate::core::operation::plan_ops::PlanOps;
use crate::core::commands::plan_command::PlanCommand;
use crate::utils::domains_ids::PlanID;
use crate::context::Context;
use crate::utils::hub_context::HubContext;

#[rustfmt::skip]
pub fn config(cfg: &mut web::ServiceConfig)
{
    cfg.service(
        web::scope("/api/plan")
            .route("",              web::post().to(create_plan))
            .route("/all",          web::get().to(load_all_plans))
            .route("/{id}",         web::get().to(get_plan_by_id))
            .route("/{id}",         web::patch().to(dispatch_commands))
            .route("/{id}",         web::delete().to(delete_plan_by_id)),
    );
}

async fn create_plan(req: HttpRequest, 
                    _hub_ctx: HubContext,
                    context: web::Data<Arc<Context>>, 
                    payload: Json<NewPlan>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, PLAN_CREATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let plan_repo = context.get_ref().get_plan_repo();
    let plan_ops = PlanOps::new(&plan_repo, &context);

    match plan_ops.create_plan(payload.into_inner()).await
    {
        Ok(plan) => HttpResponse::Ok().json(plan),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn load_all_plans(req: HttpRequest, 
                        _hub_ctx: HubContext,
                        context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, PLAN_READ_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let plan_repo = context.get_ref().get_plan_repo();
    match plan_repo.fetch_all().await
    {
        Ok(plans) => HttpResponse::Ok().json(plans),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn get_plan_by_id(req: HttpRequest, 
                        _hub_ctx: HubContext,
                        path: Path<String>, 
                        context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, PLAN_READ_OWN).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let plan_id = match PlanID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let plan_repo = context.get_ref().get_plan_repo();
    match plan_repo.fetch_by_id(plan_id).await
    {
        Ok(plan) => HttpResponse::Ok().json(plan),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn delete_plan_by_id(req: HttpRequest, 
                           _hub_ctx: HubContext,
                           path: Path<String>, 
                           context: web::Data<Arc<Context>>) -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, PLAN_DELETE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let plan_id = match PlanID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let plan_repo = context.get_ref().get_plan_repo();
    let plan_ops = PlanOps::new(&plan_repo, &context);
    
    match plan_ops.delete_plan(plan_id).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

async fn dispatch_commands(req: HttpRequest,
                           _hub_ctx: HubContext,
                           path: Path<String>,
                           context: web::Data<Arc<Context>>,
                           payload: Json<Vec<PlanCommand>>)
                           -> impl Responder
{
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY not found");
    if !has_permission(secret, req, PLAN_UPDATE_GLOBAL).await
    {
        return HttpResponse::Forbidden().body("No permission");
    }

    let plan_id = match PlanID::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ID format"),
    };

    let plan_repo = context.get_ref().get_plan_repo();
    let plan_ops = PlanOps::new(&plan_repo, &context);

    match plan_ops.dispatch_commands(plan_id, &payload.into_inner()).await
    {
        Ok(plan) => HttpResponse::Ok().json(plan),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
