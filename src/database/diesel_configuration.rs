use crate::database::DatabaseConnectionPool;
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::AsyncPgConnection;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;

pub fn create_connection_pool(url: &str) -> DatabaseConnectionPool {
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);
    let connection_manager =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            url, config,
        );
    Pool::builder(connection_manager)
        .build()
        .expect("Failed to setup pool")
}

pub fn establish_connection(
    config: &str,
) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let future = async {
        let rustls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_certificates())
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Database Connection: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    };
    future.boxed()
}

fn root_certificates() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs()
        .expect("Certificates not loadable.");
    for cert in certs {
        roots
            .add(cert)
            .expect("Could not load plataform certificate");
    }
    roots
}
