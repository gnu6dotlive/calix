//! The Calix daemon's socket.
//!
//! The frontend never reads the Calix state directory. Everything it knows
//! about the ecosystems the CLI backend does not own comes from
//! `calix-daemon`, as a versioned snapshot over a Unix socket.
//!
//! A frontend with no daemon running is not broken. It renders the packages
//! the CLI backend returns, and the extra ecosystems are simply absent. That
//! is a deliberate degradation: the daemon is a second source, not a
//! prerequisite for the window to open.

use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use super::calix_snapshot::Snapshot;

/// The wire protocol version. Distinct from the snapshot's format version:
/// the envelope can change without the document changing, and vice versa.
pub const PROTOCOL_VERSION: u32 = 1;

/// How long to wait for the socket to accept a connection.
///
/// A missing daemon fails immediately with `NotFound`; this bound only
/// matters for a socket that exists but is not answering.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// How long to wait for the inventory.
///
/// Gathering reads six package databases and shells out to `npm list -g`,
/// so this is seconds rather than milliseconds. It is bounded anyway: a
/// wedged daemon must not wedge the window.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

/// The socket the daemon listens on, by default.
pub fn socket_path() -> PathBuf {
    let runtime_directory = std::env::var_os("XDG_RUNTIME_DIR");
    // SAFETY: `getuid` has no preconditions and cannot fail.
    let uid = unsafe { libc::getuid() };
    path_for(runtime_directory.as_deref(), uid)
}

/// The socket path for a given runtime directory and uid.
///
/// Mirrors the daemon's own defaults exactly: `$XDG_RUNTIME_DIR/calix/daemon.sock`,
/// falling back to `/tmp/calix-<uid>/daemon.sock` when the session has no
/// runtime directory.
fn path_for(runtime_directory: Option<&std::ffi::OsStr>, uid: u32) -> PathBuf {
    match runtime_directory {
        Some(directory) if !directory.is_empty() => {
            PathBuf::from(directory).join("calix").join("daemon.sock")
        }
        _ => PathBuf::from(format!("/tmp/calix-{uid}")).join("daemon.sock"),
    }
}

/// Fetches the inventory from the default socket.
pub async fn inventory() -> Result<Snapshot, String> {
    inventory_at(&socket_path()).await
}

/// Fetches the inventory from a specific socket.
pub async fn inventory_at(path: &Path) -> Result<Snapshot, String> {
    let stream = tokio::time::timeout(CONNECT_TIMEOUT, UnixStream::connect(path))
        .await
        .map_err(|_| format!("timed out connecting to the Calix daemon at {}", path.display()))?
        .map_err(|error| {
            format!(
                "no Calix daemon answering at {}: {error}",
                path.display()
            )
        })?;

    let (read_half, mut write_half) = stream.into_split();
    let request = format!("{{\"version\":{PROTOCOL_VERSION},\"action\":\"inventory\"}}\n");
    write_half
        .write_all(request.as_bytes())
        .await
        .map_err(|error| format!("cannot send the inventory request: {error}"))?;
    write_half
        .flush()
        .await
        .map_err(|error| format!("cannot send the inventory request: {error}"))?;

    let mut reader = BufReader::new(read_half);
    let mut response = String::new();
    tokio::time::timeout(RESPONSE_TIMEOUT, reader.read_line(&mut response))
        .await
        .map_err(|_| "the Calix daemon did not answer in time".to_string())?
        .map_err(|error| format!("cannot read the inventory response: {error}"))?;

    if response.is_empty() {
        return Err("the Calix daemon closed the connection without answering".to_string());
    }
    parse_response(&response)
}

