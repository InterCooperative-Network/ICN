use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use crate::services::{BlockchainService, IdentityService, GovernanceService};
use std::sync::Arc;
use crate::api::federation_resource_sharing::federation_resource_sharing_routes;
use crate::api::federation::federation_routes;
use crate::api::governance::governance_routes;
use crate::api::identity::identity_routes;

pub struct ApiServer {
    port: u16,
    blockchain_service: Arc<BlockchainService>,
    identity_service: Arc<IdentityService>,
    governance_service: Arc<GovernanceService>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl ApiServer {
    pub fn new(
        port: u16,
        blockchain_service: Arc<BlockchainService>,
        identity_service: Arc<IdentityService>,
        governance_service: Arc<GovernanceService>,
    ) -> Self {
        Self {
            port,
            blockchain_service,
            identity_service,
            governance_service,
        }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let blockchain_service = self.blockchain_service.clone();
        let identity_service = self.identity_service.clone();
        let governance_service = self.governance_service.clone();
        
        println!("Starting API server on port {}", self.port);

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(blockchain_service.clone()))
                .app_data(web::Data::new(identity_service.clone()))
                .app_data(web::Data::new(governance_service.clone()))
                .service(web::scope("/api/v1")
                    .configure(federation_resource_sharing_routes)
                    .configure(federation_routes)
                    .configure(governance_routes)
                    .configure(identity_routes)
                )
                .route("/health", web::get().to(health_check))
                .route("/blocks", web::get().to(get_blocks))
        })
        .bind(("0.0.0.0", self.port))?
        .run()
        .await
    }
}

// API handlers for testing
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some("API is running"),
        error: None,
    })
}

async fn get_blocks() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(Vec::<serde_json::Value>::new()),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_rt::test]
    async fn test_health_check() {
        let resp = health_check().await;
        // In a real test, we would check the response
    }
}
