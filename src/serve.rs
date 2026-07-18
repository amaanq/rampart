//! `rampart serve` — axum wiring.

use std::{
   net::SocketAddr,
   sync::Arc,
};

use anyhow::{
   Context as _,
   Result,
};
use axum::{
   Form,
   Json,
   Router,
   extract::{
      ConnectInfo,
      Path,
      Query,
      Request,
      State,
   },
   http::{
      HeaderMap,
      HeaderValue,
      StatusCode,
      header,
   },
   middleware,
   response::{
      Html,
      IntoResponse as _,
      Redirect,
      Response,
   },
   routing,
};
use data_encoding::{
   BASE64,
   BASE64URL_NOPAD,
};
use rampart_codegen::queries::{
   sessions,
   users,
   webauthn,
};
use rand::rngs::SysRng;
use serde::Deserialize;
use time::{
   Duration,
   OffsetDateTime,
};
use tokio::{
   net::TcpListener,
   signal,
};
use tower::ServiceBuilder;
use tower_http::{
   services::ServeDir,
   set_header::SetResponseHeaderLayer,
   trace::TraceLayer,
};
use webauthn_rs::prelude::{
   DiscoverableKey,
   PublicKeyCredential,
};

use crate::{
   AppState,
   abuse,
   api,
   auth::{
      self,
      SESSION_COOKIE_NAME,
      SESSION_LIFETIME_DAYS,
   },
   config::Config,
   db,
   error::{
      ApiError,
      ApiResult,
   },
   flows::{
      self,
      EmailChangeError,
      MailboxVerifyError,
      PasswordResetError,
   },
   mailer,
   web,
   webauthn as passkey_flow,
};

/// Builds the axum application and serves it until a shutdown signal.
///
/// # Errors
///
/// Returns an error if the database pool, mailer, or `WebAuthn` context fail to
/// initialize, if the listener cannot bind, or if the server exits abnormally.
pub async fn serve(cfg: Config) -> Result<()> {
   let pool = db::build_pool(&cfg.database_url)?;
   let _client = pool.get().await.context("acquiring db client")?;

   let mailer: Arc<dyn mailer::Mailer> = if cfg.smtp_password_file.is_some() {
      Arc::new(mailer::SmtpMailer::from_config(&cfg)?)
   } else {
      tracing::warn!("RAMPART_SMTP_PASSWORD_FILE not set — using in-memory mailer (dev mode)");
      Arc::new(mailer::MemoryMailer::new())
   };

   let webauthn = Arc::new(passkey_flow::build(&cfg)?);

   let state = AppState {
      pool,
      config: Arc::new(cfg),
      verify_cache: Arc::new(auth::VerifyCache::new()),
      mailer,
      webauthn,
   };

   let authed = Router::new()
      .merge(api::router())
      .merge(web::router())
      .layer(middleware::from_fn_with_state(
         state.clone(),
         auth::origin_layer,
      ))
      .layer(middleware::from_fn_with_state(
         state.clone(),
         auth::auth_layer,
      ));

   // Routes whose URL carries a 24-byte random one-shot token already
   // have CSRF protection built in (attacker can't construct the URL
   // without the token, and the token only reaches the email recipient).
   // The Origin layer is redundant here and falsely-rejects browsers
   // that strip Origin/Referer.
   let public_form_token = Router::new()
      .route(
         "/signup/{token}",
         routing::get(signup_page).post(signup_post),
      )
      .route(
         "/auth/reset/{token}",
         routing::get(reset_page).post(reset_post),
      )
      .route(
         "/auth/change-email/{token}",
         routing::get(change_email_page).post(change_email_post),
      )
      .route(
         "/mailbox/verify/{token}",
         routing::get(mailbox_verify_page).post(mailbox_verify_post),
      );

   // No URL token to lean on — keep the Origin/Referer gate.
   let public_form_origin = Router::new()
      .route("/login", routing::get(login_page).post(login_post))
      .route("/logout", routing::post(logout_post))
      .route("/auth/forgot", routing::get(forgot_page).post(forgot_post))
      .route(
         "/api/v1/auth/passkey/start",
         routing::post(passkey_auth_start),
      )
      .route(
         "/api/v1/auth/passkey/finish",
         routing::post(passkey_auth_finish),
      )
      .layer(middleware::from_fn_with_state(
         state.clone(),
         auth::public_form_origin_layer,
      ));

   // /setup uses its own double-submit-cookie CSRF defense.
   let static_files = ServiceBuilder::new()
      .layer(SetResponseHeaderLayer::overriding(
         header::CACHE_CONTROL,
         HeaderValue::from_static("no-cache"),
      ))
      .service(ServeDir::new(state.config.static_dir.clone()));

   let public = Router::new()
      .route("/healthz", routing::get(|| async { "ok" }))
      .route("/setup", routing::get(setup_page).post(setup_post))
      .merge(public_form_token)
      .merge(public_form_origin)
      .nest_service("/static", static_files);

   // No CompressionLayer: nginx in front handles gzip; double-gzip would
   // just burn CPU.
   let app = public
      .merge(authed)
      .layer(middleware::from_fn(security_headers_layer))
      .layer(TraceLayer::new_for_http())
      .with_state(state.clone());

   let listener = TcpListener::bind(state.config.listen)
      .await
      .with_context(|| format!("binding {}", state.config.listen))?;

   tracing::info!(addr = %state.config.listen, "rampart listening");
   let _ = sd_notify::notify(&[sd_notify::NotifyState::Ready]);

   axum::serve(
      listener,
      app.into_make_service_with_connect_info::<SocketAddr>(),
   )
   .with_graceful_shutdown(shutdown_signal())
   .await
   .context("axum serve")?;
   Ok(())
}

