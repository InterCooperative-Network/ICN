use warp::Filter;
use crate::services::reputation_service::{get_reputation, adjust_reputation};

pub fn reputation_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("reputation")
        .and(
            warp::path("get")
                .and(warp::get())
                .and(warp::query::<String>())
                .and_then(get_reputation)
        )
        .or(
            warp::path("adjust")
                .and(warp::post())
                .and(warp::body::json())
                .and_then(adjust_reputation)
        )
}
