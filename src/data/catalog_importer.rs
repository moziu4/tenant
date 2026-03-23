use std::env;
use std::fs;
use std::sync::Arc;
use mongodb::bson::Document;
use serde::{Deserialize, Serialize};
use crate::context::Context;

#[derive(Debug, Serialize, Deserialize)]
pub struct Plan {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub limits: PlanLimits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanLimits {
    pub max_tenants: i32,
}

pub struct CatalogImporter {
    context: Arc<Context>,
}

impl CatalogImporter {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    pub async fn import_plans(&self) -> Result<(), String> {
        let json_path = env::var("CATALOG_JSON_PATH")
            .map_err(|_| "CATALOG_JSON_PATH env var not set".to_string())?;

        let content = fs::read_to_string(json_path)
            .map_err(|e| format!("Failed to read catalog file: {}", e))?;

        let plans: Vec<Plan> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse catalog JSON: {}", e))?;

        let collection = self.context.get_collection("plans");

        // Limpiar planes existentes antes de importar
        collection.delete_many(mongodb::bson::doc! {}).await
            .map_err(|e| format!("Failed to clear existing plans: {}", e))?;

        for plan in plans {
            let doc = mongodb::bson::to_document(&plan)
                .map_err(|e| format!("Failed to convert plan to document: {}", e))?;
            
            collection.insert_one(doc).await
                .map_err(|e| format!("Failed to insert plan: {}", e))?;
        }

        Ok(())
    }
}
