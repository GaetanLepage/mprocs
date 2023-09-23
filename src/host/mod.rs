use std::{path::PathBuf, time::Duration};

use crate::error::ResultLogger;

fn get_socket_path() -> PathBuf {
  let mut path = std::env::temp_dir();
  path.push("dekit.sock");
  path
}

pub fn create_server_socket() -> anyhow::Result<ServerSocket> {
  let path = get_socket_path();

  let bind = || tokio::net::UnixListener::bind(&path);
  let listener = match bind() {
    Ok(listener) => listener,
    Err(err) => match err.kind() {
      std::io::ErrorKind::AddrInUse => {
        std::fs::remove_file(&path)?;
        bind()?
      }
      _ => return Err(err.into()),
    },
  };

  Ok(ServerSocket { path, listener })
}

pub struct ServerSocket {
  path: PathBuf,
  listener: tokio::net::UnixListener,
}

impl Drop for ServerSocket {
  fn drop(&mut self) {
    std::fs::remove_file(&self.path).log_ignore();
  }
}

impl ServerSocket {
  pub fn listener(&mut self) -> &mut tokio::net::UnixListener {
    &mut self.listener
  }
}

pub async fn connect_client_socket() -> anyhow::Result<tokio::net::UnixStream> {
  let path = get_socket_path();
  loop {
    match tokio::net::UnixStream::connect(&path).await {
      Ok(socket) => return Ok(socket),
      Err(_) => {
        tokio::time::sleep(Duration::from_millis(20)).await;
      }
    }
  }
}
