use warp::Filter;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub did: String,
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub did: String,
    pub public_key: String,
}

// Create some example identities for testing
fn get_identities() -> Vec<Identity> {
    vec![
        Identity {
            did: "did:icn:abc123".to_string(),
            public_key: "0x123456".to_string(),
        },
        Identity {
            did: "did:icn:def456".to_string(),
            public_key: "0x789012".to_string(),
        },
    ]
}

/// Identity routes handler
pub fn identity_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let list_route = warp::path!("api" / "v1" / "identities")
        .and(warp::get())
        .map(|| {
            info!("Identity list requested");
            let identities = get_identities();
            warp::reply::json(&serde_json::json!({ "identities": identities }))
        });

    let create_route = warp::path!("api" / "v1" / "identities")
        .and(warp::post())
        .and(warp::body::json())
        .map(|request: IdentityRequest| {
            info!("Identity creation requested: {:?}", request);
            let did = format!("did:icn:{}", rand::random::<u32>());
            let public_key = format!("0x{:x}", rand::random::<u64>());
            let identity = Identity {
                did: did.clone(),
                public_key,
            };
            warp::reply::json(&serde_json::json!({ "identity": identity }))
        });

    list_route.or(create_route)
}