async fn shutdown_signal() {
   let ctrl_c = async {
      let _ = signal::ctrl_c().await;
   };
   #[cfg(unix)]
   let term = async {
      use tokio::signal::unix::{
         self,
         SignalKind,
      };
      let mut sigterm = unix::signal(SignalKind::terminate()).expect("install SIGTERM");
      sigterm.recv().await;
   };
   #[cfg(not(unix))]
   let term = std::future::pending::<()>();

   tokio::select! { () = ctrl_c => {}, () = term => {} }
   tracing::info!("shutting down");
   let _ = sd_notify::notify(&[sd_notify::NotifyState::Stopping]);
}

#[derive(Default, Deserialize)]
struct LoginQuery {
   #[serde(default)]
   next:  String,
   #[serde(default)]
   reset: bool,
}

fn login_destination(next: &str) -> &str {
   if next.starts_with('/')
      && !next.starts_with("//")
      && !next.contains('\\')
      && !next.chars().any(char::is_control)
   {
      next
   } else {
      "/"
   }
}

async fn login_page(State(state): State<AppState>, Query(query): Query<LoginQuery>) -> Response {
   use askama::Template;
   #[derive(Template)]
   #[template(path = "login.html")]
   struct LoginPage<'a> {
      error:          Option<&'a str>,
      next:           &'a str,
      password_reset: bool,
      email:          &'a str,
      focus_password: bool,
   }
   // First-run UX: if no user has been created yet, bounce the operator
   // straight to /setup. Removes the need for any documented "first run
   // CLI step" — the deploy URL is the only thing they need to hit.
   if user_table_empty(&state).await.unwrap_or(false) {
      return Redirect::to("/setup").into_response();
   }
   match (LoginPage {
      error:          None,
      next:           login_destination(&query.next),
      password_reset: query.reset,
      email:          "",
      focus_password: false,
   })
   .render()
   {
      Ok(body) => (StatusCode::OK, Html(body)).into_response(),
      Err(err) => ApiError::Template(err).into_response(),
   }
}

async fn user_table_empty(state: &AppState) -> ApiResult<bool> {
   let client = state.pool.get().await?;
   Ok(!users::any_exists().bind(&client).one().await?)
}

// Double-submit-cookie CSRF for /setup. GET emits a random token in both
// the form (hidden field) and a same-site cookie; POST requires both to
// match. A cross-origin attacker can't read the cookie (SameSite=Strict
// keeps it out of cross-site POSTs anyway, and HttpOnly hides it from
// scripts), so they can't forge a matching form field.
const SETUP_CSRF_COOKIE: &str = "rampart_setup_csrf";

fn new_setup_csrf_token() -> String {
   use rand::TryRng as _;
   let mut bytes = [0_u8; 24];
   SysRng
      .try_fill_bytes(&mut bytes)
      .expect("SysRng must not fail");
   BASE64URL_NOPAD.encode(&bytes)
}

fn build_setup_csrf_cookie(state: &AppState, token: &str) -> String {
   let secure_attr = if state.config.public_origin.starts_with("https://") {
      "; Secure"
   } else {
      ""
   };
   format!(
      "{SETUP_CSRF_COOKIE}={token}; Path=/setup; HttpOnly; SameSite=Strict{secure_attr}; \
       Max-Age=900"
   )
}

fn extract_setup_csrf_cookie(headers: &HeaderMap) -> Option<String> {
   let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
   for piece in cookie_header.split(';') {
      let (key, value) = piece.trim().split_once('=')?;
      if key == SETUP_CSRF_COOKIE {
         return Some(value.to_owned());
      }
   }
   None
}

async fn setup_page(State(state): State<AppState>) -> Response {
   // 404 once a user exists — the route is one-shot. We don't redirect
   // to /login here on purpose; an existing-system enumerator probing
   // /setup gets the same response as any unknown path.
   match user_table_empty(&state).await {
      Ok(true) => {},
      Ok(false) => return (StatusCode::NOT_FOUND, "404").into_response(),
      Err(error) => return error.into_response(),
   }
   render_setup_page(&state, StatusCode::OK, None, "", "", false)
}

fn render_setup_page(
   state: &AppState,
   status: StatusCode,
   error: Option<&str>,
   email: &str,
   display_name: &str,
   focus_password: bool,
) -> Response {
   use askama::Template;
   #[derive(Template)]
   #[template(path = "setup.html")]
   struct SetupPage<'a> {
      error:          Option<&'a str>,
      csrf_token:     &'a str,
      email:          &'a str,
      display_name:   &'a str,
      focus_password: bool,
   }
   let token = new_setup_csrf_token();
   let body = match (SetupPage {
      error,
      csrf_token: &token,
      email,
      display_name,
      focus_password,
   })
   .render()
   {
      Ok(rendered) => rendered,
      Err(err) => return ApiError::Template(err).into_response(),
   };
   let mut resp = (status, Html(body)).into_response();
   resp.headers_mut().insert(
      header::SET_COOKIE,
      build_setup_csrf_cookie(state, &token).parse().unwrap(),
   );
   resp
}

