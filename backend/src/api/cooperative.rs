use warp::Filter;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cooperative {
    pub id: String,
    pub name: String,
    pub member_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CooperativeRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequest {
    pub cooperative_id: String,
}

// Create some example cooperatives for testing
fn get_cooperatives() -> Vec<Cooperative> {
    vec![
        Cooperative {
            id: "coop_1".to_string(),
            name: "Example Cooperative 1".to_string(),
            member_count: 5,
        },
        Cooperative {
            id: "coop_2".to_string(),
            name: "Example Cooperative 2".to_string(),
            member_count: 10,
        },
    ]
}

/// Cooperative routes handler
pub fn cooperative_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let list_route = warp::path!("api" / "v1" / "cooperatives")
        .and(warp::get())
        .map(|| {
            info!("Cooperative list requested");
            let cooperatives = get_cooperatives();
            warp::reply::json(&serde_json::json!({ "cooperatives": cooperatives }))
        });

    let create_route = warp::path!("api" / "v1" / "cooperatives")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request: CooperativeRequest| {
            info!("Cooperative creation requested: {:?}", request);
            let id = format!("coop_{}", rand::random::<u32>());
            let cooperative = Cooperative {
                id: id.clone(),
                name: request.name,
                member_count: 1,
            };
            warp::reply::json(&serde_json::json!({ "cooperative": cooperative }))
        });

    let join_route = warp::path!("api" / "v1" / "cooperatives" / "join")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request: JoinRequest| {
            info!("Join cooperative requested: {:?}", request);
            warp::reply::json(&serde_json::json!({ 
                "status": "success",
                "message": format!("Successfully joined cooperative {}", request.cooperative_id)
            }))
        });

    list_route.or(create_route).or(join_route)
}
