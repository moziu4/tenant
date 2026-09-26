use std::sync::Arc;
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::context::Context;
use crate::data::catalog_importer::CatalogImporter;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/catalog")
            .route("/import", web::post().to(import_catalog)),
    );
}

async fn import_catalog(_req: HttpRequest, context: web::Data<Arc<Context>>) -> impl Responder {
    // Note: The user said everything should have permissions EXCEPT the load we just did?
    // Wait: "los palnes menos la carga que acabamos de hacer tambien" 
    // -> means plans (getting them) should have perms, BUT NOT the load/import we just did.
    // So import_catalog doesn't need has_permission according to my reading.
    // However, if they want to READ plans, it should have it.
    // I will add a GET /api/catalog/plans with perms if needed, but the instruction says:
    // "todo lo que sea gestion de tenants creacion todo y de agncias debe de tener y los palnes menos la carga que acabamos de hacer tambien"
    // I'll interpret "los planes" as needing permissions for management/viewing.
    
    let importer = CatalogImporter::new(context.get_ref().clone());
    
    match importer.import_plans().await {
        Ok(_) => HttpResponse::Ok().body("Catalog imported successfully"),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}