#[derive(Deserialize)]
struct SetupForm {
   email:        String,
   password:     String,
   #[serde(default)]
   display_name: Option<String>,
   csrf_token:   String,
}

#[expect(
   clippy::cognitive_complexity,
   reason = "linear CSRF-check then account-bootstrap flow reads clearer inline than split"
)]
async fn setup_post(
   State(state): State<AppState>,
   headers: HeaderMap,
   Form(form): Form<SetupForm>,
) -> Response {
   let Some(cookie_token) = extract_setup_csrf_cookie(&headers) else {
      tracing::warn!("setup_post: missing CSRF cookie");
      return render_setup_page(
         &state,
         StatusCode::FORBIDDEN,
         Some("This setup page expired. Review the details and try again."),
         &form.email,
         form.display_name.as_deref().unwrap_or(""),
         true,
      );
   };
   if !constant_time_eq::constant_time_eq(cookie_token.as_bytes(), form.csrf_token.as_bytes()) {
      tracing::warn!("setup_post: CSRF token mismatch");
      return render_setup_page(
         &state,
         StatusCode::FORBIDDEN,
         Some("This setup page expired. Review the details and try again."),
         &form.email,
         form.display_name.as_deref().unwrap_or(""),
         true,
      );
   }

   // The flow function's `INSERT ... WHERE NOT EXISTS` guard is the
   // authoritative gate, so even a race between two simultaneous POSTs
   // resolves correctly. The page-level check is just for a nicer
   // error message — Ok(None) on race-lost still falls through to 404.
   let user_id = match flows::bootstrap_first_admin(
      &state.pool,
      &form.email,
      &form.password,
      form.display_name.as_deref(),
   )
   .await
   {
      Ok(Some(id)) => id,
      Ok(None) => return (StatusCode::NOT_FOUND, "404").into_response(),
      Err(err) => {
         tracing::warn!(error = %err, "setup_post failed");
         let (status, message) = if err.to_string().contains("password must be") {
            (
               StatusCode::BAD_REQUEST,
               "Password must be at least 10 characters.",
            )
         } else {
            (
               StatusCode::INTERNAL_SERVER_ERROR,
               "Couldn’t create the admin account. Try again.",
            )
         };
         return render_setup_page(
            &state,
            status,
            Some(message),
            &form.email,
            form.display_name.as_deref().unwrap_or(""),
            true,
         );
      },
   };

   // Sign the new admin in immediately so they land on the dashboard.
   let session_id = auth::new_session_id();
   let expiry = OffsetDateTime::now_utc() + Duration::hours(24 * SESSION_LIFETIME_DAYS);
   let user_agent = headers
      .get(header::USER_AGENT)
      .and_then(|value| value.to_str().ok())
      .map(str::to_owned);
   let client = match state.pool.get().await {
      Ok(client) => client,
      Err(err) => return ApiError::Pool(err).into_response(),
   };
   if let Err(err) = sessions::create()
      .bind(
         &client,
         &session_id.to_vec(),
         &user_id,
         &expiry,
         &user_agent,
      )
      .await
   {
      return ApiError::Db(err).into_response();
   }

   let cookie = build_session_cookie(&state, &session_id);
   let mut resp = Redirect::to("/").into_response();
   resp
      .headers_mut()
      .insert(header::SET_COOKIE, cookie.parse().unwrap());
   resp
}

#[derive(Deserialize)]
struct LoginForm {
   email:    String,
   password: String,
   #[serde(default)]
   next:     String,
}

