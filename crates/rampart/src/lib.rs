//! rampart library crate. Entry points are `serve::serve`, `worker::run`,
//! `migrate::run`, and the `admin::*` subcommands; the binary in
//! `src/main.rs` is a thin pound dispatcher.

mod abuse;
mod api;
mod domain_setup;
mod error;
mod quota;
mod sieve;
mod template_filters;
mod web;
mod webauthn;

pub mod admin;
pub mod auth;
pub mod bootstrap;
pub mod config;
pub mod db;
pub mod flows;
pub mod mailer;
pub mod migrate;
pub mod preview;
pub mod serve;
pub mod worker;

use std::sync::Arc;

use deadpool_postgres::Pool;

#[derive(Clone)]
pub struct AppState {
   pub pool:         Pool,
   pub config:       Arc<config::Config>,
   pub verify_cache: Arc<auth::VerifyCache>,
   pub mailer:       Arc<dyn mailer::Mailer>,
   pub webauthn:     Arc<webauthn_rs::Webauthn>,
}
