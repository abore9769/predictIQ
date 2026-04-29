use std::{sync::Arc, time::Duration};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{security::{extract_client_ip, RateLimitConfig, RateLimiter}, AppState};

/// Newsletter endpoint rate limiting — policy is driven by config:
/// `NEWSLETTER_RATE_LIMIT_MAX` (default 5) per `NEWSLETTER_RATE_LIMIT_WINDOW_SECS` (default 3600).
pub async fn newsletter_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<std::net::SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_client_ip(&headers, connect_info.as_ref(), true);
    let allowed = state
        .newsletter_rate_limiter
        .allow(
            &format!("newsletter:{ip}"),
            state.config.newsletter_rate_limit_max,
            Duration::from_secs(state.config.newsletter_rate_limit_window_secs),
        )
        .await;

    if !allowed {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

/// Contact endpoint rate limiting (3 req/hour per IP)
pub async fn contact_rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<std::net::SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_client_ip(&headers, connect_info.as_ref(), true);
    let config = RateLimitConfig::new(3, Duration::from_secs(3600));

    if !limiter.check(&format!("contact:{}", ip), &config).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

/// Analytics endpoint rate limiting (1000 req/min per session)
/// 
/// Key generation strategy (no allocations leak):
/// - Prefers x-session-id header if provided (client-supplied session tracking)
/// - Falls back to client IP if header missing
/// - Both paths use owned Strings; no Box::leak or static string tricks
pub async fn analytics_rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<std::net::SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_client_ip(&headers, connect_info.as_ref(), true);
    
    // Build rate limit key using either session ID or IP as fallback.
    // This uses owned String creation in all paths — no Box::leak required.
    let session_id = headers
        .get("x-session-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or(ip);

    let config = RateLimitConfig::new(1000, Duration::from_secs(60));

    if !limiter
        .check(&format!("analytics:{}", session_id), &config)
        .await
    {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

/// Admin endpoint rate limiting (30 req/min per IP)
pub async fn admin_rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<std::net::SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_client_ip(&headers, connect_info.as_ref(), true);
    let config = RateLimitConfig::new(30, Duration::from_secs(60));

    if !limiter.check(&format!("admin:{}", ip), &config).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