async fn login_post(
   State(state): State<AppState>,
   ConnectInfo(remote): ConnectInfo<SocketAddr>,
   headers: HeaderMap,
   Form(form): Form<LoginForm>,
) -> Response {
   // Rate limit per IP and per email — whichever hits the cap first blocks.
   let ip_key = format!("login_fail:ip:{}", remote.ip());
   let email_key = format!("login_fail:email:{}", form.email.to_lowercase());
   for key in [&ip_key, &email_key] {
      match abuse::check(&state.pool, key, abuse::LOGIN_FAIL).await {
         Ok(false) => {
            return (
               StatusCode::TOO_MANY_REQUESTS,
               login_page_with_error(
                  "Too many sign-in attempts. Try again later.",
                  &form.next,
                  &form.email,
               ),
            )
               .into_response();
         },
         Ok(true) => {},
         Err(err) => return ApiError::Internal(err).into_response(),
      }
   }

   let creds = BASE64.encode(format!("{}:{}", form.email, form.password).as_bytes());
   let mut auth_headers = HeaderMap::new();
   auth_headers.insert(
      header::AUTHORIZATION,
      format!("Basic {creds}").parse().unwrap(),
   );
   let principal = match auth::extract_principal(&state, &auth_headers).await {
      Ok(Some(principal)) => principal,
      Ok(None) => {
         return (
            StatusCode::UNAUTHORIZED,
            login_page_with_error("Email or password is incorrect.", &form.next, &form.email),
         )
            .into_response();
      },
      Err(err) => return err.into_response(),
   };

   let _ = abuse::clear(&state.pool, &ip_key).await;
   let _ = abuse::clear(&state.pool, &email_key).await;

   let session_id = auth::new_session_id();
   let expiry = OffsetDateTime::now_utc() + Duration::hours(24 * SESSION_LIFETIME_DAYS);
   let user_agent = headers
      .get(header::USER_AGENT)
      .and_then(|value| value.to_str().ok())
      .map(str::to_owned);

   let client = match state.pool.get().await {
      Ok(client) => client,
      Err(err) => return ApiError::Pool(err).into_response(),
   };
   if let Err(err) = sessions::create()
      .bind(
         &client,
         &session_id.to_vec(),
         &principal.user_id,
         &expiry,
         &user_agent,
      )
      .await
   {
      return ApiError::Db(err).into_response();
   }

   let cookie = build_session_cookie(&state, &session_id);
   let mut resp = Redirect::to(login_destination(&form.next)).into_response();
   resp
      .headers_mut()
      .insert(header::SET_COOKIE, cookie.parse().unwrap());
   resp
}

/// `Secure` only emitted under HTTPS — browsers drop `Secure` cookies on
/// plain HTTP, so a tailscale-only HTTP deploy would silently break login.
fn build_session_cookie(state: &AppState, session_id: &[u8]) -> String {
   let value = auth::session_cookie_value(session_id);
   let secure_attr = if state.config.public_origin.starts_with("https://") {
      "; Secure"
   } else {
      ""
   };
   format!(
      "{SESSION_COOKIE_NAME}={value}; Path=/; HttpOnly; SameSite=Lax{secure_attr}; Max-Age={}",
      SESSION_LIFETIME_DAYS * 86400
   )
}

fn build_clear_session_cookie(state: &AppState) -> String {
   let secure_attr = if state.config.public_origin.starts_with("https://") {
      "; Secure"
   } else {
      ""
   };
   format!("{SESSION_COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Lax{secure_attr}; Max-Age=0")
}

fn login_page_with_error(msg: &str, next: &str, email: &str) -> Response {
   use askama::Template;
   #[derive(Template)]
   #[template(path = "login.html")]
   struct LoginPage<'a> {
      error:          Option<&'a str>,
      next:           &'a str,
      password_reset: bool,
      email:          &'a str,
      focus_password: bool,
   }
   match (LoginPage {
      error: Some(msg),
      next: login_destination(next),
      password_reset: false,
      email,
      focus_password: true,
   })
   .render()
   {
      Ok(body) => Html(body).into_response(),
      Err(err) => ApiError::Template(err).into_response(),
   }
}

async fn logout_post(State(state): State<AppState>, headers: HeaderMap) -> Response {
   // /logout lives on the public router so origin_layer doesn't cover it.
   let origin = headers
      .get(header::ORIGIN)
      .and_then(|value| value.to_str().ok());
   match origin {
      Some(value) if value == state.config.public_origin => {},
      _ => {
         return (StatusCode::FORBIDDEN, "403 cross-origin").into_response();
      },
   }
   if let Some(sid) = extract_session_id_from_headers(&headers) {
      let client = match state.pool.get().await {
         Ok(client) => client,
         Err(err) => return ApiError::Pool(err).into_response(),
      };
      let _ = sessions::delete_by_id().bind(&client, &sid).await;
   }
   let clear_cookie = build_clear_session_cookie(&state);
   let mut resp = Redirect::to("/login").into_response();
   resp
      .headers_mut()
      .insert(header::SET_COOKIE, clear_cookie.parse().unwrap());
   resp
}

fn extract_session_id_from_headers(headers: &HeaderMap) -> Option<Vec<u8>> {
   let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
   for piece in cookie_header.split(';') {
      let (key, value) = piece.trim().split_once('=')?;
      if key == SESSION_COOKIE_NAME {
         return BASE64URL_NOPAD.decode(value.as_bytes()).ok();
      }
   }
   None
}

async fn signup_page(Path(token): Path<String>) -> Response {
   render_signup_page(StatusCode::OK, &token, None, "", "")
}

fn render_signup_page(
   status: StatusCode,
   token: &str,
   error: Option<&str>,
   email: &str,
   display_name: &str,
) -> Response {
   use askama::Template;
   #[derive(Template)]
   #[template(path = "signup.html")]
   struct SignupPage<'a> {
      token:        &'a str,
      error:        Option<&'a str>,
      email:        &'a str,
      display_name: &'a str,
   }
   match (SignupPage {
      token,
      error,
      email,
      display_name,
   })
   .render()
   {
      Ok(body) => (status, Html(body)).into_response(),
      Err(err) => ApiError::Template(err).into_response(),
   }
}

#[derive(Deserialize)]
struct SignupForm {
   email:        String,
   password:     String,
   #[serde(default)]
   display_name: Option<String>,
}

