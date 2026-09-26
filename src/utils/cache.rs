use std::sync::Arc;

use redis::{AsyncCommands, Client, RedisError};
use serde_json;

use crate::{
    core::domain::tenant::tenant_type::Tenant,
    utils::cache_error::CacheError,
};

// Estructura para manejar Redis
#[derive(Clone)]
pub struct RedisCache
{
    client: Arc<Client>,
}


impl RedisCache
{
    pub fn new(connection_string: &str) -> Result<Self, RedisError>
    {
        let client = Client::open(connection_string)?;
        Ok(Self { client: Arc::new(client) })
    }
    
    pub fn get_client(&self) -> &Client
    {
        &self.client
    }

    pub async fn invalidate_cache(&self, cache_key: &str) -> Result<(), RedisError>
    {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let _: () = con.del(cache_key).await?; // Eliminamos la clave específica
        Ok(())
    }

    pub async fn get_tenants(&self, cache_key: &str) -> Result<Option<Vec<Tenant>>, CacheError>
    {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let cached_value: Option<String> = con.get(cache_key).await?;
        if let Some(value) = cached_value {
            let tenants: Vec<Tenant> = serde_json::from_str(&value)?;
            return Ok(Some(tenants));
        }
        Ok(None)
    }

    pub async fn set_tenants(&self, cache_key: &str, tenants: Vec<Tenant>, ttl: usize) -> Result<(), CacheError>
    {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let serialized = serde_json::to_string(&tenants)?;
        con.set_ex::<_, _, ()>(cache_key, serialized, ttl as u64).await?;
        Ok(())
    }
}
