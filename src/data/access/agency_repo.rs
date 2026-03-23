use futures_util::TryStreamExt;
use mongodb::{
    bson,
    bson::{doc, from_document, oid::ObjectId, to_document, Document},
    Collection,
};

use crate::core::domain::agency::agency_error::AgencyError;
use crate::core::domain::agency::agency_type::Agency;
use crate::utils::domains_ids::AgencyID;

#[derive(Clone)]
pub struct MongoAgencyRepo
{
    collection: Collection<Document>,
}

impl MongoAgencyRepo
{
    pub fn new(collection: Collection<Document>) -> Self
    {
        Self { collection }
    }

    pub async fn create(&self, mut new_agency: Agency) -> Result<Agency, AgencyError>
    {
        if new_agency.id.is_none()
        {
            new_agency.id = Some(AgencyID::new());
        }
        let collection = &self.collection;
        let agency_doc = to_document(&new_agency).map_err(|_| AgencyError::AgencyDocumentNotCreated)?;
        let _insert_result = collection.insert_one(agency_doc).await?;
        
        Ok(new_agency)
    }

    pub async fn fetch_all(&self) -> Result<Vec<Agency>, AgencyError>
    {
        let filter = doc! {};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| AgencyError::AgencyNotFound)?;

        let mut agencies: Vec<Agency> = Vec::new();
        while let Some(agency_doc) = cursor.try_next()
            .await
            .map_err(|_| AgencyError::AgencyNotFound)?
        {
            let agency: Agency = from_document(agency_doc).map_err(|_| AgencyError::AgencyNotFound)?;
            agencies.push(agency);
        }
        Ok(agencies)
    }

    pub async fn fetch_by_id(&self, id: AgencyID) -> Result<Agency, AgencyError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id)};
        let agency_doc = collection.find_one(filter)
            .await
            .map_err(|_| AgencyError::AgencyNotFound)?
            .ok_or(AgencyError::AgencyNotFound)?;

        let agency: Agency = bson::from_document(agency_doc).map_err(|_| AgencyError::AgencyNotFound)?;
        Ok(agency)
    }

    pub async fn save(&self, agency: Agency) -> Result<Agency, AgencyError>
    {
        let collection = &self.collection;
        if let Some(agency_id) = &agency.id
        {
            let agency_doc = to_document(&agency).map_err(|_| AgencyError::AgencyDocumentNotCreated)?;

            let filter = doc! { "_id": ObjectId::from(agency_id.clone()) };
            let update_result = collection.update_one(filter, doc! { "$set": agency_doc })
                .await;

            match update_result
            {
                Ok(result) =>
                    {
                        if result.matched_count == 0
                        {
                            Err(AgencyError::AgencyNotFound)
                        }
                        else
                        {
                            Ok(agency)
                        }
                    },
                Err(_) => Err(AgencyError::AgencyDocNotUpdated),
            }
        }
        else
        {
            Err(AgencyError::AgencyNotFound)
        }
    }

    pub async fn delete(&self, id: AgencyID) -> Result<(), AgencyError>
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
                        Err(AgencyError::AgencyNotFound)
                    }
                    else
                    {
                        Ok(())
                    }
                },
            Err(_) => Err(AgencyError::AgencyNotFound),
        }
    }
}
