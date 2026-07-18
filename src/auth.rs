//! Four auth mechanisms (Basic / Bearer / Cookie / Passkey) feed one
//! `Principal` extractor. Argon2id verification is LRU-cached (16 / 60s)
//! to skip re-hashing on every request. Origin-check middleware rejects
//! cross-site mutations for Cookie auth; Basic/Bearer are exempt
//! (CLI/extension clients have no Origin).

use std::{
   num::NonZeroUsize,
   sync::{
      Mutex,
      OnceLock,
   },
   time::{
      Duration,
      Instant,
   },
};

use axum::{
   body::Body,
   extract::{
      FromRequestParts,
      State,
   },
   http::{
      HeaderMap,
      Method,
      Request,
      StatusCode,
      header,
      request::Parts,
   },
   middleware::Next,
   response::Response,
};
use data_encoding::{
   BASE64,
   BASE64URL_NOPAD,
};
use hmac_sha256::Hash;
use lru::LruCache;
use rampart_codegen::queries::{
   api_keys,
   sessions,
   users,
};
use rand::TryRngCore;
use time::OffsetDateTime;

use crate::{
   AppState,
   error::ApiError,
};

const VERIFY_TTL: Duration = Duration::from_secs(60);
const VERIFY_CACHE_SIZE: usize = 16;

pub const SESSION_COOKIE_NAME: &str = "rampart_session";
pub const SESSION_LIFETIME_DAYS: i64 = 30;

#[derive(Clone, Debug)]
pub struct Principal {
   pub user_id:  i64,
   pub is_admin: bool,
   pub via:      AuthVia,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AuthVia {
   Basic,
   Bearer,
   Cookie,
   Passkey,
}

/// LRU cache for argon2 verifications. Keying on the stored hash means
/// any password mutation (UI / CLI / admin reset) implicitly invalidates
/// — the new hash has a different fingerprint, the lookup misses, argon2
/// re-runs against the new hash.
pub struct VerifyCache {
   inner: Mutex<LruCache<(i64, [u8; 32], [u8; 8]), Instant>>,
}

impl VerifyCache {
   pub fn new() -> Self {
      Self {
         inner: Mutex::new(LruCache::new(
            NonZeroUsize::new(VERIFY_CACHE_SIZE).expect("non-zero"),
         )),
      }
   }

   fn fingerprint(user_id: i64, password: &[u8], stored_hash: &str) -> (i64, [u8; 32], [u8; 8]) {
      let hp = Hash::hash(password);
      let hh_full = Hash::hash(stored_hash.as_bytes());
      let mut hh_short = [0u8; 8];
      hh_short.copy_from_slice(&hh_full[..8]);
      (user_id, hp, hh_short)
   }

   fn is_fresh(&self, user_id: i64, password: &[u8], stored_hash: &str) -> bool {
      let key = Self::fingerprint(user_id, password, stored_hash);
      let mut cache = self.inner.lock().unwrap();
      matches!(cache.get(&key), Some(t) if t.elapsed() < VERIFY_TTL)
   }

   fn record(&self, user_id: i64, password: &[u8], stored_hash: &str) {
      let key = Self::fingerprint(user_id, password, stored_hash);
      let mut cache = self.inner.lock().unwrap();
      cache.put(key, Instant::now());
   }

   /// Call after any in-process password mutation. Out-of-process
   /// mutations rely on the hash-fingerprint in the cache key.
   pub fn invalidate_user(&self, user_id: i64) {
      let mut cache = self.inner.lock().unwrap();
      let to_drop: Vec<_> = cache
         .iter()
         .filter_map(|(k, _)| if k.0 == user_id { Some(*k) } else { None })
         .collect();
      for k in to_drop {
         cache.pop(&k);
      }
   }
}

pub fn hash_api_key(token: &str) -> Vec<u8> {
   Hash::hash(token.as_bytes()).to_vec()
}

/// Argon2id PHC hash. Used at signup, password change/reset.
pub fn hash_password(password: &str) -> anyhow::Result<String> {
   let mut salt = [0u8; 16];
   rand::rngs::OsRng
      .try_fill_bytes(&mut salt)
      .expect("OsRng must not fail");
   argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())
      .map_err(|e| anyhow::anyhow!("argon2: {e}"))
}

