use warp::Filter;
use crate::services::identity_service::{IdentityService, IdentityServiceImpl};
use std::sync::Arc;
use icn_core::verifiable_credentials::{VerifiableCredential, Proof}; // Import VerifiableCredential and Proof
use icn_zkp::zk_snark; // Import zk-SNARK
use icn_crypto::KeyPair; // Import KeyPair for signature verification

pub fn identity_routes(
    identity_service: Arc<dyn IdentityService>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let create_identity = warp::path!("api" / "v1" / "identity" / "create")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_identity_service(identity_service.clone()))
        .and_then(handle_create_identity);

    let get_identity = warp::path!("api" / "v1" / "identity" / "get" / String)
        .and(warp::get())
        .and(with_identity_service(identity_service.clone()))
        .and_then(handle_get_identity);

    let rotate_key = warp::path!("api" / "v1" / "identity" / "rotate_key" / String)
        .and(warp::post())
        .and(with_identity_service(identity_service.clone()))
        .and_then(handle_rotate_key);

    let revoke_key = warp::path!("api" / "v1" / "identity" / "revoke_key" / String)
        .and(warp::post())
        .and(with_identity_service(identity_service.clone()))
        .and_then(handle_revoke_key);

    create_identity.or(get_identity).or(rotate_key).or(revoke_key)
}

fn with_identity_service(
    identity_service: Arc<dyn IdentityService>,
) -> impl Filter<Extract = (Arc<dyn IdentityService>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || identity_service.clone())
}

async fn handle_create_identity(
    identity: String,
    identity_service: Arc<dyn IdentityService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Verify signature using icn-crypto
    if !verify_signature(&identity, &identity_service).await {
        return Err(warp::reject::custom("Invalid signature"));
    }

    identity_service.create_identity(&identity).await.map_err(|e| {
        warp::reject::custom(warp::reject::custom(e))
    })?;

    // Generate ICN-compliant verifiable credential
    let credential = VerifiableCredential {
        credential_type: "IdentityCredential".to_string(),
        issuer_did: "did:icn:issuer".to_string(),
        subject_did: identity.clone(),
        issuance_date: chrono::Utc::now().to_rfc3339(),
        expiration_date: None,
        credential_status: None,
        credential_schema: None,
        proof: Proof {
            type_: "Ed25519Signature2018".to_string(),
            created: chrono::Utc::now().to_rfc3339(),
            proof_purpose: "assertionMethod".to_string(),
            verification_method: "did:icn:issuer#keys-1".to_string(),
            jws: "example-jws".to_string(),
        },
    };

    Ok(warp::reply::json(&credential))
}

async fn handle_get_identity(
    identity: String,
    identity_service: Arc<dyn IdentityService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let data = identity_service.get_identity(&identity).await.map_err(|e| {
        warp::reject::custom(warp::reject::custom(e))
    })?;

    // Generate zk-SNARK proof for identity validation
    let proof = zk_snark::generate_proof(&data);

    Ok(warp::reply::json(&proof))
}

async fn handle_rotate_key(
    identity: String,
    identity_service: Arc<dyn IdentityService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    identity_service.rotate_key(&identity).await.map_err(|e| {
        warp::reject::custom(warp::reject::custom(e))
    })?;
    Ok(warp::reply::with_status("Key rotated", warp::http::StatusCode::OK))
}

async fn handle_revoke_key(
    identity: String,
    identity_service: Arc<dyn IdentityService>,
) -> Result<impl warp::Reply, warp::Rejection> {
    identity_service.revoke_key(&identity).await.map_err(|e| {
        warp::reject::custom(warp::reject::custom(e))
    })?;
    Ok(warp::reply::with_status("Key revoked", warp::http::StatusCode::OK))
}

async fn verify_signature(identity: &str, identity_service: &Arc<dyn IdentityService>) -> bool {
    if let Some(public_key) = identity_service.get_public_key(identity).await {
        let key_pair = KeyPair {
            public_key,
            private_key: vec![], // Not needed for verification
            algorithm: icn_crypto::Algorithm::Secp256k1, // Assuming Secp256k1 for this example
        };
        return key_pair.verify(identity.as_bytes(), identity.as_bytes());
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::Filter;
    use crate::services::identity_service::IdentityServiceImpl;
    use std::sync::Arc;
    use warp::http::StatusCode;

    #[tokio::test]
    async fn test_create_identity() {
        let identity_service = Arc::new(IdentityServiceImpl::new(Arc::new(Database::new())));
        let api = identity_routes(identity_service);

        let resp = warp::test::request()
            .method("POST")
            .path("/api/v1/identity/create")
            .json(&"did:icn:test")
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_get_identity() {
        let identity_service = Arc::new(IdentityServiceImpl::new(Arc::new(Database::new())));
        let api = identity_routes(identity_service);

        let resp = warp::test::request()
            .method("GET")
            .path("/api/v1/identity/get/did:icn:test")
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rotate_key() {
        let identity_service = Arc::new(IdentityServiceImpl::new(Arc::new(Database::new())));
        let api = identity_routes(identity_service);

        let resp = warp::test::request()
            .method("POST")
            .path("/api/v1/identity/rotate_key/did:icn:test")
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let identity_service = Arc::new(IdentityServiceImpl::new(Arc::new(Database::new())));
        let api = identity_routes(identity_service);

        let resp = warp::test::request()
            .method("POST")
            .path("/api/v1/identity/revoke_key/did:icn:test")
            .reply(&api)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
