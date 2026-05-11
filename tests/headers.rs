//! Security-headers middleware test. Layers the *production* middleware
//! `rampart::serve::security_headers_layer` against a tiny test router and
//! asserts every header. Using the real function (rather than a copy)
//! catches drift between prod and test.

use axum::{Router, body::Body, http::Request, middleware, routing::get};
use rampart::serve::security_headers_layer;
use tower::ServiceExt;

#[tokio::test]
async fn security_headers_set_on_every_response() {
    let app = Router::new()
        .route("/x", get(|| async { "hi" }))
        .layer(middleware::from_fn(security_headers_layer));
    let resp = app
        .oneshot(Request::builder().uri("/x").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let h = resp.headers();
    assert_eq!(
        h.get("X-Frame-Options").map(|v| v.to_str().unwrap()),
        Some("DENY"),
    );
    assert_eq!(
        h.get("X-Content-Type-Options").map(|v| v.to_str().unwrap()),
        Some("nosniff"),
    );
    assert_eq!(
        h.get("Referrer-Policy").map(|v| v.to_str().unwrap()),
        Some("strict-origin-when-cross-origin"),
    );
    assert_eq!(
        h.get("Permissions-Policy").map(|v| v.to_str().unwrap()),
        Some("interest-cohort=()"),
    );

    let csp = h
        .get("Content-Security-Policy")
        .expect("CSP must be set")
        .to_str()
        .unwrap();
    // Substring-asserting key directives lets us tighten the prod CSP
    // (add directives) without breaking this test, while still failing
    // if any of these particular directives ever loosen.
    for directive in [
        "default-src 'self'",
        "script-src 'self'",
        "style-src 'self'",
        "frame-ancestors 'none'",
        "form-action 'self'",
    ] {
        assert!(csp.contains(directive), "CSP missing `{directive}`: {csp}");
    }
    // Critical: NO 'unsafe-inline' / 'unsafe-eval' anywhere.
    assert!(
        !csp.contains("unsafe-inline"),
        "CSP must not allow unsafe-inline: {csp}"
    );
    assert!(
        !csp.contains("unsafe-eval"),
        "CSP must not allow unsafe-eval: {csp}"
    );
}
