use diesel_async::{pooled_connection::deadpool::Object, AsyncPgConnection};

pub mod configuration;
pub mod models;
pub mod routes;
pub mod schema;
pub mod startup;
pub mod telemetry;

// type re-exports:
pub type DatabaseConnection = Object<AsyncPgConnection>;
