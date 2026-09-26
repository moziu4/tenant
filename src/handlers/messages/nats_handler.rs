use std::env;
use async_nats::jetstream::{self, Context as JetStreamContext};
use crate::handlers::messages::{StateChangedEvent, TenantCreatedEvent};
use serde_json;

#[derive(Clone)]
pub struct NatsHandler {
    client: async_nats::Client,
    js: JetStreamContext,
}

impl NatsHandler {
    pub async fn new() -> Result<Self, String> {
        let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats:4222".to_string());
        let client = async_nats::connect(nats_url).await
            .map_err(|e| format!("Failed to connect to NATS: {}", e))?;
        
        let js = jetstream::new(client.clone());
        
        // Asegurarse de que el stream TENANTS existe
        let stream_name = "TENANTS";
        let subjects = vec!["tenant.created".to_string(), "tenant.state.*".to_string(), "organization.state.*".to_string()];

        let _stream = match js.get_stream(stream_name).await {
            Ok(stream) => stream,
            Err(_) => {
                js.create_stream(jetstream::stream::Config {
                    name: stream_name.to_string(),
                    subjects,
                    ..Default::default()
                }).await.map_err(|e| format!("Failed to create stream {}: {}", stream_name, e))?
            }
        };

        Ok(Self { client, js })
    }

    pub fn get_client(&self) -> async_nats::Client {
        self.client.clone()
    }

    pub async fn publish_tenant_created(&self, event: TenantCreatedEvent) -> Result<(), String> {
        let subject = "tenant.created".to_string();
        let payload = serde_json::to_vec(&event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;

        self.js.publish(subject, payload.into()).await
            .map_err(|e| format!("Failed to publish to JetStream: {}", e))?;

        Ok(())
    }

    pub async fn publish_state_changed(&self, event: StateChangedEvent) -> Result<(), String> {
        let subject = match event.entity_type {
            crate::handlers::messages::EntityType::Tenant => format!("tenant.state.{}", event.entity_id),
            crate::handlers::messages::EntityType::Organization => format!("organization.state.{}", event.entity_id),
        };

        let payload = serde_json::to_vec(&event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;

        self.js.publish(subject, payload.into()).await
            .map_err(|e| format!("Failed to publish to JetStream: {}", e))?;

        Ok(())
    }
}