async fn signup_post(
   State(state): State<AppState>,
   Path(token): Path<String>,
   headers: HeaderMap,
   Form(form): Form<SignupForm>,
) -> Response {
   match signup_inner(&state, &token, &form, &headers).await {
      Ok(resp) => resp,
      Err(ApiError::BadRequest(message)) => render_signup_page(
         StatusCode::BAD_REQUEST,
         &token,
         Some(&message),
         &form.email,
         form.display_name.as_deref().unwrap_or(""),
      ),
      Err(ApiError::Conflict(message)) => render_signup_page(
         StatusCode::CONFLICT,
         &token,
         Some(&message),
         &form.email,
         form.display_name.as_deref().unwrap_or(""),
      ),
      Err(err) => err.into_response(),
   }
}

fn signup_api_error(error: flows::InviteSignupError) -> ApiError {
   use crate::flows::InviteSignupError;
   match error {
      InviteSignupError::PasswordTooShort => {
         ApiError::BadRequest("Password must be at least 10 characters.".into())
      },
      InviteSignupError::Invalid => ApiError::BadRequest("This invitation isn’t valid.".into()),
      InviteSignupError::Expired => ApiError::BadRequest(
         "This invitation has expired. Ask an administrator for a new one.".into(),
      ),
      InviteSignupError::AlreadyUsed => {
         ApiError::BadRequest("This invitation has already been used.".into())
      },
      InviteSignupError::EmailMismatch => {
         ApiError::BadRequest("This invitation is tied to a different email address.".into())
      },
      InviteSignupError::AlreadyRegistered => {
         ApiError::Conflict("An account already exists for this email.".into())
      },
      InviteSignupError::Internal(error) => ApiError::Internal(error),
   }
}

async fn signup_inner(
   state: &AppState,
   token: &str,
   form: &SignupForm,
   headers: &HeaderMap,
) -> ApiResult<Response> {
   let (user_id, _is_admin) = flows::claim_invite_and_create_user(
      &state.pool,
      token,
      &form.email,
      &form.password,
      form.display_name.as_deref(),
   )
   .await
   .map_err(signup_api_error)?;

   let session_id = auth::new_session_id();
   let expiry = OffsetDateTime::now_utc() + Duration::hours(24 * SESSION_LIFETIME_DAYS);
   let user_agent = headers
      .get(header::USER_AGENT)
      .and_then(|value| value.to_str().ok())
      .map(str::to_owned);
   let client = state.pool.get().await?;
   sessions::create()
      .bind(
         &client,
         &session_id.to_vec(),
         &user_id,
         &expiry,
         &user_agent,
      )
      .await?;

   let cookie = build_session_cookie(state, &session_id);
   let mut resp = Redirect::to("/").into_response();
   resp
      .headers_mut()
      .insert(header::SET_COOKIE, cookie.parse().unwrap());
   Ok(resp)
}

async fn forgot_page() -> Response {
   render_forgot_page(StatusCode::OK, false, None, "")
}

fn render_forgot_page(
   status: StatusCode,
   sent: bool,
   error: Option<&str>,
   email: &str,
) -> Response {
   use askama::Template;
   #[derive(Template)]
   #[template(path = "forgot.html")]
   struct ForgotPage<'a> {
      sent:  bool,
      error: Option<&'a str>,
      email: &'a str,
   }
   match (ForgotPage { sent, error, email }).render() {
      Ok(body) => (status, Html(body)).into_response(),
      Err(err) => ApiError::Template(err).into_response(),
   }
}

#[derive(Deserialize)]
struct ForgotForm {
   email: String,
}

async fn forgot_post(
   State(state): State<AppState>,
   ConnectInfo(remote): ConnectInfo<SocketAddr>,
   Form(form): Form<ForgotForm>,
) -> Response {
   // IP cap defends against enumeration across many distinct email inputs.
   for key in [
      format!("forgot:email:{}", form.email.to_lowercase()),
      format!("forgot:ip:{}", remote.ip()),
   ] {
      match abuse::check(&state.pool, &key, abuse::FORGOT_PASSWORD).await {
         Ok(true) => {},
         Ok(false) => {
            return render_forgot_page(
               StatusCode::TOO_MANY_REQUESTS,
               false,
               Some("Too many reset requests. Try again later."),
               &form.email,
            );
         },
         Err(error) => return ApiError::Internal(error).into_response(),
      }
   }
   if let Err(error) = flows::start_password_reset(
      &state.pool,
      state.mailer.as_ref(),
      &state.config.public_origin,
      &form.email,
   )
   .await
   {
      tracing::error!(error = %error, "password reset request failed");
   }
   render_forgot_page(StatusCode::OK, true, None, "")
}

async fn reset_page(Path(token): Path<String>) -> Response {
   render_reset_page(StatusCode::OK, &token, None)
}

fn render_reset_page(status: StatusCode, token: &str, error: Option<&str>) -> Response {
   use askama::Template;
   #[derive(Template)]
   #[template(path = "reset.html")]
   struct ResetPage<'a> {
      token: &'a str,
      error: Option<&'a str>,
   }
   match (ResetPage { token, error }).render() {
      Ok(body) => (status, Html(body)).into_response(),
      Err(err) => ApiError::Template(err).into_response(),
   }
}

