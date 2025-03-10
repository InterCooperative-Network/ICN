use warp::Filter;
use warp::Rejection;

pub fn with_auth() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::any().map(|| ()).map(Ok).untuple_one()
}

pub async fn verify_auth_token(token: &str) -> bool {
    // TODO: Implement proper token verification
    !token.is_empty()
}