use governor::middleware::NoOpMiddleware;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};

/// Create a rate limiting layer for general API requests
/// Uses IP address as the key for rate limiting
#[must_use]
pub fn create_rate_limiter(
    requests_per_min: u32,
) -> GovernorLayer<'static, SmartIpKeyExtractor, NoOpMiddleware> {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(u64::from((requests_per_min / 60).max(1)))
            .burst_size(requests_per_min.max(10))
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    GovernorLayer {
        config: Box::leak(governor_conf),
    }
}

/// Create a rate limiter based on requests per hour
#[must_use]
pub fn create_rate_limiter_per_hour(
    requests_per_hour: u32,
) -> GovernorLayer<'static, SmartIpKeyExtractor, NoOpMiddleware> {
    let per_minute = (requests_per_hour / 60).max(1);
    create_rate_limiter(per_minute)
}

/// Get a simple global rate limiter layer using the default `SmartIpKeyExtractor`
#[must_use]
pub fn get_rate_limiter_layer() -> GovernorLayer<'static, SmartIpKeyExtractor, NoOpMiddleware> {
    let config = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2) // ~120 per minute
            .burst_size(10)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    GovernorLayer {
        config: Box::leak(config),
    }
}
