use std::time::Duration;
use tower::ServiceBuilder;
use tower::limit::RateLimit;
use warp::Filter;

pub fn with_rate_limit<F: Filter>(
    filter: F,
    requests_per_second: u64
) -> impl Filter<Extract = F::Extract, Error = warp::Rejection> + Clone {
    let rate_limiter = ServiceBuilder::new()
        .layer(RateLimit::new(
            requests_per_second, 
            Duration::from_secs(1)
        ))
        .service(warp::service(filter));

    warp::service(rate_limiter)
}
