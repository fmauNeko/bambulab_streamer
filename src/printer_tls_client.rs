use std::error::Error;
use tokio::net::TcpStream;
use tokio_native_tls::{native_tls::TlsConnector, TlsStream};

pub async fn connect(address: &str) -> Result<TlsStream<TcpStream>, Box<dyn Error>> {
  let connector = TlsConnector::builder()
    .danger_accept_invalid_certs(true)
    .danger_accept_invalid_hostnames(true)
    .build()?;
  let connector = tokio_native_tls::TlsConnector::from(connector);

  let stream = TcpStream::connect(format!("{}:6000", address)).await?;
  let stream = connector.connect(address, stream).await?;
  Ok(stream)
}