#[derive(Deserialize)]
struct ResetForm {
   password: String,
}

async fn reset_post(
   State(state): State<AppState>,
   ConnectInfo(remote): ConnectInfo<SocketAddr>,
   Path(token): Path<String>,
   Form(form): Form<ResetForm>,
) -> Response {
   // 20/hour/IP — flattens automated enumeration without hitting typo retries.
   let key = format!("reset_apply:ip:{}", remote.ip());
   if matches!(
      abuse::check(&state.pool, &key, abuse::RESET_APPLY).await,
      Ok(false)
   ) {
      return render_reset_page(
         StatusCode::TOO_MANY_REQUESTS,
         &token,
         Some("Too many reset attempts. Try again later."),
      );
   }
   match flows::apply_password_reset(&state.pool, &state.verify_cache, &token, &form.password).await
   {
      Ok(()) => Redirect::to("/login?reset=true").into_response(),
      Err(PasswordResetError::PasswordTooShort) => render_reset_page(
         StatusCode::BAD_REQUEST,
         &token,
         Some("Password must be at least 10 characters."),
      ),
      Err(PasswordResetError::Invalid) => render_simple_message(
         StatusCode::BAD_REQUEST,
         "Reset link isn’t valid",
         "Check that you opened the complete link from your password reset email.",
         true,
         "/auth/forgot",
         "Request a new reset link",
      ),
      Err(PasswordResetError::Expired) => render_simple_message(
         StatusCode::GONE,
         "Reset link expired",
         "This password reset link has expired. Request a new one to continue.",
         true,
         "/auth/forgot",
         "Request a new reset link",
      ),
      Err(PasswordResetError::AlreadyUsed) => render_simple_message(
         StatusCode::GONE,
         "Reset link already used",
         "This password reset link has already been used. Request a new one if you still need to \
          change your password.",
         true,
         "/auth/forgot",
         "Request a new reset link",
      ),
      Err(PasswordResetError::Internal(error)) => ApiError::Internal(error).into_response(),
   }
}

async fn change_email_page(Path(token): Path<String>) -> Response {
   use askama::Template;
   #[derive(Template)]
   #[template(path = "confirm.html")]
   struct ConfirmPage<'a> {
      title:         &'a str,
      body:          &'a str,
      action:        &'a str,
      button_label:  &'a str,
      pending_label: &'a str,
   }
   render_or_err(
      (ConfirmPage {
         title:         "Confirm email change",
         body:          "Change your rampart account email to the address this message was sent \
                         to.",
         action:        &format!("/auth/change-email/{token}"),
         button_label:  "Change email",
         pending_label: "Changing email…",
      })
      .render(),
   )
}

async fn change_email_post(State(state): State<AppState>, Path(token): Path<String>) -> Response {
   match flows::apply_email_change(&state.pool, &token).await {
      Ok(email) => render_simple_message(
         StatusCode::OK,
         "Email changed",
         &format!("Your rampart sign-in email is now {email}."),
         true,
         "/settings",
         "Go to settings",
      ),
      Err(EmailChangeError::Invalid) => render_simple_message(
         StatusCode::BAD_REQUEST,
         "Email change link isn’t valid",
         "Check that you opened the complete link from your email.",
         true,
         "/settings",
         "Go to settings",
      ),
      Err(EmailChangeError::Expired) => render_simple_message(
         StatusCode::GONE,
         "Email change link expired",
         "This email change link has expired. Start the change again from settings.",
         true,
         "/settings",
         "Go to settings",
      ),
      Err(EmailChangeError::AlreadyUsed) => render_simple_message(
         StatusCode::GONE,
         "Email change link already used",
         "This email change link has already been used. Check your current address in settings.",
         true,
         "/settings",
         "Go to settings",
      ),
      Err(EmailChangeError::AlreadyRegistered) => render_simple_message(
         StatusCode::CONFLICT,
         "Email already in use",
         "Another account already uses this email address. Choose a different address in settings.",
         true,
         "/settings",
         "Go to settings",
      ),
      Err(EmailChangeError::Internal(error)) => ApiError::Internal(error).into_response(),
   }
}

async fn mailbox_verify_page(Path(token): Path<String>) -> Response {
   use askama::Template;
   #[derive(Template)]
   #[template(path = "confirm.html")]
   struct ConfirmPage<'a> {
      title:         &'a str,
      body:          &'a str,
      action:        &'a str,
      button_label:  &'a str,
      pending_label: &'a str,
   }
   render_or_err(
      (ConfirmPage {
         title:         "Verify mailbox",
         body:          "Confirm that you own the mailbox this message was sent to.",
         action:        &format!("/mailbox/verify/{token}"),
         button_label:  "Verify mailbox",
         pending_label: "Verifying mailbox…",
      })
      .render(),
   )
}

