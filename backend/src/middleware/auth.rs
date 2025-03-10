use warp::Filter;

pub fn with_auth() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::any()
        .map(|| ())
        .untuple_one()
}