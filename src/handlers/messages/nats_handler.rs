use std::env;
use async_nats::jetstream::{self, Context as JetStreamContext};
use crate::handlers::messages::StateChangedEvent;
use serde_json;

#[derive(Clone)]
pub struct NatsHandler {
    js: JetStreamContext,
}

impl NatsHandler {
    pub async fn new() -> Result<Self, String> {
        let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats:4222".to_string());
        let client = async_nats::connect(nats_url).await
            .map_err(|e| format!("Failed to connect to NATS: {}", e))?;
        
        let js = jetstream::new(client);
        
        // Opcional: Asegurarse de que el stream existe
        // En este caso, asumimos que el stream 'EVENTS' o similar está pre-configurado
        // o lo creamos si es necesario. Para simplicidad, publicaremos directamente.

        Ok(Self { js })
    }

    pub async fn publish_state_changed(&self, event: StateChangedEvent) -> Result<(), String> {
        let subject = match event.entity_type {
            crate::handlers::messages::EntityType::Tenant => format!("tenant.state.{}", event.entity_id),
            crate::handlers::messages::EntityType::Agency => format!("agency.state.{}", event.entity_id),
        };

        let payload = serde_json::to_vec(&event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;

        self.js.publish(subject, payload.into()).await
            .map_err(|e| format!("Failed to publish to JetStream: {}", e))?;

        Ok(())
    }
}
