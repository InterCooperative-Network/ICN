use warp::Filter;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub resource_type: String,
    pub capacity: String,
    pub owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub resource_type: String,
    pub capacity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceResponse {
    pub id: String,
    pub resource_type: String,
    pub capacity: String,
    pub owner: String,
}

// Create some example resources for testing
fn get_resources() -> Vec<Resource> {
    vec![
        Resource {
            id: "res_1".to_string(),
            resource_type: "compute".to_string(),
            capacity: "10 CPU cores".to_string(),
            owner: "coop_1".to_string(),
        },
        Resource {
            id: "res_2".to_string(),
            resource_type: "storage".to_string(),
            capacity: "100 GB".to_string(),
            owner: "coop_2".to_string(),
        },
    ]
}

/// Resource routes handler
pub fn resource_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let list_route = warp::path!("api" / "v1" / "resources")
        .and(warp::get())
        .map(|| {
            info!("Resource list requested");
            let resources = get_resources();
            warp::reply::json(&serde_json::json!({ "resources": resources }))
        });

    let register_route = warp::path!("api" / "v1" / "resources")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request: ResourceRequest| {
            info!("Resource registration requested: {:?}", request);
            let id = format!("res_{}", rand::random::<u32>());
            let resource = Resource {
                id: id.clone(),
                resource_type: request.resource_type,
                capacity: request.capacity,
                owner: "current_user".to_string(), // In a real implementation, this would come from auth
            };
            warp::reply::json(&serde_json::json!({ "resource": resource }))
        });

    list_route.or(register_route)
} 