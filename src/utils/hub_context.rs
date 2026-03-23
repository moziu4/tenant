use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures_util::future::{ready, Ready};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HubContext {
    pub tenant_id: String,
    pub user_id: String,
    pub user_role: String,
}

impl FromRequest for HubContext {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let tenant_id = req.headers().get("X-Tenant-Id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        let user_id = req.headers().get("X-User-Id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();

        let user_role = req.headers().get("X-User-Role")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();

        // En un entorno real, aquí se podría validar que la petición provenga del Hub.
        // Pero el requerimiento dice que el Hub ya valida y confiamos si están presentes.

        ready(Ok(HubContext {
            tenant_id,
            user_id,
            user_role,
        }))
    }
}