/// The envelope every daemon response is wrapped in.
#[derive(Debug, serde::Deserialize)]
struct Envelope {
    version: u32,
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

/// Unwraps one response line into a snapshot.
fn parse_response(line: &str) -> Result<Snapshot, String> {
    let envelope: Envelope = serde_json::from_str(line)
        .map_err(|error| format!("the daemon sent a malformed response: {error}"))?;

    if envelope.version != PROTOCOL_VERSION {
        return Err(format!(
            "the daemon speaks protocol version {} (this frontend speaks {})",
            envelope.version, PROTOCOL_VERSION
        ));
    }
    if !envelope.ok {
        let detail = envelope
            .message
            .or(envelope.error)
            .unwrap_or_else(|| "no reason given".to_string());
        return Err(format!("the daemon refused the request: {detail}"));
    }

    let data = envelope
        .data
        .ok_or_else(|| "the daemon answered without a document".to_string())?;
    Snapshot::from_value(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_socket_follows_the_runtime_directory() {
        let path = path_for(Some(std::ffi::OsStr::new("/run/user/1000")), 1000);
        assert_eq!(path, PathBuf::from("/run/user/1000/calix/daemon.sock"));
    }

    #[test]
    fn without_a_runtime_directory_the_socket_is_keyed_by_uid() {
        assert_eq!(
            path_for(None, 1000),
            PathBuf::from("/tmp/calix-1000/daemon.sock")
        );
        // An empty variable is not a directory.
        assert_eq!(
            path_for(Some(std::ffi::OsStr::new("")), 42),
            PathBuf::from("/tmp/calix-42/daemon.sock")
        );
    }

    #[test]
    fn a_well_formed_answer_becomes_a_snapshot() {
        let line = r#"{"version":1,"ok":true,"data":{"version":1,"generated_at":1,"packages":[{"name":"rg","version":"14.1.1","source_type":"Cargo","binaries":["rg"]}]}}"#;
        let snapshot = parse_response(line).expect("the response must parse");
        assert_eq!(snapshot.packages.len(), 1);
        assert_eq!(snapshot.packages[0].binaries, vec!["rg".to_string()]);
    }

    #[test]
    fn a_refusal_carries_the_daemons_reason() {
        let line = r#"{"version":1,"ok":false,"error":"unknown-action","message":"unknown action"}"#;
        let error = parse_response(line).expect_err("a refusal is an error");
        assert!(error.contains("unknown action"));
    }

    #[test]
    fn a_protocol_version_the_frontend_does_not_speak_is_refused() {
        let line = r#"{"version":2,"ok":true,"data":{"version":1,"packages":[]}}"#;
        let error = parse_response(line).expect_err("a protocol mismatch is an error");
        assert!(error.contains("protocol version 2"));
    }

    #[test]
    fn a_snapshot_version_the_frontend_does_not_know_is_refused() {
        let line = r#"{"version":1,"ok":true,"data":{"version":9,"packages":[]}}"#;
        let error = parse_response(line).expect_err("a format mismatch is an error");
        assert!(error.contains("unsupported inventory version 9"));
    }

    #[tokio::test]
    async fn a_snapshot_arrives_over_a_real_socket() {
        let directory =
            std::env::temp_dir().join(format!("calix-daemon-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("daemon.sock");
        let _ = std::fs::remove_file(&path);

        let listener = tokio::net::UnixListener::bind(&path).expect("the test binds its own socket");
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("one connection");
            let (read_half, mut write_half) = stream.into_split();
            let mut request = String::new();
            BufReader::new(read_half)
                .read_line(&mut request)
                .await
                .expect("the request is one line");
            assert!(request.contains("\"inventory\""), "got {request:?}");
            write_half
                .write_all(
                    b"{\"version\":1,\"ok\":true,\"data\":{\"version\":1,\"packages\":[{\"name\":\"rg\",\"version\":\"14.1.1\",\"source_type\":\"Cargo\"}]}}\n",
                )
                .await
                .expect("the response is written");
        });

        let snapshot = inventory_at(&path).await.expect("the snapshot arrives");
        assert_eq!(snapshot.packages.len(), 1);
        assert_eq!(snapshot.packages[0].name, "rg");
        server.await.expect("the server task finishes");

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&directory).ok();
    }

    #[tokio::test]
    async fn a_missing_daemon_says_so_instead_of_hanging() {
        let path = std::env::temp_dir().join(format!(
            "calix-no-daemon-{}-{}.sock",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0)
        ));
        let error = inventory_at(&path).await.expect_err("there is no daemon there");
        assert!(error.contains("no Calix daemon answering"), "got {error}");
    }

    #[test]
    fn absence_of_a_document_is_an_error_not_an_empty_list() {
        let line = r#"{"version":1,"ok":true}"#;
        let error = parse_response(line).expect_err("a missing document is an error");
        assert!(error.contains("without a document"));
    }
}