/// Pre-computed argon2id hash used for timing-equalization on missing
/// users / passkey-only accounts. Verifying against this is ~the same
/// cost as verifying against a real password hash, so an attacker
/// can't enumerate accounts from response timing.
fn timing_canary_hash() -> &'static str {
   static H: OnceLock<String> = OnceLock::new();
   H.get_or_init(|| {
      // Fixed salt is fine — the hash is throwaway, only its compute cost matters.
      let salt: [u8; 16] = *b"timing-canary-rp";
      argon2::hash_encoded(
         b"never-equal-to-any-real-password",
         &salt,
         &argon2::Config::default(),
      )
      .expect("argon2 hash")
   })
}

/// Run argon2 verify against the canary; always returns false. Used
/// to pay the same CPU cost on missing-user paths so timing doesn't
/// leak account presence.
fn pay_argon2_canary_cost(password: &[u8]) {
   let _ = argon2::verify_encoded(timing_canary_hash(), password);
}

/// Resolve a Basic auth pair. Returns None on any mismatch.
async fn resolve_basic(
   state: &AppState,
   user_email: &str,
   password: &str,
) -> Result<Option<Principal>, ApiError> {
   let c = state.pool.get().await?;
   let user = users::by_email_for_basic_auth()
      .bind(&c, &user_email)
      .opt()
      .await?;
   let Some(user) = user else {
      pay_argon2_canary_cost(password.as_bytes());
      return Ok(None);
   };
   let Some(hash) = user.password_hash.as_deref() else {
      // Passkey-only account.
      pay_argon2_canary_cost(password.as_bytes());
      return Ok(None);
   };
   if state
      .verify_cache
      .is_fresh(user.id, password.as_bytes(), hash)
   {
      return Ok(Some(Principal {
         user_id:  user.id,
         is_admin: user.is_admin,
         via:      AuthVia::Basic,
      }));
   }
   if argon2::verify_encoded(hash, password.as_bytes()).unwrap_or(false) {
      state
         .verify_cache
         .record(user.id, password.as_bytes(), hash);
      return Ok(Some(Principal {
         user_id:  user.id,
         is_admin: user.is_admin,
         via:      AuthVia::Basic,
      }));
   }
   Ok(None)
}

async fn resolve_bearer(state: &AppState, token: &str) -> Result<Option<Principal>, ApiError> {
   let digest = hash_api_key(token);
   let c = state.pool.get().await?;
   let Some(r) = api_keys::lookup_with_user().bind(&c, &digest).opt().await? else {
      return Ok(None);
   };
   let _ = api_keys::bump_last_used().bind(&c, &digest).await;
   Ok(Some(Principal {
      user_id:  r.user_id,
      is_admin: r.is_admin,
      via:      AuthVia::Bearer,
   }))
}

async fn resolve_cookie(
   state: &AppState,
   session_id: &[u8],
) -> Result<Option<Principal>, ApiError> {
   let c = state.pool.get().await?;
   let session_id_vec = session_id.to_vec();
   let Some(r) = sessions::lookup_with_user()
      .bind(&c, &session_id_vec)
      .opt()
      .await?
   else {
      return Ok(None);
   };
   if !r.enabled || r.expires_at <= OffsetDateTime::now_utc() {
      let _ = sessions::delete_by_id().bind(&c, &session_id_vec).await;
      return Ok(None);
   }
   let new_expiry = OffsetDateTime::now_utc() + time::Duration::hours(24 * SESSION_LIFETIME_DAYS);
   let _ = sessions::bump_last_seen()
      .bind(&c, &new_expiry, &session_id_vec)
      .await;
   Ok(Some(Principal {
      user_id:  r.user_id,
      is_admin: r.is_admin,
      via:      AuthVia::Cookie,
   }))
}

pub async fn extract_principal(
   state: &AppState,
   headers: &HeaderMap,
) -> Result<Option<Principal>, ApiError> {
   if let Some(auth) = headers
      .get(header::AUTHORIZATION)
      .and_then(|v| v.to_str().ok())
   {
      if let Some(rest) = auth.strip_prefix("Bearer ") {
         return resolve_bearer(state, rest.trim()).await;
      }
      if let Some(rest) = auth.strip_prefix("Basic ") {
         if let Ok(decoded) = BASE64.decode(rest.trim().as_bytes()) {
            if let Ok(utf8) = std::str::from_utf8(&decoded) {
               if let Some((u, p)) = utf8.split_once(':') {
                  return resolve_basic(state, u, p).await;
               }
            }
         }
         return Ok(None);
      }
   }
   if let Some(session_id) = extract_session_id(headers) {
      return resolve_cookie(state, &session_id).await;
   }
   Ok(None)
}

