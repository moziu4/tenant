use futures_util::TryStreamExt;
use mongodb::{
    bson,
    bson::{doc, from_document, oid::ObjectId, to_document, Document},
    Collection,
};

use crate::core::domain::tenant::tenant_error::TenantError;
use crate::core::domain::tenant::tenant_type::Tenant;
use crate::utils::domains_ids::{OrganizationID, TenantID};

#[derive(Clone)]
pub struct MongoTenantRepo
{
    collection: Collection<Document>,
}

impl MongoTenantRepo
{
    pub fn new(collection: Collection<Document>) -> Self
    {
        Self { collection }
    }

    pub async fn create(&self, mut new_tenant: Tenant) -> Result<Tenant, TenantError>
    {
        if new_tenant.id.is_none()
        {
            new_tenant.id = Some(TenantID::new());
        }
        let collection = &self.collection;
        let tenant_doc = to_document(&new_tenant).map_err(|_| TenantError::TenantDocumentNotCreated)?;
        let _insert_result = collection.insert_one(tenant_doc).await?;
        
        Ok(new_tenant)
    }

    pub async fn fetch_all(&self, host: Option<String>, organization_id: Option<OrganizationID>) -> Result<Vec<Tenant>, TenantError>
    {
        let mut filter = doc! {};
        if let Some(h) = host {
            filter.insert("host", h.trim());
        }
        if let Some(oid) = organization_id {
            filter.insert("organization_id", ObjectId::from(oid));
        }
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|e| {
                eprintln!("Error in fetch_all find: {:?}", e);
                TenantError::TenantNotFound
            })?;

        let mut tenants: Vec<Tenant> = Vec::new();
        while let Some(tenant_doc) = cursor.try_next()
            .await
            .map_err(|e| {
                eprintln!("Error in fetch_all next: {:?}", e);
                TenantError::TenantNotFound
            })?
        {
            match from_document(tenant_doc) {
                Ok(tenant) => tenants.push(tenant),
                Err(e) => {
                    eprintln!("Error deserializing tenant in fetch_all: {:?}", e);
                    // Decidimos si ignorar el documento corrupto o fallar. 
                    // Por ahora fallamos para ser consistentes con fetch_by_host.
                    return Err(TenantError::TenantNotFound);
                }
            }
        }
        Ok(tenants)
    }

    pub async fn fetch_by_id(&self, id: TenantID) -> Result<Tenant, TenantError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id.clone())};
        let tenant_doc = collection.find_one(filter)
            .await
            .map_err(|e| {
                eprintln!("Error in fetch_by_id find_one: {:?}", e);
                TenantError::TenantNotFound
            })?
            .ok_or_else(|| {
                eprintln!("Tenant not found for ID: {}", id);
                TenantError::TenantNotFound
            })?;

        let tenant: Tenant = bson::from_document(tenant_doc).map_err(|e| {
            eprintln!("Error deserializing tenant in fetch_by_id: {:?}", e);
            TenantError::TenantNotFound
        })?;
        Ok(tenant)
    }

    pub async fn save(&self, tenant: Tenant) -> Result<Tenant, TenantError>
    {
        let collection = &self.collection;
        if let Some(tenant_id) = &tenant.id
        {
            let tenant_doc = to_document(&tenant).map_err(|_| TenantError::TenantDocumentNotCreated)?;

            let filter = doc! { "_id": ObjectId::from(tenant_id.clone()) };
            let update_result = collection.update_one(filter, doc! { "$set": tenant_doc })
                .await;

            match update_result
            {
                Ok(result) =>
                    {
                        if result.matched_count == 0
                        {
                            Err(TenantError::TenantNotFound)
                        }
                        else
                        {
                            Ok(tenant)
                        }
                    },
                Err(_) => Err(TenantError::TenantDocNotUpdated),
            }
        }
        else
        {
            Err(TenantError::TenantNotFound)
        }
    }

    pub async fn delete(&self, id: TenantID) -> Result<(), TenantError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id) };
        let delete_result = collection.delete_one(filter).await;

        match delete_result
        {
            Ok(result) =>
                {
                    if result.deleted_count == 0
                    {
                        Err(TenantError::TenantNotFound)
                    }
                    else
                    {
                        Ok(())
                    }
                },
            Err(_) => Err(TenantError::TenantNotFound),
        }
    }

    pub async fn fetch_by_host(&self, host: String) -> Result<Tenant, TenantError>
    {
        let filter = doc! { "host": host.trim() };
        let tenant_doc = self.collection.find_one(filter)
            .await
            .map_err(|e| {
                eprintln!("Error fetching by host: {:?}", e);
                TenantError::TenantNotFound
            })?
            .ok_or_else(|| {
                eprintln!("Tenant not found for host: {}", host);
                TenantError::TenantNotFound
            })?;

        let tenant: Tenant = bson::from_document(tenant_doc).map_err(|e| {
            eprintln!("Error deserializing tenant: {:?}", e);
            TenantError::TenantNotFound
        })?;
        Ok(tenant)
    }
}
