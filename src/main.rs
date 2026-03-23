use std::{env, io, sync::Arc};

use actix_cors::Cors;
use actix_web::{
    dev::RequestHead,
    http::header::{self, HeaderValue},
    web,
    App,
    HttpServer,
};
use env_logger::Env;
use tenant::{context::Context, db::connect_to_db, handlers::http, utils::cache::RedisCache};
use tenant::data::access::migration::{ MigrationContext};
use tenant::data::access::migration::mongo::migrate_mongo;
use tenant::handlers::messages::nats_handler::NatsHandler;

#[actix_web::main]
async fn main() -> io::Result<()>
{
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let client = connect_to_db().await;


    let redis_cache = RedisCache::new(&*env::var("REDIS_URI").unwrap()).expect("Error al conectar a Redis");
    let nats_handler = NatsHandler::new().await.expect("Error al conectar a NATS");

    let context = Arc::new(Context::new(client.clone(), redis_cache, nats_handler));
    let migration_context = MigrationContext{ client: client.clone()};
    match migrate_mongo(migration_context).await {
        Ok(applied) => {
            println!("Migraciones completadas. Total migraciones aplicadas: {}", applied);
        }
        Err(err) => {
            eprintln!("Error al ejecutar las migraciones: {:?}", err);
            std::process::exit(1); // Salida del programa si hay un error crítico
        }
    }


    HttpServer::new(move || {
        App::new().wrap(Cors::default().allowed_origin_fn(|origin: &HeaderValue, _req_head: &RequestHead| {
                                           if let Ok(origin_str) = origin.to_str()
                                           {
                                               origin_str == env::var("URL_FRONT_DEV").unwrap()
                                               || origin_str == env::var("URL_FRONT").unwrap()
                                           }
                                           else
                                           {
                                               false
                                           }
                                       })
                                       .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                                       .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
                                       .max_age(3600))
                  .app_data(web::Data::new(context.clone()))
                  .configure(http::tenant::tenant_routes::config)
                  .configure(http::agency::agency_routes::config)
                  .configure(http::plan::plan_routes::config)
                  .configure(http::catalog::config)
    }).bind(env::var("HTTP_BIND").unwrap().to_string())?
      .run()
      .await
}