async fn mailbox_verify_post(State(state): State<AppState>, Path(token): Path<String>) -> Response {
   match flows::apply_mailbox_verify(&state.pool, &token).await {
      Ok(_id) => render_simple_message(
         StatusCode::OK,
         "Mailbox verified",
         "This mailbox is ready to use with rampart.",
         true,
         "/mailboxes",
         "Go to mailboxes",
      ),
      Err(MailboxVerifyError::Invalid) => render_simple_message(
         StatusCode::BAD_REQUEST,
         "Verification link isn’t valid",
         "Check that you opened the complete link from your email.",
         true,
         "/mailboxes",
         "Go to mailboxes",
      ),
      Err(MailboxVerifyError::Expired) => render_simple_message(
         StatusCode::GONE,
         "Verification link expired",
         "This mailbox verification link has expired. Send a new one from mailboxes.",
         true,
         "/mailboxes",
         "Go to mailboxes",
      ),
      Err(MailboxVerifyError::AlreadyUsed) => render_simple_message(
         StatusCode::GONE,
         "Verification link already used",
         "This link has already been used. Check the mailbox status in rampart.",
         true,
         "/mailboxes",
         "Go to mailboxes",
      ),
      Err(MailboxVerifyError::Internal(error)) => ApiError::Internal(error).into_response(),
   }
}

fn render_or_err(rendered: Result<String, askama::Error>) -> Response {
   match rendered {
      Ok(body) => Html(body).into_response(),
      Err(err) => ApiError::Template(err).into_response(),
   }
}

#[derive(askama::Template)]
#[template(path = "simple_message.html")]
struct SimpleMessage<'a> {
   heading:    &'a str,
   message:    &'a str,
   show_link:  bool,
   link_href:  &'a str,
   link_label: &'a str,
}

fn render_simple_message(
   status: StatusCode,
   heading: &str,
   message: &str,
   show_link: bool,
   link_href: &str,
   link_label: &str,
) -> Response {
   use askama::Template as _;
   match (SimpleMessage {
      heading,
      message,
      show_link,
      link_href,
      link_label,
   })
   .render()
   {
      Ok(body) => (status, Html(body)).into_response(),
      Err(err) => ApiError::Template(err).into_response(),
   }
}

#[derive(Deserialize)]
struct PasskeyStartReq {
   email: String,
}

#[derive(serde::Serialize)]
struct PasskeyStartResp {
   ceremony_id:  String,
   challenge:    serde_json::Value,
   discoverable: bool,
}

async fn passkey_auth_start(
   State(state): State<AppState>,
   ConnectInfo(remote): ConnectInfo<SocketAddr>,
   Json(body): Json<PasskeyStartReq>,
) -> Response {
   // Blunt account-presence enumeration via start responses.
   let key = format!("passkey_start:ip:{}", remote.ip());
   if matches!(
      abuse::check(&state.pool, &key, abuse::LOGIN_FAIL).await,
      Ok(false)
   ) {
      return (StatusCode::TOO_MANY_REQUESTS, "slow down").into_response();
   }
   passkey_auth_start_inner(&state, body.email)
      .await
      .map_or_else(
         // Uniform error — don't leak whether the user exists / has passkeys.
         |_| (StatusCode::BAD_REQUEST, "passkey auth unavailable").into_response(),
         |resp| Json(resp).into_response(),
      )
}

async fn passkey_auth_start_inner(
   state: &AppState,
   email: String,
) -> anyhow::Result<PasskeyStartResp> {
   if email.trim().is_empty() {
      let (challenge, auth_state) = state.webauthn.start_discoverable_authentication()?;
      let id =
         passkey_flow::save_discoverable_authentication_state(&state.pool, &auth_state).await?;
      return Ok(PasskeyStartResp {
         ceremony_id:  hex::encode(&id),
         challenge:    serde_json::to_value(&challenge)?,
         discoverable: true,
      });
   }

   let (_user_id, passkeys) = passkey_flow::load_passkeys_for_email(&state.pool, &email).await?;
   let (challenge, auth_state) = state.webauthn.start_passkey_authentication(&passkeys)?;
   let id = passkey_flow::save_authentication_state(&state.pool, None, &auth_state).await?;
   Ok(PasskeyStartResp {
      ceremony_id:  hex::encode(&id),
      challenge:    serde_json::to_value(&challenge)?,
      discoverable: false,
   })
}

#[derive(Deserialize)]
struct PasskeyFinishReq {
   ceremony_id:  String,
   credential:   PublicKeyCredential,
   #[serde(default)]
   discoverable: bool,
}

async fn passkey_auth_finish(
   State(state): State<AppState>,
   headers: HeaderMap,
   Json(body): Json<PasskeyFinishReq>,
) -> Response {
   match passkey_auth_finish_inner(&state, headers, body).await {
      Ok(resp) => resp,
      Err(err) => {
         // Uniform 401 — don't leak which step failed.
         tracing::warn!(error = ?err, "passkey auth finish failed");
         (StatusCode::UNAUTHORIZED, "auth failed").into_response()
      },
   }
}