fn extract_session_id(headers: &HeaderMap) -> Option<Vec<u8>> {
   let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
   for piece in cookie_header.split(';') {
      let (k, v) = piece.trim().split_once('=')?;
      if k == SESSION_COOKIE_NAME {
         return BASE64URL_NOPAD.decode(v.as_bytes()).ok();
      }
   }
   None
}

pub async fn auth_layer(
   State(state): State<AppState>,
   mut req: Request<Body>,
   next: Next,
) -> Response {
   let headers = req.headers().clone();
   match extract_principal(&state, &headers).await {
      Ok(Some(p)) => {
         req.extensions_mut().insert(p);
         next.run(req).await
      },
      Ok(None) => unauthorized(&req, &headers),
      Err(e) => {
         tracing::error!(error = ?e, "auth layer error");
         Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("500 auth error"))
            .unwrap()
      },
   }
}

/// Browser GET navigations get a 303 to `/login?next=<original>`;
/// API clients get a 401. We deliberately omit `WWW-Authenticate: Basic`
/// — the header triggers the OS Basic-auth modal on every unauthed
/// dashboard hit. Basic auth is still accepted by the extractor.
fn unauthorized(req: &Request<Body>, headers: &HeaderMap) -> Response {
   let is_browser_nav = req.method() == Method::GET
      && headers
         .get(header::ACCEPT)
         .and_then(|v| v.to_str().ok())
         .is_some_and(|a| a.contains("text/html"));

   if is_browser_nav {
      let path = req
         .uri()
         .path_and_query()
         .map(|p| p.as_str())
         .unwrap_or("/");
      // Skip the next= round-trip when the user is already at /
      let location = if path == "/" {
         "/login".to_owned()
      } else {
         let mut q = url::form_urlencoded::Serializer::new(String::new());
         q.append_pair("next", path);
         format!("/login?{}", q.finish())
      };
      return Response::builder()
         .status(StatusCode::SEE_OTHER)
         .header(header::LOCATION, location)
         .body(Body::from(""))
         .unwrap();
   }

   Response::builder()
      .status(StatusCode::UNAUTHORIZED)
      .body(Body::from("401 unauthorized"))
      .unwrap()
}

/// Origin-check middleware for PUBLIC form-POST routes (login, signup,
/// forgot, reset). Cookie-establishing endpoints all need this — a
/// cross-site form-POST to /login can otherwise overwrite the
/// victim's `rampart_session` with attacker credentials and force the
/// victim into the attacker's account (Codex P1.3). The main
/// `origin_layer` doesn't cover these because they sit on the public
/// router, outside the auth chain.
///
/// Unlike `origin_layer`, there's no Basic/Bearer bypass: these
/// endpoints are browser-form-only (programmatic clients use the
/// `/api/v1/auth/*` JSON endpoints, which sit on the public router
/// AND have their own Origin treatment for the same reason).
pub async fn public_form_origin_layer(
   State(state): State<AppState>,
   req: Request<Body>,
   next: Next,
) -> Response {
   if !matches!(
      req.method(),
      &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE
   ) {
      return next.run(req).await;
   }
   let origin = req
      .headers()
      .get(header::ORIGIN)
      .and_then(|v| v.to_str().ok());
   let referer = req
      .headers()
      .get(header::REFERER)
      .and_then(|v| v.to_str().ok());
   if !same_origin_post_ok(&state.config.public_origin, origin, referer) {
      tracing::warn!(
          path = %req.uri().path(),
          origin = origin.unwrap_or("<absent>"),
          referer = referer.unwrap_or("<absent>"),
          expected = %state.config.public_origin,
          "rejected public form POST: cross-origin or missing Origin/Referer"
      );
      return Response::builder()
         .status(StatusCode::FORBIDDEN)
         .body(Body::from("403 cross-origin form POST rejected"))
         .unwrap();
   }
   next.run(req).await
}

