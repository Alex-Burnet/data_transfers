use anyhow::{Context, Result};
use ssh2::{Session, Sftp};
use std::{fs::File, io, net::TcpStream, path::Path};

/// SFTP client for transferring files to and from a remote server.
/// 
/// This struct wraps an SFTP connection and provides methods for uploading and downloading files.
/// 
/// # Thread Safety
/// SftpClient is Send and Sync. Operations require mutable access to the 
/// client because the underlying SFTP connection maintains connection state.
/// 
/// # Connection Persistence
/// The connection remains open after creation and must be kept alive for multiple operations.
pub struct SftpClient {
    conn: Sftp,
}

impl SftpClient {
    /// Connects to an SFTP server and authenticates with the provided credentials.
    /// 
    /// # Arguments
    /// * `host` - The hostname or IP address of the SFTP server.
    /// * `port` - The port number of the SFTP server.
    /// * `username` - The username for authentication.
    /// * `password` - The password for authentication.
    /// 
    /// # Returns
    /// A result containing the connected SftpClient or an error.
    /// 
    /// # Errors
    /// Returns an error if:
    /// * The connection to the SFTP server fails.
    /// * Authentication with the SFTP server fails.
    /// 
    /// # Example
    /// ```no_run
    /// use data_transfers::sftp::SftpClient;
    /// 
    /// let client = SftpClient::connect("example.com", "22", "username", "password").unwrap();
    /// ```
    pub fn connect(host: &str, port: &str, username: &str, password: &str) -> Result<Self> {
        let tcp = TcpStream::connect(format!("{host}:{port}"))
            .with_context(|| format!("Failed to connect to SFTP server {host}:{port}"))?;

        // Create a new SSH session and perform the handshake
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        // Authenticate with the SFTP server using the provided username and password
        session
            .userauth_password(username, password)
            .with_context(|| "Failed to authenticate with SFTP server")?;

        // Check if authentication was successful
        if !session.authenticated() {
            anyhow::bail!("Authentication failed");
        }

        // Initialize the SFTP subsystem
        let sftp = session.sftp()?;

        Ok(Self{ conn: sftp })
    }

    /// Uploads a file from the local filesystem to the remote SFTP server.
    /// 
    /// # Arguments
    /// * `local_path` - The path to the local file to upload.
    /// * `remote_path` - The path on the remote SFTP server where the file will be uploaded.
    /// 
    /// # Returns
    /// A result indicating success or failure.
    pub fn local_to_remote(&self, local_path: &str, remote_path: &str) -> Result<()> {
        if remote_path.is_empty() {
            anyhow::bail!("Remote path cannot be empty");
        }

        if local_path.is_empty() {
            anyhow::bail!("Local path cannot be empty");
        }

        let mut local =
            File::open(local_path).with_context(|| format!("Failed to open {}", local_path))?;

        let mut remote = self.conn
            .create(Path::new(remote_path))
            .with_context(|| format!("Failed to create remote file {}", remote_path))?;

        io::copy(&mut local, &mut remote)
            .with_context(|| {format!("Failed to copy {} to {}", local_path, remote_path)})?;

        Ok(())
    }

    /// Downloads a file from the remote SFTP server to the local filesystem.
    /// 
    /// # Arguments
    /// * `remote_path` - The path on the remote SFTP server of the file to download.
    /// * `local_path` - The path where the file will be saved locally.
    /// 
    /// # Returns
    /// A result indicating success or failure.
    pub fn remote_to_local(&self, remote_path: &str, local_path: &str) -> Result<()> {
        if remote_path.is_empty() {
            anyhow::bail!("Remote path cannot be empty");
        }

        if local_path.is_empty() {
            anyhow::bail!("Local path cannot be empty");
        }
        
        let mut remote = self.conn
            .open(Path::new(remote_path))
            .with_context(|| format!("Failed to open remote file {}", remote_path))?;

        let mut local = File::create(local_path)
            .with_context(|| format!("Failed to create {}", local_path))?;

        io::copy(&mut remote, &mut local)
            .with_context(|| {format!("Failed to copy {} to {}", remote_path, local_path)})?;

        Ok(())
    }
}