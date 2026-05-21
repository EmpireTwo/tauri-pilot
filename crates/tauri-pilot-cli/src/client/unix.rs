use super::Client;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::io::BufReader;
use tokio::net::UnixStream;

pub async fn connect(path: &Path) -> Result<Client> {
    let stream = UnixStream::connect(path)
        .await
        .with_context(|| format!("Cannot connect to socket: {}", path.display()))?;
    let (reader, writer) = stream.into_split();
    Ok(Client {
        tcp_reader: None,
        tcp_writer: None,
        reader: Some(BufReader::new(reader)),
        writer: Some(writer),
        next_id: 1,
    })
}
