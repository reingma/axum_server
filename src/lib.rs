pub mod configuration;
pub mod database;
pub mod domain;
pub mod email_client;
pub mod errors;
pub mod models;
pub mod routes;
pub mod schema;
pub mod startup;
pub mod telemetry;

use once_cell::sync::Lazy;
use tera::Tera;

pub static TEMPLATES: Lazy<Tera> =
    Lazy::new(|| match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Tera parsing error(s): {}", e);
            panic!("Failed at loading tera templates");
        }
    });
