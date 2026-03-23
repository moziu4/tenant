use mongodb::error::Error as MongoError;
use crate::data::access::migration::{Migration, MigrationContext};
use async_trait::async_trait;

pub struct Migration001;

#[async_trait]
impl Migration for Migration001 {
    fn name(&self) -> &'static str {
        "001_initial_collections"
    }

    async fn up(&self, context: &MigrationContext) -> Result<(), MongoError> {
        let db_name = std::env::var("MONGO_DATABASE").unwrap_or_else(|_| "tenant_admin".to_string());
        let db = context.client.database(&db_name);

        // Crear colecciones
        let collections = vec!["agency", "tenant", "migrations", "plans"];
        
        for coll_name in collections {
            // MongoDB crea las colecciones automáticamente al insertar, 
            // pero podemos crearlas explícitamente si queremos asegurar su existencia.
            match db.create_collection(coll_name).await {
                Ok(_) => println!("Colección '{}' creada correctamente.", coll_name),
                Err(e) => {
                    // Si la colección ya existe, ignoramos el error
                    if !e.to_string().contains("already exists") {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }
}
