use crate::control::protocol::{ControlCommand, ControlRequest, ControlResponse};
use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

pub struct InstanceLock {
    _file: std::fs::File,
}

impl InstanceLock {
    /// Attempts to acquire the exclusive non-blocking lifetime lock for the Calix GUI instance.
    /// Returns Ok(Some(lock)) if this process is the sole authoritative instance.
    /// Returns Ok(None) if another process already holds the lock.
    pub fn try_acquire() -> Result<Option<Self>> {
        let dir = ControlSocket::socket_dir();
        Self::try_acquire_in(&dir)
    }

    /// Attempts to acquire the exclusive lifetime lock in the specified directory.
    pub fn try_acquire_in(dir: &std::path::Path) -> Result<Option<Self>> {
        std::fs::create_dir_all(dir)?;

        if let Ok(metadata) = std::fs::metadata(dir) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o700);
            let _ = std::fs::set_permissions(dir, perms);
        }

        let lock_path = dir.join("instance.lock");
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)?;

        if let Ok(metadata) = file.metadata() {
            let mut file_perms = metadata.permissions();
            file_perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&lock_path, file_perms);
        }

        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
        if ret == 0 {
            Ok(Some(Self { _file: file }))
        } else {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EWOULDBLOCK)
                || err.raw_os_error() == Some(libc::EAGAIN)
            {
                Ok(None)
            } else {
                Err(err.into())
            }
        }
    }
}

pub struct ControlSocket;

impl ControlSocket {
    pub fn socket_dir() -> PathBuf {
        dirs::runtime_dir()
            .unwrap_or_else(|| {
                let uid = std::fs::metadata("/proc/self")
                    .map(|m| std::os::unix::fs::MetadataExt::uid(&m))
                    .unwrap_or(1000);
                PathBuf::from(format!("/run/user/{}", uid))
            })
            .join("calix-gpui")
    }

    pub fn socket_path() -> PathBuf {
        Self::socket_dir().join("control.sock")
    }

    /// Checks whether an existing GUI instance is running and responding on the control socket
    pub async fn is_instance_running() -> bool {
        let path = Self::socket_path();
        if !path.exists() {
            return false;
        }

        matches!(
            tokio::time::timeout(
                std::time::Duration::from_millis(150),
                UnixStream::connect(&path),
            )
            .await,
            Ok(Ok(_stream))
        )
    }

    /// Sends a typed command to the running GUI instance over the control socket
    pub async fn send_command(command: ControlCommand) -> Result<ControlResponse> {
        let path = Self::socket_path();
        let stream = UnixStream::connect(&path).await.with_context(|| {
            format!("Failed to connect to control socket at {}", path.display())
        })?;

        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);

        let request = ControlRequest::new(command);
        let mut req_str = serde_json::to_string(&request)?;
        req_str.push('\n');

        writer.write_all(req_str.as_bytes()).await?;
        writer.flush().await?;

        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;

        let response: ControlResponse = serde_json::from_str(&line)
            .with_context(|| format!("Malformed control response: {line}"))?;

        Ok(response)
    }

    /// Binds the Unix domain socket for the GPUI server listener.
    /// Never unlinks a socket that is actively connected and listening.
    pub fn bind_listener() -> Result<UnixListener> {
        let dir = Self::socket_dir();
        std::fs::create_dir_all(&dir)?;

        // Enforce user-only directory permissions 0700
        let mut dir_perms = std::fs::metadata(&dir)?.permissions();
        dir_perms.set_mode(0o700);
        let _ = std::fs::set_permissions(&dir, dir_perms);

        let path = Self::socket_path();
        if path.exists() {
            // Check if existing socket is active; never unlink a healthy listener socket!
            if std::os::unix::net::UnixStream::connect(&path).is_ok() {
                anyhow::bail!(
                    "Cannot bind listener: active instance is already listening on {}",
                    path.display()
                );
            }
            // Stale socket confirmed unreachable; safe to unlink
            let _ = std::fs::remove_file(&path);
        }

        let listener = UnixListener::bind(&path)?;

        // Enforce user-only socket permissions 0600
        let mut sock_perms = std::fs::metadata(&path)?.permissions();
        sock_perms.set_mode(0o600);
        let _ = std::fs::set_permissions(&path, sock_perms);

        Ok(listener)
    }

    /// Cleans up the socket file upon clean exit
    pub fn cleanup() {
        let path = Self::socket_path();
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_socket_path_resolution() {
        let path = ControlSocket::socket_path();
        assert!(path.ends_with("calix-gpui/control.sock"));
    }

    #[test]
    fn test_instance_lock_acquisition() {
        let test_dir = ControlSocket::socket_dir().join(".test_instance_lock");
        let _ = std::fs::create_dir_all(&test_dir);

        let lock1 =
            InstanceLock::try_acquire_in(&test_dir).expect("First acquisition should succeed");
        assert!(lock1.is_some());

        // Second acquisition while first is held must return None (EWOULDBLOCK)
        let lock2 =
            InstanceLock::try_acquire_in(&test_dir).expect("Second attempt should not error");
        assert!(lock2.is_none());

        // Dropping first lock allows subsequent acquisition
        drop(lock1);
        let lock3 = InstanceLock::try_acquire_in(&test_dir)
            .expect("Re-acquisition after drop should succeed");
        assert!(lock3.is_some());

        drop(lock3);
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
