use std::sync::Arc;

use crate::database::DatabaseConnectionPool;
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::AsyncPgConnection;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use rustls::crypto::aws_lc_rs as provider;

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
        let mut rustls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_certificates())
            .with_no_client_auth();
        rustls_config.dangerous().set_certificate_verifier(Arc::new(danger::NoCertificateVerification::new(provider::default_provider())));
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
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let certs = rustls_native_certs::load_native_certs()
        .expect("Certificates not loadable.");
    for cert in certs {
        roots
            .add(cert)
            .expect("Could not load plataform certificate");
    }
    roots
}

//Workaround since rustls is not accepting digital ocean certificate issuer
mod danger {
    use rustls::client::danger::HandshakeSignatureValid;
    use rustls::crypto::{
        verify_tls12_signature, verify_tls13_signature, CryptoProvider,
    };
    use rustls::DigitallySignedStruct;

    #[derive(Debug)]
    pub struct NoCertificateVerification(CryptoProvider);

    impl NoCertificateVerification {
        pub fn new(provider: CryptoProvider) -> Self {
            Self(provider)
        }
    }

    impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::pki_types::CertificateDer<'_>,
            _intermediates: &[rustls::pki_types::CertificateDer<'_>],
            _server_name: &rustls::pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error>
        {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            message: &[u8],
            cert: &rustls::pki_types::CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            verify_tls12_signature(
                message,
                cert,
                dss,
                &self.0.signature_verification_algorithms,
            )
        }

        fn verify_tls13_signature(
            &self,
            message: &[u8],
            cert: &rustls::pki_types::CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            verify_tls13_signature(
                message,
                cert,
                dss,
                &self.0.signature_verification_algorithms,
            )
        }
        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            self.0.signature_verification_algorithms.supported_schemes()
        }
    }
}
