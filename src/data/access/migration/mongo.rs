use crate::data::access::migration::{MigrationContext};
use crate::data::access::migration::migrator::{Migrator};

use mongodb::{ error::Error as MongoError};
use crate::data::access::migration::mongo::v01::Migration001;

pub mod v01;
pub async fn migrate_mongo(context: MigrationContext) -> Result<usize, MongoError> {
    let mut migrator = Migrator::new(&context)
        .register_migration(Box::new(Migration001));

    let applied = migrator.migrate().await?;
    Ok(applied)
}