async fn passkey_auth_finish_inner(
   state: &AppState,
   headers: HeaderMap,
   body: PasskeyFinishReq,
) -> anyhow::Result<Response> {
   let id = hex::decode(&body.ceremony_id)?;
   let (result, user_id) = if body.discoverable {
      let auth_state =
         passkey_flow::load_discoverable_authentication_state(&state.pool, &id).await?;
      let (handle, credential_id) = state
         .webauthn
         .identify_discoverable_authentication(&body.credential)?;
      let credential_id = credential_id.to_vec();
      let client = state.pool.get().await?;
      let user_id = webauthn::credential_user_id()
         .bind(&client, &credential_id)
         .one()
         .await?;
      anyhow::ensure!(handle == passkey_flow::user_handle(user_id));
      let passkeys = passkey_flow::load_passkeys_for_user(&state.pool, user_id).await?;
      let discoverable_keys = passkeys
         .iter()
         .map(DiscoverableKey::from)
         .collect::<Vec<_>>();
      let result = state.webauthn.finish_discoverable_authentication(
         &body.credential,
         auth_state,
         &discoverable_keys,
      )?;
      (result, user_id)
   } else {
      let auth_state = passkey_flow::load_authentication_state(&state.pool, &id).await?;
      let result = state
         .webauthn
         .finish_passkey_authentication(&body.credential, &auth_state)?;
      let credential_id = result.cred_id().as_ref().to_vec();
      let client = state.pool.get().await?;
      let user_id = webauthn::credential_user_id()
         .bind(&client, &credential_id)
         .one()
         .await?;
      (result, user_id)
   };
   let cred_id: &[u8] = result.cred_id().as_ref();
   let client = state.pool.get().await?;
   // webauthn-rs needs the full updated Passkey blob (counter + backup
   // flags) round-tripped, not just sign_count.
   passkey_flow::update_credential_after_auth(&state.pool, cred_id, &result).await?;

   let session_id = auth::new_session_id();
   let expiry = OffsetDateTime::now_utc() + Duration::hours(24 * SESSION_LIFETIME_DAYS);
   let user_agent = headers
      .get(header::USER_AGENT)
      .and_then(|value| value.to_str().ok())
      .map(str::to_owned);
   sessions::create()
      .bind(
         &client,
         &session_id.to_vec(),
         &user_id,
         &expiry,
         &user_agent,
      )
      .await?;

   let cookie = build_session_cookie(state, &session_id);
   let mut resp = (StatusCode::NO_CONTENT, "").into_response();
   resp
      .headers_mut()
      .insert(header::SET_COOKIE, cookie.parse().unwrap());
   Ok(resp)
}

/// Strict CSP (`script-src 'self'`) — see static/app.js + static/webauthn.js
/// for the template refactor that lets this hold. `pub` so tests/headers.rs
/// can layer the production middleware directly.
pub async fn security_headers_layer(req: Request, next: middleware::Next) -> Response {
   let mut resp = next.run(req).await;
   let headers = resp.headers_mut();
   headers.insert(
      "Content-Security-Policy",
      HeaderValue::from_static(
         "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; \
          connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
      ),
   );
   headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
   headers.insert(
      "X-Content-Type-Options",
      HeaderValue::from_static("nosniff"),
   );
   headers.insert(
      "Referrer-Policy",
      HeaderValue::from_static("strict-origin-when-cross-origin"),
   );
   headers.insert(
      "Permissions-Policy",
      HeaderValue::from_static("interest-cohort=()"),
   );
   resp
}

#[cfg(test)]
#[expect(
   clippy::inline_modules,
   reason = "unit tests kept inline with the code they cover"
)]
mod tests {
   use super::{
      login_destination,
      signup_api_error,
   };
   use crate::{
      error::ApiError,
      flows::InviteSignupError,
   };

   #[test]
   fn login_destination_accepts_local_paths() {
      assert_eq!(login_destination("/admin/users"), "/admin/users");
      assert_eq!(login_destination("/aliases/1?page=2"), "/aliases/1?page=2");
   }

   #[test]
   fn login_destination_rejects_external_and_invalid_paths() {
      for next in [
         "",
         "https://example.com",
         "//example.com",
         "/\\example.com",
         "/ok\nset-cookie:x",
      ] {
         assert_eq!(login_destination(next), "/");
      }
   }

   #[test]
   fn signup_errors_are_safe_and_actionable() {
      let cases = [
         (
            InviteSignupError::PasswordTooShort,
            "Password must be at least 10 characters.",
         ),
         (InviteSignupError::Invalid, "This invitation isn’t valid."),
         (
            InviteSignupError::Expired,
            "This invitation has expired. Ask an administrator for a new one.",
         ),
         (
            InviteSignupError::AlreadyUsed,
            "This invitation has already been used.",
         ),
         (
            InviteSignupError::EmailMismatch,
            "This invitation is tied to a different email address.",
         ),
         (
            InviteSignupError::AlreadyRegistered,
            "An account already exists for this email.",
         ),
      ];

      #[expect(
         clippy::panic,
         reason = "test must fail loudly on an unexpected error variant"
      )]
      for (input, expected) in cases {
         match signup_api_error(input) {
            ApiError::BadRequest(message) | ApiError::Conflict(message) => {
               assert_eq!(message, expected);
            },
            error => panic!("unexpected error: {error}"),
         }
      }
   }

   #[test]
   fn signup_internal_errors_stay_internal() {
      assert!(matches!(
         signup_api_error(InviteSignupError::Internal(anyhow::anyhow!(
            "connection closed"
         ))),
         ApiError::Internal(_)
      ));
   }
}
