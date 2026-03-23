pub mod mongo;
pub mod migrator;
use mongodb::Client;
use mongodb::error::Error as MongoError;

pub struct MigrationContext {
    pub client: Client,
}

#[async_trait::async_trait]
pub trait Migration: Send + Sync {
    fn name(&self) -> &'static str; // Nombre único de la migración
    async fn up(&self, context: &MigrationContext) -> Result<(), MongoError>;

}