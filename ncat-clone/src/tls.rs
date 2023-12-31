use crate::common::read_write;
use rustls_pemfile;
use std::fs::File;
use std::io;
use std::io::{BufReader, Error};
use std::path::Path;
use std::sync::Arc;
use tokio::io::split;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::{
    Certificate, OwnedTrustAnchor, PrivateKey, RootCertStore, ServerConfig,
};
use tokio_rustls::{rustls::ClientConfig, TlsAcceptor, TlsConnector};
use webpki_roots;

// Created a secured connection leveraging TLS
pub async fn tls_connect(host: &String, port: &u16, ca: &Option<String>) -> Result<(), Error> {
    let addr = format!("{}:{}", host, port);

    // List of all certificates that Mozilla Web Browser trusts
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    /*
    Since we're adding CA (Certificate Authority) locally, we need to add our certificate to the store
    And when we trying to connect to ourselves while running ncat-clone server, we don't have issues with CAs are being trusted

    We can comment this snippet to make the connection to the server denied since the client won't trust the server's certificate.
    */
    if let Some(ca) = ca {
        for certificate in load_certs(Path::new(&ca))? {
            root_cert_store
                .add(&certificate)
                .map_err(|e| format!("Error adding the certificate: {}", e))
                .unwrap();
        }
    }

    // Sets up TLS config
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth(); // we don't wanna authenticate with servers we connect to

    let tls_connector = TlsConnector::from(Arc::new(config));

    let server_name = host.as_str().try_into().unwrap();

    // TCP socket connection and wrapping the socket with TLS connector
    let stream = TcpStream::connect(&addr).await?;
    let stream = tls_connector.connect(server_name, stream).await?;

    let (reader, writer) = split(stream);
    read_write(reader, writer).await;

    Ok(())
}

pub async fn tls_listen(
    host: &String,
    port: &u16,
    ca: &Option<String>,
    cert: String,
    key: String,
) -> Result<(), Error> {
    let addr = format!("{}:{}", host, port);

    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    if let Some(ca) = ca {
        for cert in load_certs(Path::new(ca.as_str()))? {
            root_cert_store
                .add(&cert)
                .map_err(|_e| Error::new(io::ErrorKind::InvalidInput, "could not add CA"))?;
        }
    }

    // We need to provide authentication to the client to tell him who we are
    let certs = load_certs(Path::new(cert.as_str()))?;
    let mut keys = load_keys(Path::new(key.as_str()))?;
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0)) // takes the first key out
        .map_err(|err| Error::new(io::ErrorKind::InvalidInput, err))?;

    // accepts a connection
    let acceptor = TlsAcceptor::from(Arc::new(config));

    // binds to an address and waits for connections to it
    let listener = TcpListener::bind(&addr).await?;

    // accepts the first connection we receved
    let (socket, _) = listener.accept().await?;

    // wraps up the received socker from above and with the TLS acceptor to add TLS messaging
    let stream = acceptor.accept(socket).await?;

    let (reader, writer) = split(stream);

    read_write(reader, writer).await;

    Ok(())
}

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    let f = File::open(path)?;

    return rustls_pemfile::certs(&mut BufReader::new(f))
        .map_err(|_| Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect());
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    let f = File::open(path)?;

    rustls_pemfile::rsa_private_keys(&mut BufReader::new(f))
        .map_err(|_| Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}