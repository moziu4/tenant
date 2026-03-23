use futures_util::TryStreamExt;
use mongodb::{
    bson,
    bson::{doc, from_document, oid::ObjectId, to_document, Document},
    Collection,
};

use crate::core::domain::plans::plan_error::PlanError;
use crate::core::domain::plans::plan_type::Plan;
use crate::utils::domains_ids::PlanID;

#[derive(Clone)]
pub struct MongoPlanRepo
{
    collection: Collection<Document>,
}

impl MongoPlanRepo
{
    pub fn new(collection: Collection<Document>) -> Self
    {
        Self { collection }
    }

    pub async fn create(&self, mut new_plan: Plan) -> Result<Plan, PlanError>
    {
        if new_plan.id.is_none()
        {
            new_plan.id = Some(PlanID::new());
        }
        let collection = &self.collection;
        let plan_doc = to_document(&new_plan).map_err(|_| PlanError::PlanDocumentNotCreated)?;
        let _insert_result = collection.insert_one(plan_doc).await?;
        
        Ok(new_plan)
    }

    pub async fn fetch_all(&self) -> Result<Vec<Plan>, PlanError>
    {
        let filter = doc! {};
        let mut cursor = self.collection
            .find(filter)
            .await
            .map_err(|_| PlanError::PlanNotFound)?;

        let mut plans: Vec<Plan> = Vec::new();
        while let Some(plan_doc) = cursor.try_next()
            .await
            .map_err(|_| PlanError::PlanNotFound)?
        {
            let plan: Plan = from_document(plan_doc).map_err(|_| PlanError::PlanNotFound)?;
            plans.push(plan);
        }
        Ok(plans)
    }

    pub async fn fetch_by_id(&self, id: PlanID) -> Result<Plan, PlanError>
    {
        let collection = &self.collection;
        let filter = doc! { "_id": ObjectId::from(id)};
        let plan_doc = collection.find_one(filter)
            .await
            .map_err(|_| PlanError::PlanNotFound)?
            .ok_or(PlanError::PlanNotFound)?;

        let plan: Plan = bson::from_document(plan_doc).map_err(|_| PlanError::PlanNotFound)?;
        Ok(plan)
    }

    pub async fn save(&self, plan: Plan) -> Result<Plan, PlanError>
    {
        let collection = &self.collection;
        if let Some(plan_id) = &plan.id
        {
            let plan_doc = to_document(&plan).map_err(|_| PlanError::PlanDocumentNotCreated)?;

            let filter = doc! { "_id": ObjectId::from(plan_id.clone()) };
            let update_result = collection.update_one(filter, doc! { "$set": plan_doc })
                .await;

            match update_result
            {
                Ok(result) =>
                    {
                        if result.matched_count == 0
                        {
                            Err(PlanError::PlanNotFound)
                        }
                        else
                        {
                            Ok(plan)
                        }
                    },
                Err(_) => Err(PlanError::PlanDocNotUpdated),
            }
        }
        else
        {
            Err(PlanError::PlanNotFound)
        }
    }

    pub async fn delete(&self, id: PlanID) -> Result<(), PlanError>
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
                        Err(PlanError::PlanNotFound)
                    }
                    else
                    {
                        Ok(())
                    }
                },
            Err(_) => Err(PlanError::PlanNotFound),
        }
    }
}
