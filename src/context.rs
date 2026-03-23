use std::{env, sync::Arc};

use mongodb::{bson::Document, Client, Collection};

use crate::{
    data::access::{
        agency_repo::MongoAgencyRepo,
        tenant_repo::MongoTenantRepo,
        plan_repo::MongoPlanRepo,
    },
    utils::cache::RedisCache,
    handlers::messages::nats_handler::NatsHandler,
};

#[derive(Clone)]
pub struct Context
{
    pub client:    Arc<Client>,
    agency_repo:  Arc<MongoAgencyRepo>,
    tenant_repo:  Arc<MongoTenantRepo>,
    plan_repo:    Arc<MongoPlanRepo>,
    redis_cache:   Arc<RedisCache>,
    nats_handler: Arc<NatsHandler>,
}


impl Context
{
    pub fn new(client: Client, redis_cache: RedisCache, nats_handler: NatsHandler) -> Self
    {
        let arc_client = Arc::new(client);
        let db_name = env::var("MONGO_DATABASE").expect("Var MONGO_DATABASE no definida");
        
        let agency_collection = arc_client.database(&db_name).collection("agency");
        let tenant_collection = arc_client.database(&db_name).collection("tenant");
        let plan_collection = arc_client.database(&db_name).collection("plan");

        Self { 
               client:        arc_client.clone(),
               agency_repo:  Arc::new(MongoAgencyRepo::new(agency_collection)),
               tenant_repo:  Arc::new(MongoTenantRepo::new(tenant_collection)),
               plan_repo:    Arc::new(MongoPlanRepo::new(plan_collection)),
               redis_cache:   Arc::new(redis_cache), 
               nats_handler: Arc::new(nats_handler),
        }
    }


    pub fn get_agency_repo(&self) -> Arc<MongoAgencyRepo>
    {
        Arc::clone(&self.agency_repo)
    }

    pub fn get_tenant_repo(&self) -> Arc<MongoTenantRepo>
    {
        Arc::clone(&self.tenant_repo)
    }

    pub fn get_plan_repo(&self) -> Arc<MongoPlanRepo>
    {
        Arc::clone(&self.plan_repo)
    }
    
    pub fn get_redis_cache(&self) -> Arc<RedisCache>
    {
        Arc::clone(&self.redis_cache)
    }

    pub fn get_nats_handler(&self) -> Arc<NatsHandler>
    {
        Arc::clone(&self.nats_handler)
    }

    pub fn get_collection(&self, collection: &str) -> Collection<Document>
    {
        let db_name = env::var("MONGO_DATABASE").expect("Var MONGO_DATABASE no definida");
        self.client
            .database(&db_name)
            .collection(collection)
    }
}
