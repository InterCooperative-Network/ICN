use warp::Filter;

pub fn cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allow_headers(vec!["content-type"])
}

pub mod auth;
pub mod cors;

pub use cors::cors;