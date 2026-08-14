//! FTP client for transferring files between local and remote filesystems.
//!
//! This module provides the [`FtpClient`] struct, which wraps an FTP connection
//! and provides methods for uploading and downloading files.
//!
//! # Example
//!
//! ```no_run
//! use data_transfers::ftp::FtpClient;
//!
//! // Connect to FTP server
//! let mut client = FtpClient::connect(
//!     "ftp.example.com",
//!     "21",
//!     "username",
//!     "password"
//! ).expect("Failed to connect to FTP server");
//!
//! // Download a file
//! client.remote_to_local("/remote/file.txt", "local/file.txt")
//!     .expect("Failed to download file");
//! ```

use suppaftp::FtpStream;
use std::fs::File;
use std::io::{self, Write};
use anyhow::{ Context, Result };

/// Client for interacting with an FTP server.
///
/// This struct wraps an FTP connection and provides methods for uploading
/// and downloading files between the local filesystem and remote server.
///
/// # Thread Safety
///
/// FtpClient is Send and Sync. Operations require mutable access to the 
/// client because the underlying FTP connection maintains connection state.
///
/// # Connection Persistence
///
/// The connection remains open after creation and must be kept alive for
/// multiple operations. Some FTP servers may timeout idle connections.
///
/// # Errors
///
/// This client can fail due to:
/// - Network connection failures
/// - Authentication failures
/// - File not found on remote server
/// - Permission issues
/// - File I/O errors (creation, writing, reading)
pub struct FtpClient {
    conn: FtpStream,
}

impl FtpClient {
    /// Opens and authenticates an FTP connection using the configured server credentials.
    ///
    /// # Arguments
    /// * `host` - The FTP server hostname or IP address (e.g., `"ftp.example.com"`)
    /// * `port` - The FTP server port number (e.g., `"21"` for standard FTP, `"22"` for SFTP)
    /// * `username` - The FTP username for authentication
    /// * `password` - The FTP password for authentication
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The server is unreachable or the connection times out
    /// - Authentication fails with invalid credentials
    /// - The port is invalid or not listening
    /// - Network connectivity issues occur
    ///
    /// # Example
    ///
    /// ```no_run
    /// use data_transfers::ftp::FtpClient;
    ///
    /// let mut client = FtpClient::connect(
    ///     "ftp.example.com",
    ///     "21",
    ///     "username",
    ///     "password"
    /// )?;
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    pub fn connect(host: &str, port: &str, username: &str, password: &str) -> Result<Self> {
        // Set up FTP connection to the specified host and port
        let mut conn = FtpStream::connect(format!("{host}:{port}"))
            .with_context(|| format!("Failed to connect to FTP server {host}:{port}"))?;

        // Authenticate with the FTP server using provided credentials
        conn.login(username, password)
            .with_context(|| "Failed to authenticate with FTP server")?;

        Ok(Self{ conn })
    }

    /// Downloads a remote file to the local filesystem.
    pub fn remote_to_local(
        &mut self,
        remote_path: &str,
        local_path: &str,
    ) -> Result<()> {
        if remote_path.is_empty() {
            anyhow::bail!("Remote path cannot be empty");
        }

        if local_path.is_empty() {
            anyhow::bail!("Local path cannot be empty");
        }

        let mut remote_file = self
            .conn
            .retr_as_buffer(remote_path)
            .with_context(|| {format!("Failed to download remote file {remote_path}")})?;

        let mut local_file = File::create(local_path)
            .with_context(|| {format!("Failed to create local file {local_path}")})?;

        io::copy(&mut remote_file, &mut local_file)
            .with_context(|| {format!("Failed to write local file {local_path}")})?;

        Ok(())
    }

    /// Uploads a local file to remote
    pub fn local_to_remote(
        &mut self,
        local_path: &str,
        remote_path: &str,
    ) -> Result<()> {
        if remote_path.is_empty() {
            anyhow::bail!("Remote path cannot be empty");
        }

        if local_path.is_empty() {
            anyhow::bail!("Local path cannot be empty");
        }
        
        let mut local_file = File::open(local_path)
            .with_context(|| {format!("Failed to open local file {local_path}")})?;

        self.conn
            .put_file(remote_path, &mut local_file)
            .with_context(|| {format!("Failed to upload file to {remote_path}")})?;

        Ok(())
    }
}