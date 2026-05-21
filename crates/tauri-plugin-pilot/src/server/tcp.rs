use super::{EvalFn, FocusFn, ListWindowsFn, handle_connection};

use crate::error::Error;
use crate::eval::EvalEngine;
use crate::recorder::Recorder;

use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener as TokioTcpListener;

pub struct PortFileGuard {
    path: PathBuf,
}

impl Drop for PortFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        tracing::info!(path = %self.path.display(), "TCP port file removed");
    }
}

fn discovery_dir(identifier: &str) -> PathBuf {
    std::env::temp_dir().join("tauri-pilot").join(identifier)
}

pub fn port_file_path(identifier: &str) -> PathBuf {
    discovery_dir(identifier).join("pilot.port")
}

fn ensure_private_dir(path: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    use std::io::Write as _;
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

pub fn bind(identifier: &str) -> Result<(TcpListener, SocketAddr, PortFileGuard), Error> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;

    ensure_private_dir(&std::env::temp_dir().join("tauri-pilot"))?;
    ensure_private_dir(&discovery_dir(identifier))?;
    let path = port_file_path(identifier);
    write_private_file(&path, addr.port().to_string().as_bytes())?;
    tracing::info!(addr = %addr, path = %path.display(), "tauri-pilot TCP transport listening");
    Ok((listener, addr, PortFileGuard { path }))
}

pub async fn run(
    listener: TcpListener,
    _guard: PortFileGuard,
    engine: EvalEngine,
    eval_fn: Option<EvalFn>,
    list_fn: Option<ListWindowsFn>,
    focus_fn: Option<FocusFn>,
    recorder: Recorder,
) {
    let listener = match TokioTcpListener::from_std(listener) {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("failed to convert TCP listener to tokio: {e}");
            return;
        }
    };

    let ctx = Arc::new((engine, eval_fn, list_fn, focus_fn, recorder));
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::warn!("TCP accept error: {e}");
                continue;
            }
        };
        if !addr.ip().is_loopback() {
            tracing::warn!(peer = %addr, "rejected non-loopback TCP peer");
            continue;
        }
        let ctx = Arc::clone(&ctx);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(
                stream,
                &ctx.0,
                ctx.1.as_ref(),
                ctx.2.as_ref(),
                ctx.3.as_ref(),
                &ctx.4,
            )
            .await
            {
                tracing::warn!("TCP connection error: {e}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_writes_private_port_file() {
        let identifier = format!("tcp-test-{}", std::process::id());
        let (_listener, addr, _guard) = bind(&identifier).expect("bind TCP listener");
        let path = port_file_path(&identifier);
        let port = std::fs::read_to_string(&path).expect("read port file");
        assert_eq!(port, addr.port().to_string());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        let _ = std::fs::remove_dir_all(discovery_dir(&identifier));
    }
}
