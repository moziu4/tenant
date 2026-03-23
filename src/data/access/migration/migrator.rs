use std::collections::HashSet;
use std::env;
use mongodb::{bson::doc, error::Error as MongoError, Collection, Database};
use chrono::{Utc, DateTime};
use dotenv::dotenv;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use crate::data::access::migration::{Migration, MigrationContext};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationLog {
    pub name: String,
    pub applied_at: DateTime<Utc>,
}


pub struct Migrator {
    db: Database,
    migrations: Vec<Box<dyn Migration + Send + Sync>>,
    applied_migrations: HashSet<String>,
}

impl Migrator {
    pub fn new(context: &MigrationContext) -> Self {
        dotenv().ok();
        let database_name = env::var("MONGO_DATABASE")
            .expect("Database name isn't found in environment variables.");

        Self {
            db: context.client.database(database_name.as_str()),
            migrations: vec![],
            applied_migrations: HashSet::new(),
        }
    }

    pub fn register_migration(mut self, migration: Box<dyn Migration + Send + Sync>) -> Self {
        // Agregamos las migraciones al contenedor
        self.migrations.push(migration);
        self
    }

    async fn load_applied_migrations(&mut self) -> Result<(), MongoError> {
        let migration_logs: Collection<MigrationLog> = self.db.collection("migrations");

        let cursor = migration_logs.find(doc! {}).await?;

        let logs: Vec<MigrationLog> = cursor.try_collect().await?;

        self.applied_migrations = logs
            .into_iter()
            .map(|log| log.name)
            .collect();
        Ok(())
    }

    pub async fn migrate(&mut self) -> Result<usize, MongoError> {
        // Cargar todas las migraciones ya aplicadas
        self.load_applied_migrations().await?;

        let migration_logs: Collection<MigrationLog> = self.db.collection("migrations");
        let mut applied_count = 0;

        for migration in &self.migrations {
            let name = migration.name().to_string();

            // Verificamos si la migración ya fue aplicada
            if self.applied_migrations.contains(&name) {
                info!("Migración '{}' ya aplicada. Saltando...", name);
                continue;
            }

            // Intentamos aplicar la migración
            info!("Aplicando migración '{}'", name);
            match migration.up(&MigrationContext {
                client: self.db.client().clone(),
            }).await {
                Ok(_) => {
                    info!("Migración '{}' aplicada correctamente", name);
                    migration_logs
                        .insert_one(
                            MigrationLog {
                                name: name.clone(),
                                applied_at: Utc::now(),
                            },
                        )
                        .await?;
                    self.applied_migrations.insert(name);
                    applied_count += 1;
                },
                Err(err) => {
                    error!("Error aplicando migración '{}': {:?}", name, err);
                    return Err(err);
                }
            }
        }

        if applied_count == 0 {
            info!("No hay nuevas migraciones para aplicar.");
        } else {
            info!("Número total de migraciones aplicadas: {}", applied_count);
        }
        Ok(applied_count)
    }
}
