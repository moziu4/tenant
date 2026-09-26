use futures_util::TryStreamExt;
use mongodb::{
    bson,
    bson::{doc, from_document, oid::ObjectId, to_document, Document},
    Collection,
};

use crate::core::domain::organization::organization_error::OrganizationError;
use crate::core::domain::organization::organization_type::Organization;
use crate::utils::domains_ids::OrganizationID;

#[derive(Clone)]
pub struct MongoOrganizationRepo
{
    collection: Collection<Document>,
}

impl MongoOrganizationRepo
{
    pub fn new(collection: Collection<Document>) -> Self
    {
        Self { collection }
    }

    pub async fn create(&self, mut new_organization: Organization) -> Result<Organization, OrganizationError>
    {
        if new_organization.id.is_none()
        {
            new_organization.id = Some(OrganizationID::new());
        }
        let collection = &self.collection;
        let organization_doc = to_document(&new_organization).map_err(|_| OrganizationError::OrganizationDocumentNotCreated)?;
        let _insert_result = collection.insert_one(organization_doc).await?;
        
        Ok(new_organization)
    }

    pub async fn fetch_all(&self) -> Result<Vec<Organization>, OrganizationError>
    {
        let filter = doc! {};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| OrganizationError::OrganizationNotFound)?;

        let mut organizations: Vec<Organization> = Vec::new();
        while let Some(organization_doc) = cursor.try_next()
            .await
            .map_err(|_| OrganizationError::OrganizationNotFound)?
        {
            let organization: Organization = from_document(organization_doc).map_err(|_| OrganizationError::OrganizationNotFound)?;
            organizations.push(organization);
        }
        Ok(organizations)
    }

    pub async fn fetch_by_id(&self, id: OrganizationID) -> Result<Organization, OrganizationError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id)};
        let organization_doc = collection.find_one(filter)
            .await
            .map_err(|_| OrganizationError::OrganizationNotFound)?
            .ok_or(OrganizationError::OrganizationNotFound)?;

        let organization: Organization = bson::from_document(organization_doc).map_err(|_| OrganizationError::OrganizationNotFound)?;
        Ok(organization)
    }

    pub async fn save(&self, organization: Organization) -> Result<Organization, OrganizationError>
    {
        let collection = &self.collection;
        if let Some(organization_id) = &organization.id
        {
            let organization_doc = to_document(&organization).map_err(|_| OrganizationError::OrganizationDocumentNotCreated)?;

            let filter = doc! { "_id": ObjectId::from(organization_id.clone()) };
            let update_result = collection.update_one(filter, doc! { "$set": organization_doc })
                .await;

            match update_result
            {
                Ok(result) =>
                    {
                        if result.matched_count == 0
                        {
                            Err(OrganizationError::OrganizationNotFound)
                        }
                        else
                        {
                            Ok(organization)
                        }
                    },
                Err(_) => Err(OrganizationError::OrganizationDocNotUpdated),
            }
        }
        else
        {
            Err(OrganizationError::OrganizationNotFound)
        }
    }

    pub async fn delete(&self, id: OrganizationID) -> Result<(), OrganizationError>
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
                        Err(OrganizationError::OrganizationNotFound)
                    }
                    else
                    {
                        Ok(())
                    }
                },
            Err(_) => Err(OrganizationError::OrganizationNotFound),
        }
    }
}