/// OWASP-style "Origin OR Referer" CSRF gate. Origin is the strict signal
/// when present and concrete; some browser/proxy combinations strip it to
/// `null` for top-level form submissions (notably Firefox under certain
/// Referrer-Policy values + cookie SameSite interactions, and proxied
/// Cloudflare paths), so we fall back to a Referer prefix-check when
/// Origin isn't usable. Referer alone is weaker than Origin, but
/// browser-controlled and not spoofable by attacker JS, so the fallback
/// only really opens the door for adversaries who can already MITM the
/// victim's browser — at which point Origin/Referer aren't the defense.
fn same_origin_post_ok(public_origin: &str, origin: Option<&str>, referer: Option<&str>) -> bool {
   match origin {
      Some(o) if o == public_origin => true,
      Some(o) if o == "null" => referer_matches(public_origin, referer),
      Some(_) => false,
      None => referer_matches(public_origin, referer),
   }
}

fn referer_matches(public_origin: &str, referer: Option<&str>) -> bool {
   let Some(r) = referer else {
      return false;
   };
   // Accept `<public_origin>` or `<public_origin>/...` — i.e., a Referer
   // whose origin component equals public_origin. Substring check on a
   // boundary prefix; the trailing-character requirement rules out
   // origin-prefix attacks like `https://bunker.rampart.email.evil.example/`.
   r == public_origin
      || (r.starts_with(public_origin)
         && matches!(
            r.as_bytes().get(public_origin.len()),
            Some(b'/') | Some(b'?') | Some(b'#')
         ))
}

/// Origin-check middleware: reject cookie-authed mutations with
/// missing/mismatched Origin. Basic and Bearer paths are exempt
/// because those clients (CLI, extension) don't send Origin.
pub async fn origin_layer(
   State(state): State<AppState>,
   req: Request<Body>,
   next: Next,
) -> Response {
   let method = req.method().clone();
   if !matches!(
      method,
      Method::POST | Method::PUT | Method::PATCH | Method::DELETE
   ) {
      return next.run(req).await;
   }

   let via = req.extensions().get::<Principal>().map(|p| p.via);
   // CLI paths (Basic, Bearer) bypass Origin check — they have no Origin.
   if matches!(via, Some(AuthVia::Basic) | Some(AuthVia::Bearer)) {
      return next.run(req).await;
   }

   let origin = req
      .headers()
      .get(header::ORIGIN)
      .and_then(|v| v.to_str().ok());

   let ok = match origin {
      Some(o) => o == state.config.public_origin,
      None => false, // cookie-auth w/o Origin is a suspicious cross-site form POST
   };

   if !ok {
      return Response::builder()
         .status(StatusCode::FORBIDDEN)
         .body(Body::from("403 cross-origin mutation rejected"))
         .unwrap();
   }
   next.run(req).await
}

/// Middleware: reject any request reaching this layer unless the
/// resolved Principal is admin. Mounted on the admin sub-routers in
/// api/web so admin-only routes can't be added without the check —
/// "forgot to call require_admin" can't happen because the route is
/// only reachable through this middleware. Returns 404 (stealth) so
/// non-admins can't enumerate admin endpoints.
pub async fn admin_layer(req: Request<Body>, next: Next) -> Response {
   let is_admin = req
      .extensions()
      .get::<Principal>()
      .is_some_and(|p| p.is_admin);
   if is_admin {
      next.run(req).await
   } else {
      Response::builder()
         .status(StatusCode::NOT_FOUND)
         .body(Body::from("404 not found"))
         .unwrap()
   }
}

/// Typed admin principal — extractor that pulls Principal from request
/// extensions and rejects non-admins. Routes mounted under `admin_layer`
/// are already gated; this exists so handler signatures *document* the
/// admin requirement (`AdminPrincipal(p)` vs `Extension<Principal>`).
/// Belt-and-suspenders — the layer is the structural guarantee, the
/// extractor is the in-handler annotation.
#[derive(Clone, Debug)]
pub struct AdminPrincipal(pub Principal);

impl<S: Send + Sync> FromRequestParts<S> for AdminPrincipal {
   type Rejection = ApiError;

   async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
      let p = parts
         .extensions
         .get::<Principal>()
         .cloned()
         .ok_or(ApiError::NotFound)?;
      if !p.is_admin {
         return Err(ApiError::NotFound);
      }
      Ok(AdminPrincipal(p))
   }
}

pub fn new_session_id() -> [u8; 32] {
   let mut buf = [0u8; 32];
   rand::rngs::OsRng
      .try_fill_bytes(&mut buf)
      .expect("OsRng must not fail");
   buf
}

pub fn session_cookie_value(session_id: &[u8]) -> String {
   BASE64URL_NOPAD.encode(session_id)
}
